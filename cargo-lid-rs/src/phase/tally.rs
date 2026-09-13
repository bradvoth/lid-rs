//! The per-agent tally (`docs/intent/phase/lld.md` § the tally): every tool
//! call, check, and refusal, counted under the agent's id and written into
//! the phase commit as trailers.

use std::path::PathBuf;

use lid_rs::implements;

use super::Phase;
use super::Replaced;
use super::policy::ToolKind;
use crate::project::Project;
use super::spec;

/// One thing the hooks count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A tool call of this kind reached the pre-tool hook.
    Tool(ToolKind),
    /// The post-edit hook ran clippy.
    PostEditCheck,
    /// The stop hook ran the phase's check.
    StopCheck,
    /// The policy refused an edit.
    PolicyRefusal,
    /// The stop hook refused a stop.
    StopRefusal,
}

/// The counts for one agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    /// Edits and writes.
    pub edits: u32,
    /// Reads, searches, LSP queries.
    pub observations: u32,
    /// Commands — zero, by the agents' tool lists.
    pub commands: u32,
    /// Post-edit clippy runs.
    pub post_edit_checks: u32,
    /// Stop-hook check runs.
    pub stop_checks: u32,
    /// Edits the policy refused.
    pub policy_refusals: u32,
    /// Stops the stop hook refused.
    pub stop_refusals: u32,
}

/// Where an agent's tally lives: `<target>/lid-rs/agents/<agent_id>.json`.
fn path(project: &Project, agent_id: &str) -> Result<PathBuf, String> {
    Ok(project.target_directory()?.join("lid-rs/agents").join(format!("{agent_id}.json")))
}

/// The tally as stored.
fn to_json(tally: &Tally) -> String {
    serde_json::json!({
        "edits": tally.edits, "observations": tally.observations, "commands": tally.commands,
        "post_edit_checks": tally.post_edit_checks, "stop_checks": tally.stop_checks,
        "policy_refusals": tally.policy_refusals, "stop_refusals": tally.stop_refusals,
    })
    .to_string()
}

/// A stored tally.
fn from_json(json: &str) -> Result<Tally, String> {
    let doc: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("parsing a tally: {e}"))?;
    let count = |key: &str| doc.pointer(&format!("/{key}")).and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
    Ok(Tally {
        edits: count("edits"),
        observations: count("observations"),
        commands: count("commands"),
        post_edit_checks: count("post_edit_checks"),
        stop_checks: count("stop_checks"),
        policy_refusals: count("policy_refusals"),
        stop_refusals: count("stop_refusals"),
    })
}

/// The agent's tally so far; empty for an agent with none.
pub fn load(project: &Project, agent_id: &str) -> Result<Tally, String> {
    std::fs::read_to_string(path(project, agent_id)?).ok().map_or(Ok(Tally::default()), |json| from_json(&json))
}

