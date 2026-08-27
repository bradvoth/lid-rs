//! The per-agent tally (`docs/intent/phase/lld.md` § the tally): every tool
//! call, check, and refusal, counted under the agent's id and written into
//! the phase commit as trailers.

use std::path::PathBuf;

use lid_rs::implements;

use super::Phase;
use super::policy::ToolKind;
use crate::project::Project;
use crate::spec;

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

/// The `Lid-Rs-*` trailers for a phase commit.
#[implements(spec::TheTallyIsWrittenAsTrailers)]
pub fn trailers(tally: &Tally, phase: Phase) -> String {
    format!(
        "Lid-Rs-Phase: {}\nLid-Rs-Tools: {} edits, {} observations, {} commands\nLid-Rs-Checks: {} post-edit, {} stop\nLid-Rs-Refusals: {} policy, {} stop\n",
        super::policy::number_of(phase),
        tally.edits,
        tally.observations,
        tally.commands,
        tally.post_edit_checks,
        tally.stop_checks,
        tally.policy_refusals,
        tally.stop_refusals
    )
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
    #[validates(spec::TheTallyIsWrittenAsTrailers)]
    fn the_tally_is_written_as_trailers() {
        let tally = Tally { edits: 14, observations: 9, commands: 0, post_edit_checks: 14, stop_checks: 1, policy_refusals: 1, stop_refusals: 0 };
        assert_eq!(
            trailers(&tally, Phase::Seven),
            "Lid-Rs-Phase: 7\nLid-Rs-Tools: 14 edits, 9 observations, 0 commands\nLid-Rs-Checks: 14 post-edit, 1 stop\nLid-Rs-Refusals: 1 policy, 0 stop\n"
        );
    }
}
