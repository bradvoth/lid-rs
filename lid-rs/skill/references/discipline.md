# Where discipline slips

The gates catch structure. They cannot catch a phase quietly merged into the
next, a helper dropped into the nearest file, or a claim borrowed from
another slice — each legal, each compounding into a module nobody can
trace. These are the moments it happens; at each, run the check. Each row
names the phase(s) it applies to — read this file at those phases' stops.

| Phase(s) | When | Do this |
|---|---|---|
| 6, 7 | A new type, function, or decision is about to appear while implementing | Stop: it is a Phase 8 event. Edit the LLD, derive or rename the claim, then write the code. At Phase 7, confirm every backticked identifier in the LLD's shape table resolves in `src/`. |
| 2, 3 | Reusing an earlier slice's claim looks economical | Don't. A slice cites its own claims. A method this slice adds to an earlier slice's type is this slice's code, in this slice's module — otherwise the tests that kill its mutants sit in another slice's plan and the mutants survive. |
| 6 | The natural file for a helper is an older module | Put it in this slice's module: untraced helpers belong to the slice that adds them. Report each module's untraced-function count at Phase 7; a rising count in an old module is a fallback bucket forming. |
| 3, 6 | A library type — enum, struct, error, event stream — is about to be used inside a leaf | Keep it at the boundary. One function translates it into domain data; interior functions take and return domain types; rendering is a second boundary. Classify strings and magic numbers the design branches on once, into an enum or a named constant. Use an enum for a closed set of decisions (its variants are claims) and a trait for a capability. Count the match sites before saying "one". |
| 6 | About to call a leaf done | An arm with two statements, or an `if` followed by more statements, is dispatch and work in one function: split it, or write the claim the branch implements. The complexity threshold bounds line counts, not decisions. |
| 2 | Writing a claim | One *when*, one *shall*; no parenthetical alternatives; both halves name something a test can construct or observe; an implementer for it exists in the shape table. If the slice's central mechanism has no claim, its tests will attach themselves to the wrong ones. |
| 4, 5 | The slice is large and the red run feels like a formality; any fork directive | Run it as its own step and paste the output. A body written in Phase 4 has lost its red run, and the commit must say so. Size is a reason to fork the red run, never to skip it. |
| 7 | A gate result looks like a tool bug | Write two hypotheses and name the observation that separates them before concluding; evidence consistent with both is not proof. For check 12, run the narrowing test in `gates.md`. |
