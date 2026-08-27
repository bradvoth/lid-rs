//! The stop protocol (`docs/intent/phase/lld.md` § `hook stop`): how a phase
//! agent's final message ends the phase, and what a refusal tells it.

use std::path::Path;

use lid_rs::implements;

use super::Phase;
use crate::project::Project;
use crate::spec;

/// How the agent's final message ends the phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    /// A ```` ```commit ```` block: the proposed commit message.
    Commit(String),
    /// A ```` ```stop ```` block: the numbered decisions that block the phase.
    Stop(String),
}

/// The check a failing output belongs to, for the `gates.md` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// `missing_docs`.
    Three,
    /// `wildcard_enum_match_arm`.
    Six,
    /// `cognitive_complexity`.
    Seven,
    /// `fn_params_excessive_bools`.
    Eight,
    /// `too_many_lines`.
    Nine,
    /// The Phase 5 red run.
    RedRun,
    /// A surviving mutant.
    Twelve,
}

/// The message's ending, or the format when it has none or both.
#[implements(spec::AFinalMessageCarriesExactlyOneEnding)]
pub fn ending_of(message: &str) -> Result<Ending, String> {
    let commits = fenced_blocks(message, "commit");
    let stops = fenced_blocks(message, "stop");
    match (commits.as_slice(), stops.as_slice()) {
        ([commit], []) => Ok(Ending::Commit((*commit).to_string())),
        ([], [stop]) => Ok(Ending::Stop((*stop).to_string())),
        ([], []) | ([_, ..], [_, ..]) | ([_, _, ..], []) | ([], [_, _, ..]) => Err(
            "the final message must end the phase with exactly one fenced block: ```commit (the commit message, subject \
             `phase N: …`) or ```stop (the numbered decisions that block the phase)"
                .to_string(),
        ),
    }
}

/// The bodies of the fenced blocks tagged `lang`.
fn fenced_blocks<'a>(message: &'a str, lang: &str) -> Vec<&'a str> {
    let open = format!("```{lang}");
    let mut blocks = Vec::new();
    let mut start: Option<usize> = None;
    let mut offset = 0;
    for line in message.split_inclusive('\n') {
        let trimmed = line.trim();
        match (start, trimmed == open, trimmed == "```") {
            (None, true, _) => start = Some(offset + line.len()),
            (Some(from), _, true) => {
                blocks.push(&message[from..offset]);
                start = None;
            }
            (None, false, _) | (Some(_), _, false) => {}
        }
        offset += line.len();
    }
    blocks
}

/// The commit message's subject must carry this phase's tag.
#[implements(spec::ACommitSubjectMustCarryThisPhasesTag)]
pub fn subject_matches(phase: Phase, message: &str) -> Result<(), String> {
    let expected = super::policy::number_of(phase);
    match super::tag_of(subject_of(message)) {
        super::Tag::Checked(tagged) if tagged == phase => Ok(()),
        super::Tag::Checked(_) | super::Tag::Untagged | super::Tag::Unchecked(_) => Err(format!(
            "the commit subject must begin `phase {expected}:` — this agent runs Phase {expected}; it began `{}`",
            subject_of(message)
        )),
    }
}

/// The message's subject: its first non-empty line.
fn subject_of(message: &str) -> &str {
    message.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("")
}

/// The refusal for a failing check: the output, the `gates.md` row for the
/// check it names, and what the phase permits.
#[implements(spec::ARefusalCarriesTheOutputTheRuleAndThePermittedMoves)]
pub fn refusal_for(project: &Project, phase: Phase, output: &str) -> String {
    let rule = check_of_output(output)
        .map(|check| gates_row(project, check).unwrap_or_else(|e| e))
        .unwrap_or_else(|| "No gate row names this failure: it is a plain compile or test error, to fix where it points.".to_string());
    format!("The phase's check failed:\n\n{output}\n\nThe skill's response to this check:\n{rule}\n\n{}", permitted_moves(phase))
}

/// The check a failing output names, if it names one.
#[implements(spec::AFailingOutputNamesItsCheck)]
pub fn check_of_output(output: &str) -> Option<Check> {
    lint_names(output).find_map(|lint| check_of_lint(&lint)).or_else(|| marker_check(output))
}

/// The lint names clippy printed, as `clippy::name` with `-` or `_`.
fn lint_names(output: &str) -> impl Iterator<Item = String> + '_ {
    output.match_indices("clippy::").map(|(at, _)| {
        output[at + "clippy::".len()..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .map(|c| if c == '-' { '_' } else { c })
            .collect()
    })
}

/// The non-clippy failures the gate names: the red run and check 12.
fn marker_check(output: &str) -> Option<Check> {
    [("is not red", Check::RedRun), ("survived", Check::Twelve)]
        .iter()
        .find(|(marker, _)| output.contains(marker))
        .map(|(_, check)| *check)
}

