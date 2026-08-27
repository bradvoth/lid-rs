//! Claims for `sync` (`docs/intent/sync/lld.md`).

use lid_rs::Spec;

/// When `sync` runs, every file under the skill shall be read from the
/// `lid-rs` package the project's manifest resolves — registry or path —
/// never from the tool's own build.
#[derive(Spec)]
pub struct TheSkillComesFromTheResolvedLidRsDependency;

/// When `sync` writes the skill, it shall write every file under it to
/// `<workspace_root>/.claude/skills/lid-rs/`, mirroring the dependency's
/// directory structure and creating directories as needed, and running it
/// again shall change nothing.
#[derive(Spec)]
pub struct TheSkillCopyLivesAtTheWorkspaceRoot;

/// When `sync --check` runs and the project's copy is missing a file, has an
/// extra file, or differs in a file's content from the dependency's skill,
/// it shall fail naming every such file and write nothing; when identical it
/// shall succeed; any other flag shall be rejected by name.
#[derive(Spec)]
pub struct SyncCheckFailsOnAnyDifferenceAndWritesNothing;

/// When the project resolves no `lid-rs`, or a `lid-rs` that ships no skill
/// directory, `sync` shall fail naming which.
#[derive(Spec)]
pub struct AMissingSkillSourceFailsByName;