/// Counts one event under the agent's id.
#[implements(spec::EveryToolCallIsTallied)]
pub fn record(project: &Project, agent_id: &str, event: Event) -> Result<(), String> {
    let updated = apply(load(project, agent_id)?, event);
    let file = path(project, agent_id)?;
    let parent = file.parent().ok_or_else(|| format!("{} has no parent", file.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    std::fs::write(&file, to_json(&updated)).map_err(|e| format!("writing {}: {e}", file.display()))
}

/// One event applied to a tally.
#[implements(spec::EveryToolCallIsTallied)]
pub fn apply(tally: Tally, event: Event) -> Tally {
    let mut next = tally;
    match event {
        Event::Tool(ToolKind::Edit) => next.edits += 1,
        Event::Tool(ToolKind::Observation) => next.observations += 1,
        Event::Tool(ToolKind::Command) => next.commands += 1,
        Event::PostEditCheck => next.post_edit_checks += 1,
        Event::StopCheck => next.stop_checks += 1,
        Event::PolicyRefusal => next.policy_refusals += 1,
        Event::StopRefusal => next.stop_refusals += 1,
    }
    next
}

/// The six `Lid-Rs-*` trailers of a phase commit: the phase, every agent
/// whose work the commit carries — two or more separated by `", "` — the
/// counts, and how many commits this one replaced. The renderer decides
/// nothing: the agent list's order and the reworks' arithmetic are answered
/// before it and handed to it, which is why it is written whole while the
/// leaves that answer them are not.
#[implements(spec::APhaseCommitEndsWithTheSixTrailersNamingEveryAgentAndTheReworks)]
pub fn trailers(tally: &Tally, phase: Phase, agents: &[String], reworks: u32) -> String {
    format!(
        "Lid-Rs-Phase: {}\nLid-Rs-Agent: {}\nLid-Rs-Tools: {} edits, {} observations, {} commands\nLid-Rs-Checks: {} post-edit, {} stop\nLid-Rs-Refusals: {} policy, {} stop\nLid-Rs-Reworks: {reworks}\n",
        super::policy::number_of(phase),
        agents.join(", "),
        tally.edits,
        tally.observations,
        tally.commands,
        tally.post_edit_checks,
        tally.stop_checks,
        tally.policy_refusals,
        tally.stop_refusals
    )
}

/// The counts a commit's trailer block carries, read back from the lines
/// `trailers` wrote — the durable record of the attempt a replacement
/// carries forward. A line that renderer could not have written, a count
/// that is not a number among them, is a failure naming that line and never
/// a zero: the replaced commit's work is not filed under counts nobody kept.
#[implements(spec::ATrailerLineTheRendererCouldNotHaveWrittenFailsNamingTheLine)]
pub fn from_trailers(body: &str) -> Result<Tally, String> {
    todo!("the counts the trailer block of {body:?} carries")
}

/// The tally the commit about to be made carries, over whether anything was
/// replaced: this agent's own when nothing was — and equally when the
/// replaced commit's agents already name `agent`, since a resumed worker
/// keeps its id and its tally already counted the rejected attempt —
/// otherwise the replaced commit's counts, read back from its trailers,
/// added count for count to this agent's, a fresh worker's tally having
/// started at zero. The two cases that answer this agent's tally are one arm
/// and not two, so the decision this holds is the one the claims name. The id
/// is a parameter because the decision is about *which* agent and the counts
/// carry none, as `agents` takes it for the same reason.
///
/// At this layer the answer is that arm, for every input: it is right for
/// every first attempt and for every resumed one, and wrong for a
/// replacement by a fresh agent, which is where its validation drives it.
#[implements(
    spec::AResumedAgentsCountsAreNotAddedToTheCommitThatAlreadyCoversThem,
    spec::AFreshAgentsCountsAreAddedToTheReplacedCommits,
)]
pub fn merged(replaced: Option<&Replaced>, this: &Tally, agent: &str) -> Tally {
    let _ = (replaced, agent);
    *this
}

