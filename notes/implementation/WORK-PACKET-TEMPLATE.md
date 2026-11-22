# Work packet assignment and result template

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

Copy the sections into a new packet receipt when dispatching a bounded slice of
[the continuation plan](WORK-PACKETS-2026-11-15.md). This template has no active
assignment and does not authorize builds or sibling edits by itself.

## Assignment

- **Packet / slice ID:** e.g. WP09.a; one concrete outcome.
- **State:** proposed / waiting-upstream / ready / active / review / accepted /
  failed / blocked / superseded. “Blocked” in this ledger means a named missing
  prerequisite; it is not an instruction to change a tool-managed goal status.
- **Role and competence:** REC / RUST / BOUNDARY / GPU / MEDIA / REVIEW.
- **Actual agent selection:** model and reasoning level, rationale for cost;
  escalation trigger. Do not invent prices or set unrequested budget controls.
- **Prerequisites:** accepted intake revision, prerequisite packet results, exact
  source/lock/profile identifiers. If not satisfied, limit work to named preparation.
- **Read first:** applicable `AGENTS.md`, canonical contracts, prior evidence.
- **Exact write ownership:** allowed files/new paths. Shared manifest/module edits
  go through SUP. List sibling files as read-only unless explicitly reassigned.
- **Excluded work:** adjacent owner's files, unrelated cleanup, feature expansion.
- **Contract:** inputs/outputs, authority, ownership, buffers/limits, deadlines,
  cancellation/retry/terminal states, refusal and incomplete-result semantics.
- **Deliverables:** implementation/specification, meaningful tests, receipt paths,
  migration notes and explicit remaining limits.
- **Acceptance:** observable assertions; positive and negative cases; evidence
  class required. Compilation alone only satisfies a compilation assignment.
- **Build slot:** owner, exact Cargo target/cache/profile/toolchain, jobs, planned
  command and outer deadline; no competing same-target process.
- **Review:** independent role, files and invariants to inspect.
- **Return condition:** finish this bounded slice or report concrete blocker with
  preserved logs; do not continue into an unassigned subsystem.

## Execution record

| Check / command | Exact sources and profile | Exit / result | Evidence path | What it proves / does not prove |
| --- | --- | --- | --- | --- |
| Not run | Not recorded | Pending | None | No claim |

Record source drift before/after execution, runtime artifact hashes, OS/device
identity where relevant, and failures/skips. Do not overwrite earlier receipts.
Record active owned process/session if execution is handed off mid-run.

## Result and acceptance

- Concrete implemented behavior and why it changed.
- Test results and new findings; no pass inferred from a partial log.
- Review findings with source references, owning correction and recheck result.
- Remaining upstream actions and register update; no unsolicited external messages.
- Exact files changed; existing unrelated work preserved.
- Owned process state and target/cache available for the next owner.
- SUP acceptance, qualified profile and downstream packet IDs released.
- Any stale or superseded evidence retained and clearly identified.

All assignments retain the user's no-commit/push/branch-change and provenance
constraints. A completed packet is not a production-readiness declaration.
