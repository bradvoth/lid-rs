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
    todo!()
}

/// The agent's tally so far; empty for an agent with none.
pub fn load(project: &Project, agent_id: &str) -> Result<Tally, String> {
    todo!()
}

/// Counts one event under the agent's id.
#[implements(spec::EveryToolCallIsTallied)]
pub fn record(project: &Project, agent_id: &str, event: Event) -> Result<(), String> {
    todo!()
}

/// One event applied to a tally.
#[implements(spec::EveryToolCallIsTallied)]
pub fn apply(tally: Tally, event: Event) -> Tally {
    todo!()
}

/// The `Lid-Rs-*` trailers for a phase commit.
#[implements(spec::TheTallyIsWrittenAsTrailers)]
pub fn trailers(tally: &Tally, phase: Phase) -> String {
    todo!()
}