/// The agents whose work the commit about to be made carries, in the order
/// they worked and none of them twice: the replaced commit's agents with
/// this one appended when it is not already among them, and this agent alone
/// when nothing was replaced — the `Lid-Rs-Agent` line `trailers` renders.
///
/// At this layer the answer is the first attempt's, which is wrong for a
/// replacement made by a second agent.
#[implements(spec::LidRsAgentNamesEveryAgentThatMadeTheCommitInOrderNoneTwice)]
pub fn agents(replaced: Option<&Replaced>, this: &str) -> Vec<String> {
    let _ = replaced;
    vec![this.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::fixture;
    use lid_rs::validates;

    #[test]
    #[validates(spec::EveryToolCallIsTallied)]
    fn every_event_counts_once() {
        let events = [
            Event::Tool(ToolKind::Edit),
            Event::Tool(ToolKind::Edit),
            Event::Tool(ToolKind::Observation),
            Event::Tool(ToolKind::Command),
            Event::PostEditCheck,
            Event::StopCheck,
            Event::PolicyRefusal,
            Event::StopRefusal,
        ];
        let tally = events.into_iter().fold(Tally::default(), apply);
        let expected = Tally { edits: 2, observations: 1, commands: 1, post_edit_checks: 1, stop_checks: 1, policy_refusals: 1, stop_refusals: 1 };
        assert_eq!(tally, expected);
    }

    #[test]
    #[validates(spec::EveryToolCallIsTallied)]
    fn the_tally_is_stored_under_the_agents_id() {
        let (_dir, project) = fixture::copy("tally-store");
        let agent = format!("store-{}", std::process::id());
        assert_eq!(load(&project, &agent).expect("empty"), Tally::default());
        record(&project, &agent, Event::Tool(ToolKind::Edit)).expect("record");
        record(&project, &agent, Event::StopRefusal).expect("record");
        assert_eq!(load(&project, &agent).expect("stored"), Tally { edits: 1, stop_refusals: 1, ..Tally::default() });
    }

    #[test]
    #[validates(spec::APhaseCommitEndsWithTheSixTrailersNamingEveryAgentAndTheReworks)]
    fn a_phase_commit_ends_with_the_six_trailers_naming_every_agent_and_the_reworks() {
        // Driven through a stop rather than asserted of the renderer: the
        // renderer is written whole and answers whatever it is handed, so only
        // a commit the hook actually made shows what it was handed. This one
        // replaces the attempt a first agent left, so its `Lid-Rs-Agent` names
        // both and its `Lid-Rs-Reworks` is one.
        let (dir, project) = fixture::copy("trailers-of-a-rework");
        fixture::commit_with_body(&dir, "phase 3: skeleton for hello", &fixture::trailer_block(3, "first", 0));
        std::fs::write(dir.join("src/hello.rs"), "//! The hello slice.\n\n/// Greets, warmly.\npub fn greet() -> &'static str {\n    \"hello there\"\n}\n")
            .expect("write");
        // Tool calls of this agent's own, as the pre-tool hook counts them, so
        // that the counts the trailers carry are a sum neither operand equals:
        // an agent with an empty tally would render the attempt's counts
        // unchanged and pin nothing.
        for event in [Event::Tool(ToolKind::Edit), Event::Tool(ToolKind::Observation), Event::Tool(ToolKind::Observation)] {
            record(&project, "second", event).expect("record");
        }
        let stop = fixture::stop_input("second", "```commit\nphase 3: skeleton for hello\n```\n");
        let verdict = crate::phase::hook_stop(&project, Phase::Three, &stop).expect("hook");
        assert_eq!(verdict, crate::phase::HookVerdict::Allow, "the stop commits the phase: {verdict:?}");
        let message = fixture::message(&dir);
        let block = message.split_once("\n\n").expect("a body after the subject").1;
        let keys: Vec<&str> = block.lines().map(|line| line.split_once(':').expect("a trailer line").0).collect();
        assert_eq!(
            keys,
            ["Lid-Rs-Phase", "Lid-Rs-Agent", "Lid-Rs-Tools", "Lid-Rs-Checks", "Lid-Rs-Refusals", "Lid-Rs-Reworks"],
            "the six, in order, ending the message: {message}"
        );
        assert!(block.contains("Lid-Rs-Phase: 3\nLid-Rs-Agent: first, second\n"), "every agent whose work the commit carries, separated by `, `: {message}");
        assert!(block.ends_with("Lid-Rs-Reworks: 1"), "the number of commits this one replaced: {message}");
        // And the counts rendered are the merged tally's: this fresh agent's
        // own, added to what the attempt's trailers carried.
        let counts = load(&project, "second").expect("this agent's tally");
        assert_eq!((counts.edits, counts.observations), (1, 2), "this agent's own calls, so the line below is a sum neither operand equals");
        let tools = format!(
            "Lid-Rs-Tools: {} edits, {} observations, {} commands\n",
            counts.edits + fixture::TIP_TALLY.edits,
            counts.observations + fixture::TIP_TALLY.observations,
            counts.commands + fixture::TIP_TALLY.commands
        );
        assert!(block.contains(&tools), "the tally the commit carries is both rounds': {message}");
    }

    #[test]
    #[validates(spec::AFreshAgentsCountsAreAddedToTheReplacedCommits, spec::AResumedAgentsCountsAreNotAddedToTheCommitThatAlreadyCoversThem)]
    fn a_fresh_agents_counts_are_added_to_the_replaced_commits() {
        // This agent is `second`, and the tally filed under that id is what
        // `merged` is handed beside the record and the id itself.
        let mine = Tally { edits: 3, observations: 4, commands: 0, post_edit_checks: 3, stop_checks: 1, policy_refusals: 0, stop_refusals: 0 };
        assert_eq!(merged(None, &mine, "second"), mine, "a first attempt has nothing to carry forward");
        // A resumed worker keeps its id, so the tally filed under it already
        // counts the rejected attempt: adding the replaced commit's counts
        // would count every call of the first round twice.
        let resumed = fixture::replaced("c0ffee", 3, "phase 3: skeleton for hello", &["second"], 0);
        assert_eq!(merged(Some(&resumed), &mine, "second"), mine, "a record whose agents already name this one takes none of its counts");
        // A fresh worker — what the unattended workflow spawns for a rework —
        // started at zero, and the replaced commit is the durable record of
        // the round before it, so the two are added count for count. The id is
        // the whole difference between the two cases: the records differ only
        // in the agents they name.
        let fresh = fixture::replaced("c0ffee", 3, "phase 3: skeleton for hello", &["first"], 0);
        let both = Tally { edits: 8, observations: 11, commands: 0, post_edit_checks: 8, stop_checks: 2, policy_refusals: 0, stop_refusals: 0 };
        assert_eq!(merged(Some(&fresh), &mine, "second"), both, "the replaced commit's counts added to this agent's");
    }

    #[test]
    #[validates(spec::LidRsAgentNamesEveryAgentThatMadeTheCommitInOrderNoneTwice)]
    fn lid_rs_agent_names_every_agent_that_made_the_commit_in_order_none_twice() {
        assert_eq!(agents(None, "solo"), ["solo"], "a first attempt is one agent's");
        let first_round = fixture::replaced("c0ffee", 3, "phase 3: skeleton for hello", &["first"], 0);
        assert_eq!(agents(Some(&first_round), "second"), ["first", "second"], "the replaced commit's agents, then this one, in the order they worked");
        assert_eq!(agents(Some(&first_round), "first"), ["first"], "a resumed worker is named once, not twice");
        let two_rounds = fixture::replaced("c0ffee", 3, "phase 3: skeleton for hello", &["first", "second"], 1);
        assert_eq!(agents(Some(&two_rounds), "third"), ["first", "second", "third"], "every agent whose work the commit carries");
    }

    #[test]
    #[validates(spec::ATrailerLineTheRendererCouldNotHaveWrittenFailsNamingTheLine)]
    fn a_trailer_line_the_renderer_could_not_have_written_fails_naming_the_line() {
        let block = fixture::trailer_block(3, "first", 0);
        assert_eq!(from_trailers(&block).expect("the counts the block carries"), fixture::TIP_TALLY, "read back from the lines the renderer wrote");
        assert_eq!(from_trailers(&format!("A phase's body.\n\n{block}")).expect("the counts"), fixture::TIP_TALLY, "whatever body precedes the block");
        for missing in ["Lid-Rs-Agent", "Lid-Rs-Tools", "Lid-Rs-Checks", "Lid-Rs-Refusals"] {
            let without: String = block.lines().filter(|line| !line.starts_with(missing)).map(|line| format!("{line}\n")).collect();
            let err = from_trailers(&without).expect_err("a line the renderer could not have left out");
            assert!(err.contains(missing), "names the line it could not read: {err}");
        }
        let not_a_number = block.replace("5 edits", "many edits");
        let err = from_trailers(&not_a_number).expect_err("a count that is not a number");
        assert!(err.contains("Lid-Rs-Tools"), "names the line, and never reads it as zero: {err}");
    }
}