/// The check a clippy lint name maps to.
#[implements(spec::AFailingOutputNamesItsCheck)]
fn check_of_lint(lint: &str) -> Option<Check> {
    match lint {
        "cognitive_complexity" => Some(Check::Seven),
        "fn_params_excessive_bools" => Some(Check::Eight),
        "too_many_lines" => Some(Check::Nine),
        "wildcard_enum_match_arm" => Some(Check::Six),
        "missing_docs" => Some(Check::Three),
        _ => None,
    }
}

/// The `gates.md` row for a check, from the synced skill.
fn gates_row(project: &Project, check: Check) -> Result<String, String> {
    match check {
        Check::RedRun => Ok(
            "Phase 5 requires every validation to fail against the skeleton (references/phase-5.md): a test that is green \
             before implementation exists needs an explanation, never a pass."
                .to_string(),
        ),
        Check::Three | Check::Six | Check::Seven | Check::Eight | Check::Nine | Check::Twelve => {
            let label = format!("check {}", check_number(check));
            super::policy::read_synced(project, "references/gates.md")?
                .lines()
                .find(|line| line.starts_with('|') && line.contains(&label))
                .map(str::to_string)
                .ok_or_else(|| format!("references/gates.md has no row for {label}"))
        }
    }
}

/// The number a check is written as in `gates.md`.
fn check_number(check: Check) -> u8 {
    match check {
        Check::Three => 3,
        Check::Six => 6,
        Check::Seven => 7,
        Check::Eight => 8,
        Check::Nine => 9,
        Check::RedRun => 5,
        Check::Twelve => 12,
    }
}

/// What the phase's policy lets the agent do about a failure.
fn permitted_moves(phase: Phase) -> String {
    let allowed: Vec<String> = super::policy::allowed_paths(phase, "<slice>").iter().map(|p| format!("`{}`", p.display())).collect();
    format!(
        "What Phase {} permits: fix it within {} and end again with a ```commit block — the LLD (docs/intent) and the claims \
         are not this phase's to change — or end with a ```stop block naming the decision this needs.",
        super::policy::number_of(phase),
        allowed.join(", ")
    )
}

/// Stages exactly `paths` and commits `message` with `trailers`; the new
/// commit's hash.
#[implements(spec::OnlyThePoliciesPathsAreStaged)]
pub fn stage_and_commit(project: &Project, paths: &[&Path], message: &str, trailers: &str) -> Result<String, String> {
    let root = project.root()?;
    let stageable: Vec<&Path> = paths.iter().copied().filter(|p| root.join(p).exists() || tracked(project, p)).collect();
    let mut add = project.git()?;
    add.args(["add", "-A", "--"]).args(stageable.iter().map(|p| p.as_os_str()));
    crate::project::capture(&mut add)?;
    commit_with_file(project, &format!("{}\n\n{trailers}", message.trim_end()))?;
    Ok(crate::project::capture(project.git()?.args(["rev-parse", "HEAD"]))?.trim().to_string())
}

/// Commits with the message in a temporary file.
fn commit_with_file(project: &Project, message: &str) -> Result<String, String> {
    let file = project.target_directory()?.join("lid-rs/commit-message");
    let parent = file.parent().ok_or("no target directory")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    std::fs::write(&file, message).map_err(|e| format!("writing {}: {e}", file.display()))?;
    crate::project::capture(project.git()?.args(["commit", "-q", "-F"]).arg(&file))
}

