// Managed by `cargo lid-rs sync` from the `lid-rs` crate this project depends
// on. Do not edit. Builds one LID-rs slice unattended: phases 2–7 from a
// human-approved LLD, a clean lid-rs-phase worker and a clean reviewer per
// phase, every commit gated by the phase hook (docs/intent/phase/lld.md).
export const meta = {
  name: 'lid-rs',
  description: 'Build one LID-rs slice unattended from its approved LLD: phases 2-7, one clean worker and reviewer per phase, every commit gated by the phase hook',
  whenToUse: 'An lld/<slice> branch carries an approved "phase 1:" commit and the human wants the slice built without stops; pass args: {slice: "<name>"}',
  phases: [
    { title: 'Precondition', detail: 'read the branch; the LLD must be committed' },
    { title: 'Phase 2', detail: 'derive claims' },
    { title: 'Phase 3', detail: 'layer-0 skeleton' },
    { title: 'Phase 4', detail: 'descend breadth-first' },
    { title: 'Phase 5', detail: 'failing-first validations, confirmed red by the hook' },
    { title: 'Phase 7', detail: 'implement leaves, then the gate' },
  ],
}

const slice = args && args.slice
if (!slice) throw new Error('args.slice is required: the slice whose lld/<slice> branch carries the approved "phase 1:" commit')
const branch = `lld/${slice}`
const SKILL = '.claude/skills/lid-rs'

const STATE = {
  type: 'object',
  required: ['branch_exists', 'phase1_committed', 'committed_phases', 'lld_path'],
  properties: {
    branch_exists: { type: 'boolean' },
    phase1_committed: { type: 'boolean' },
    committed_phases: { type: 'array', items: { type: 'integer' } },
    lld_path: { type: 'string' },
  },
}
const WORK = {
  type: 'object',
  required: ['committed', 'commit', 'decisions'],
  properties: {
    committed: { type: 'boolean' },
    commit: { type: 'string' },
    decisions: { type: 'array', items: { type: 'string' } },
    blocked: { type: 'string' },
  },
}
const REVIEW = {
  type: 'object',
  required: ['approved', 'findings'],
  properties: { approved: { type: 'boolean' }, findings: { type: 'array', items: { type: 'string' } } },
}

// The phases a worker commits, in order. Phase 7's worker also does Phase 6,
// which has no commit of its own.
const PHASES = [
  { n: 2, title: 'Phase 2', files: ['phase-2.md'], commit: `phase 2: claims for ${slice}` },
  { n: 3, title: 'Phase 3', files: ['phase-3.md'], commit: `phase 3: skeleton for ${slice}` },
  { n: 4, title: 'Phase 4', files: ['phase-4.md'], commit: `phase 4: descend for ${slice}` },
  { n: 5, title: 'Phase 5', files: ['phase-5.md'], commit: `phase 5: failing tests (red) for ${slice}` },
  { n: 7, title: 'Phase 7', files: ['phase-6.md', 'phase-7.md'], commit: `phase 7: <version>: <what and why> (${slice})` },
]

const stopped = (at, decisions) => ({ outcome: 'stopped', branch, at, decisions })

function workerPrompt(p, state, findings) {
  const refs = p.files.map(f => `${SKILL}/references/${f}`).join(' and ')
  const rework = findings.length
    ? `\nA reviewer rejected the previous attempt at this phase. Address every finding below, then commit again under the same tag:\n${findings.map((f, i) => `  ${i + 1}. ${f}`).join('\n')}\n`
    : ''
  return `You are the lid-rs-phase worker for Phase ${p.n} of slice "${slice}", unattended, on branch ${branch} (check out that branch first; do not create another).
Read, in this order: ${SKILL}/SKILL.md; ${refs}; the rows of ${SKILL}/references/discipline.md tagged ${p.n}${p.n === 7 ? ' or 6' : ''}; the slice's LLD at ${state.lld_path}; and \`git log --oneline\` on the branch. Nothing else about this slice exists outside those files and the branch.
Do Phase ${p.n} only${p.n === 7 ? ' (Phase 6, implementing the leaves, has no commit of its own and is part of this work)' : ''}. Commit as "${p.commit}". The commit-msg hook runs this phase's check and refuses the commit if it fails — fix the cause and commit again; never --no-verify.${rework}
If completing the phase needs a decision that is the human's (an LLD change, a cascade into another slice's LLD, #[mutants::skip], a gate you cannot fix honestly), do not commit: return committed=false with those decisions.
Return JSON only: committed (boolean), commit (the sha, or "" if none), decisions (at most three numbered decisions a reviewer must make — what the design traded away is one of them), blocked (why, when not committed).`
}

