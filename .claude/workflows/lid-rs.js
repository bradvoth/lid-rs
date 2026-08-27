// Managed by `cargo lid-rs sync` from the `lid-rs` crate this project depends
// on. Do not edit. Builds one LID-rs slice unattended: phases 2–7 from a
// human-approved LLD, one edit-only phase agent and one read-only reviewer
// per phase; the agents' hooks run every check and make every commit
// (docs/intent/phase/lld.md).
export const meta = {
  name: 'lid-rs',
  description: 'Build one LID-rs slice unattended from its approved LLD: phases 2-7, an edit-only phase agent and a read-only reviewer per phase, every check and commit made by the agents\' hooks',
  whenToUse: 'An lld/<slice> branch carries an approved "phase 1:" commit and the human wants the slice built without stops; pass args: {slice: "<name>"}',
  phases: [
    { title: 'Precondition', detail: 'read the branch; the LLD must be committed, the tree clean' },
    { title: 'Phase 2', detail: 'derive claims' },
    { title: 'Phase 3', detail: 'layer-0 skeleton' },
    { title: 'Phase 4', detail: 'descend breadth-first' },
    { title: 'Phase 5', detail: 'failing-first validations, red by the stop hook' },
    { title: 'Phase 7', detail: 'implement leaves, then the gate' },
  ],
}

const slice = args && args.slice
if (!slice) throw new Error('args.slice is required: the slice whose lld/<slice> branch carries the approved "phase 1:" commit')
const branch = `lld/${slice}`
const SKILL = '.claude/skills/lid-rs'

const STATE = {
  type: 'object',
  required: ['branch_exists', 'phase1_committed', 'tree_clean', 'committed_phases', 'lld_path', 'compile_time', 'compile_time_accepted', 'log'],
  properties: {
    branch_exists: { type: 'boolean' },
    phase1_committed: { type: 'boolean' },
    tree_clean: { type: 'boolean' },
    committed_phases: { type: 'array', items: { type: 'integer' } },
    lld_path: { type: 'string' },
    compile_time: { type: 'boolean' },
    compile_time_accepted: { type: 'boolean' },
    log: { type: 'string' },
  },
}
const WORK = {
  type: 'object',
  required: ['committed', 'decisions'],
  properties: {
    committed: { type: 'boolean' },
    decisions: { type: 'array', items: { type: 'string' } },
  },
}
const REVIEW = {
  type: 'object',
  required: ['approved', 'findings', 'commit', 'files'],
  properties: {
    approved: { type: 'boolean' },
    findings: { type: 'array', items: { type: 'string' } },
    commit: { type: 'string' },
    files: { type: 'array', items: { type: 'string' } },
  },
}

// The phases a worker commits, in order. Phase 7's agent also does Phase 6,
// which has no commit of its own.
const PHASES = [
  { n: 2, title: 'Phase 2', tag: `phase 2: claims for ${slice}` },
  { n: 3, title: 'Phase 3', tag: `phase 3: skeleton for ${slice}` },
  { n: 4, title: 'Phase 4', tag: `phase 4: descend for ${slice}` },
  { n: 5, title: 'Phase 5', tag: `phase 5: failing tests (red) for ${slice}` },
  { n: 7, title: 'Phase 7', tag: `phase 7: <version>: <what and why>` },
]

const stopped = (at, decisions) => ({ outcome: 'stopped', branch, at, decisions })

function workerPrompt(p, state, findings) {
  const rework = findings.length
    ? `\nA reviewer rejected the previous attempt at this phase. Address every finding, then end with a commit block again:\n${findings.map((f, i) => `  ${i + 1}. ${f}`).join('\n')}\n`
    : ''
  return `Slice: "${slice}". Branch: ${branch} (already checked out). LLD: ${state.lld_path}.
Branch history (git log --oneline):
${state.log}

Do Phase ${p.n} as your definition and ${SKILL}/references/phase-${p.n}.md describe. Your commit block's subject is "${p.tag}".${rework}
Return JSON only when you are done — after your final message has ended the phase: committed (true if your commit block was accepted), decisions (the numbered decisions from your commit or stop block, at most three).`
}