/// Whether git tracks a path (a deleted file is still stageable).
fn tracked(project: &Project, path: &Path) -> bool {
    project
        .git()
        .ok()
        .and_then(|mut git| git.args(["ls-files", "--error-unmatch", "--"]).arg(path).output().ok())
        .is_some_and(|out| out.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::fixture;
    use lid_rs::validates;

    #[test]
    #[validates(spec::AFinalMessageCarriesExactlyOneEnding)]
    fn one_block_is_the_ending() {
        let commit = ending_of("Done.\n\n```commit\nphase 3: skeleton for x\n\nBody.\n```\n").expect("one commit block");
        assert_eq!(commit, Ending::Commit("phase 3: skeleton for x\n\nBody.\n".to_string()));
        let stop = ending_of("```stop\n1. Needs a claim.\n```").expect("one stop block");
        assert_eq!(stop, Ending::Stop("1. Needs a claim.\n".to_string()));
    }

    #[test]
    #[validates(spec::AFinalMessageCarriesExactlyOneEnding)]
    fn no_block_or_more_than_one_is_refused_with_the_format() {
        let cases = ["I finished the phase.", "```commit\na\n```\n```stop\nb\n```", "```commit\na\n```\n```commit\nb\n```", "```stop\na\n```\n```stop\nb\n```"];
        for message in cases {
            let err = ending_of(message).expect_err(message);
            assert!(err.contains("```commit") && err.contains("```stop"), "{err}");
        }
    }

    #[test]
    #[validates(spec::ACommitSubjectMustCarryThisPhasesTag)]
    fn a_commit_subject_must_carry_this_phases_tag() {
        subject_matches(Phase::Three, "phase 3: skeleton for x\n\nbody").expect("matches");
        subject_matches(Phase::Seven, "\nphase 7: 0.3.0: the thing\n").expect("matches after a blank line");
        let err = subject_matches(Phase::Three, "phase 2: claims").expect_err("another phase's tag");
        assert!(err.contains("phase 3:"), "{err}");
        assert!(subject_matches(Phase::Three, "skeleton").is_err(), "no tag");
        assert!(subject_matches(Phase::Three, "phase 6: leaves").is_err(), "a phase with no check");
    }

    #[test]
    #[validates(spec::AFailingOutputNamesItsCheck)]
    fn a_clippy_lint_names_its_check() {
        let cases = [
            ("cognitive_complexity", Some(Check::Seven)),
            ("fn_params_excessive_bools", Some(Check::Eight)),
            ("too_many_lines", Some(Check::Nine)),
            ("wildcard_enum_match_arm", Some(Check::Six)),
            ("missing_docs", Some(Check::Three)),
            ("needless_return", None),
        ];
        for (lint, check) in cases {
            assert_eq!(check_of_lint(lint), check, "{lint}");
        }
    }

    #[test]
    #[validates(spec::AFailingOutputNamesItsCheck)]
    fn a_failing_output_names_its_check() {
        let cases = [
            ("error: the function has a cognitive complexity of (5/4)\n = note: `-D clippy::cognitive-complexity`", Some(Check::Seven)),
            ("error: more than 0 bools in function parameters\n#[deny(clippy::fn_params_excessive_bools)]", Some(Check::Eight)),
            ("phase 5 is not red:\n  test x passes before implementation exists", Some(Check::RedRun)),
            ("16 mutant(s) survived their validating tests", Some(Check::Twelve)),
            ("error[E0308]: mismatched types", None),
        ];
        for (output, check) in cases {
            assert_eq!(check_of_output(output), check, "{output}");
        }
    }

    #[test]
    #[validates(spec::ARefusalCarriesTheOutputTheRuleAndThePermittedMoves)]
    fn a_refusal_carries_the_output_the_rule_and_the_permitted_moves() {
        let workspace = fixture::workspace();
        let output = "error: the function has a cognitive complexity of (5/4)\n  --> src/phase.rs:10:1\n  = note: `-D clippy::cognitive-complexity`";
        let reason = refusal_for(&workspace, Phase::Seven, output);
        let (out_at, rule_at, moves_at) = (
            reason.find("cognitive complexity of (5/4)").expect("the output"),
            reason.find("Return to Phase 1").expect("the gates row for check 7"),
            reason.find("```stop").expect("what the phase permits"),
        );
        assert!(out_at < rule_at && rule_at < moves_at, "in order: {reason}");
        assert!(reason.contains("docs/intent"), "the LLD is not this phase's to edit: {reason}");
        let unknown = refusal_for(&workspace, Phase::Three, "error[E0308]: mismatched types");
        assert!(unknown.contains("E0308") && unknown.contains("```stop"), "{unknown}");
    }

    #[test]
    #[validates(spec::OnlyThePoliciesPathsAreStaged)]
    fn stage_and_commit_stages_exactly_the_given_paths() {
        let (dir, project) = fixture::copy("stage");
        std::fs::write(dir.join("src/hello.rs"), "//! staged\n").expect("write");
        std::fs::write(dir.join("README.md"), "not staged").expect("write");
        let sha = stage_and_commit(&project, &[Path::new("src/hello.rs"), Path::new("src/hello")], "phase 3: skeleton for hello\n\nBody.\n", "Lid-Rs-Phase: 3\n").expect("commits");
        assert_eq!(fixture::head(&dir), sha);
        let stat = std::process::Command::new("git").args(["show", "--stat", "--format=", "HEAD"]).current_dir(&dir).output().expect("git");
        let stat = String::from_utf8_lossy(&stat.stdout);
        assert!(stat.contains("src/hello.rs") && !stat.contains("README.md"), "{stat}");
        let body = std::process::Command::new("git").args(["log", "-1", "--format=%B"]).current_dir(&dir).output().expect("git");
        let body = String::from_utf8_lossy(&body.stdout);
        assert!(body.contains("Body.") && body.trim_end().ends_with("Lid-Rs-Phase: 3"), "{body}");
    }
}