function reviewerPrompt(p, state, work) {
  const refs = p.files.map(f => `${SKILL}/references/${f}`).join(' and ')
  return `You are the reviewer at the Phase ${p.n} stop for slice "${slice}" on branch ${branch}. You did not write it and have no conversation about it: judge it from files alone, as the methodology's stop requires.
Read ${refs} and the rows of ${SKILL}/references/discipline.md tagged ${p.n}; the LLD at ${state.lld_path}; and \`git show ${work.commit}\` (plus \`git diff\` against the previous phase commit if useful).
Try to refute the artifact against that phase's checklist — the per-claim list at Phase 2, signatures and the boundary rule at Phase 3, the dispatch/work rule and gate results at Phase 7 — and against the LLD: could a reader derive this artifact from the LLD alone? If the previous phase's artifact was insufficient for you to judge without the conversation that produced it, that is itself a finding ("not context-free").
The worker's decisions for this stop: ${JSON.stringify(work.decisions)}. A decision that changes the LLD is not yours to approve: reject with a finding that says so.
Return JSON only: approved (true only if you would say "continue" at an interactive stop), findings (at most five, each concrete and actionable; empty when approved).`
}

phase('Precondition')
const state = await agent(
  `Read-only. Report the state of branch ${branch} in this repository: whether it exists; whether its log (\`git log --oneline ${branch}\`) contains a commit whose subject starts "phase 1:"; the list of phase numbers N for which a commit subject starting "phase N:" exists; and the path of the slice's LLD (docs/intent/${slice}/lld.md under the crate or workspace that the phase 1 commit touched — find it with \`git show --stat\`). Change nothing. Return JSON only.`,
  { label: 'precondition', schema: STATE },
)
if (!state) return stopped('precondition', ['the precondition agent returned nothing; rerun'])
if (!state.branch_exists) return stopped('precondition', [`branch ${branch} does not exist; Phase 0 and Phase 1 are the human's: create it and commit the LLD as "phase 1: LLD for ${slice}"`])
if (!state.phase1_committed) return stopped('precondition', [`branch ${branch} has no "phase 1:" commit; Phase 1 is human-owned, so approve and commit the LLD first`])

const decisions = []
for (const p of PHASES) {
  if (state.committed_phases.includes(p.n)) { log(`${p.title} is already committed on ${branch}; skipping`); continue }
  phase(p.title)
  let findings = []
  let approved = false
  for (let attempt = 0; attempt < 2 && !approved; attempt++) {
    let work
    try {
      work = await agent(workerPrompt(p, state, findings), { label: `phase-${p.n}-worker`, agentType: 'lid-rs-phase', schema: WORK })
    } catch (e) {
      return stopped(p.title, [`the lid-rs-phase agent could not be spawned (${e && e.message ? e.message : e}); run \`cargo lid-rs sync\` and start a new session so .claude/agents/lid-rs-phase.md is loaded`])
    }
    if (!work) return stopped(p.title, ['the worker returned nothing; rerun'])
    for (const d of work.decisions) decisions.push(`${p.title}: ${d}`)
    if (!work.committed) return stopped(p.title, work.decisions.length ? work.decisions : [work.blocked || 'the worker ended without committing and gave no reason'])
    const review = await agent(reviewerPrompt(p, state, work), { label: `phase-${p.n}-review`, schema: REVIEW })
    if (!review || review.approved) { approved = true; break }
    findings = review.findings
    log(`${p.title}: the reviewer returned ${findings.length} finding(s); one rework round`)
  }
  if (!approved) return stopped(p.title, findings)
}
return { outcome: 'pr-ready', branch, decisions }