function reviewerPrompt(p, state) {
  return `Review Phase ${p.n} of slice "${slice}" on branch ${branch}, from files alone.
Read ${SKILL}/references/phase-${p.n}.md and the rows of ${SKILL}/references/discipline.md tagged ${p.n}; the LLD at ${state.lld_path}; then the newest commit on the branch (read .git/HEAD, the ref it names, and the files that commit touched — the worker's commit message names the phase and its body says what changed).
Try to refute the artifact against that phase's checklist and against the LLD: could a reader derive it from the LLD alone? A decision that changes the LLD is not yours to approve: reject with a finding that says so.
Return JSON only: approved (true only if you would say "continue" at an interactive stop), findings (at most five, concrete and actionable; empty when approved), commit (the hash you reviewed), files (the paths it touched).`
}

phase('Precondition')
const state = await agent(
  `Read-only. Report the state of branch ${branch} in this repository for slice "${slice}", from files alone (no shell):
- branch_exists: does .git/refs/heads/${branch} (or the packed ref) exist, and is it the checked-out branch (.git/HEAD)?
- phase1_committed: does the branch's history contain a commit whose subject starts "phase 1:"? committed_phases: the phase numbers N with a commit whose subject starts "phase N:".
- tree_clean: as far as files show, is there uncommitted work? (Report false if unsure.)
- lld_path: docs/intent/${slice}/lld.md under the workspace package that holds it.
- compile_time: does that package's Cargo.toml declare proc-macro = true, or does build.rs exist beside it? compile_time_accepted: does docs/intent/${slice}/compile-time-accepted exist there?
- log: the branch's history as "git log --oneline" would print it, newest first, from the refs and commit objects you can read.
Change nothing. Return JSON only.`,
  { label: 'precondition', agentType: 'lid-rs-review', schema: STATE },
)
if (!state) return stopped('precondition', ['the precondition agent returned nothing; rerun'])
if (!state.branch_exists) return stopped('precondition', [`branch ${branch} is not checked out; Phase 0 and Phase 1 are the human's: create it, commit the LLD as "phase 1: LLD for ${slice}", and check it out`])
if (!state.phase1_committed) return stopped('precondition', [`branch ${branch} has no "phase 1:" commit; Phase 1 is human-owned, so approve and commit the LLD first`])
if (!state.tree_clean) return stopped('precondition', ['the working tree is not clean; a previous phase ended without committing — inspect it, then commit or discard it by hand'])
if (state.compile_time && !state.compile_time_accepted) return stopped('precondition', [`"${slice}" is a compile-time slice: editing it executes the agent's code after every edit. To run it unattended, commit docs/intent/${slice}/compile-time-accepted with the LLD; the hooks refuse edits until then`])
if (state.compile_time) log(`"${slice}" is a compile-time slice; the human's acceptance file is present`)

const decisions = []
for (const p of PHASES) {
  if (state.committed_phases.includes(p.n)) { log(`${p.title} is already committed on ${branch}; skipping`); continue }
  phase(p.title)
  let findings = []
  let approved = false
  for (let attempt = 0; attempt < 2 && !approved; attempt++) {
    let work
    try {
      work = await agent(workerPrompt(p, state, findings), { label: `phase-${p.n}-worker`, agentType: `lid-rs-phase-${p.n}`, schema: WORK })
    } catch (e) {
      return stopped(p.title, [`the lid-rs-phase-${p.n} agent could not be spawned (${e && e.message ? e.message : e}); run \`cargo lid-rs sync\` and start a new session so .claude/agents/ is loaded`])
    }
    if (!work) return stopped(p.title, ['the worker returned nothing; rerun'])
    for (const d of work.decisions) decisions.push(`${p.title}: ${d}`)
    if (!work.committed) return stopped(p.title, work.decisions.length ? work.decisions : ['the worker ended without a commit and gave no decisions'])
    const review = await agent(reviewerPrompt(p, state), { label: `phase-${p.n}-review`, agentType: 'lid-rs-review', schema: REVIEW })
    if (!review || review.approved) { approved = true; break }
    findings = review.findings
    log(`${p.title}: the reviewer returned ${findings.length} finding(s); one rework round`)
  }
  if (!approved) return stopped(p.title, findings)
}
return { outcome: 'pr-ready', branch, decisions }
