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
    todo!()
}

/// The bodies of the fenced blocks tagged `lang`.
fn fenced_blocks<'a>(message: &'a str, lang: &str) -> Vec<&'a str> {
    todo!()
}

/// The commit message's subject must carry this phase's tag.
#[implements(spec::ACommitSubjectMustCarryThisPhasesTag)]
pub fn subject_matches(phase: Phase, message: &str) -> Result<(), String> {
    todo!()
}

/// The message's subject: its first non-empty line.
fn subject_of(message: &str) -> &str {
    todo!()
}

/// The refusal for a failing check: the output, the `gates.md` row for the
/// check it names, and what the phase permits.
#[implements(spec::ARefusalCarriesTheOutputTheRuleAndThePermittedMoves)]
pub fn refusal_for(project: &Project, phase: Phase, output: &str) -> String {
    todo!()
}

/// The check a failing output names, if it names one.
#[implements(spec::AFailingOutputNamesItsCheck)]
pub fn check_of_output(output: &str) -> Option<Check> {
    todo!()
}

/// The check a clippy lint name maps to.
#[implements(spec::AFailingOutputNamesItsCheck)]
fn check_of_lint(lint: &str) -> Option<Check> {
    todo!()
}

/// The `gates.md` row for a check, from the synced skill.
fn gates_row(project: &Project, check: Check) -> Result<String, String> {
    todo!()
}

/// What the phase's policy lets the agent do about a failure.
fn permitted_moves(phase: Phase) -> String {
    todo!()
}

/// Stages exactly `paths` and commits `message` with `trailers`; the new
/// commit's hash.
#[implements(spec::OnlyThePoliciesPathsAreStaged)]
pub fn stage_and_commit(project: &Project, paths: &[&Path], message: &str, trailers: &str) -> Result<String, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::fixture;
    use lid_rs::validates;

    #[test]
    #[validates(spec::AFinalMessageCarriesExactlyOneEnding)]
    fn a_final_message_carries_exactly_one_ending() {
        let commit = ending_of("Done.\n\n```commit\nphase 3: skeleton for x\n\nBody.\n```\n").expect("one commit block");
        assert_eq!(commit, Ending::Commit("phase 3: skeleton for x\n\nBody.\n".to_string()));
        let stop = ending_of("```stop\n1. Needs a claim.\n```").expect("one stop block");
        assert_eq!(stop, Ending::Stop("1. Needs a claim.\n".to_string()));
        let none = ending_of("I finished the phase.").expect_err("no block");
        assert!(none.contains("```commit") && none.contains("```stop"), "{none}");
        assert!(ending_of("```commit\na\n```\n```stop\nb\n```").is_err(), "both is an error");
        assert!(ending_of("```commit\na\n```\n```commit\nb\n```").is_err(), "two commit blocks is an error");
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
    fn a_failing_output_names_its_check() {
        assert_eq!(check_of_lint("cognitive_complexity"), Some(Check::Seven));
        assert_eq!(check_of_lint("fn_params_excessive_bools"), Some(Check::Eight));
        assert_eq!(check_of_lint("too_many_lines"), Some(Check::Nine));
        assert_eq!(check_of_lint("wildcard_enum_match_arm"), Some(Check::Six));
        assert_eq!(check_of_lint("missing_docs"), Some(Check::Three));
        assert_eq!(check_of_lint("needless_return"), None);
        assert_eq!(check_of_output("error: the function has a cognitive complexity of (5/4)\n = note: `-D clippy::cognitive-complexity`"), Some(Check::Seven));
        assert_eq!(check_of_output("error: more than 0 bools in function parameters\n#[deny(clippy::fn_params_excessive_bools)]"), Some(Check::Eight));
        assert_eq!(check_of_output("phase 5 is not red:\n  test x passes before implementation exists"), Some(Check::RedRun));
        assert_eq!(check_of_output("16 mutant(s) survived their validating tests"), Some(Check::Twelve));
        assert_eq!(check_of_output("error[E0308]: mismatched types"), None);
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
        let show = std::process::Command::new("git").args(["show", "--stat", "--format=%B", "HEAD"]).current_dir(&dir).output().expect("git");
        let show = String::from_utf8_lossy(&show.stdout);
        assert!(show.contains("src/hello.rs") && !show.contains("README.md"), "{show}");
        assert!(show.contains("Body.") && show.trim_end().ends_with("Lid-Rs-Phase: 3"), "{show}");
    }
}
