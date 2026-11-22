# Independent Aether preparation — 15 November 2026

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

## Authorization and boundary

The user authorized the independent preparation packets while concurrent sibling
work continues, with all uncommitted work preserved. This record implements the
preparation portions of [the work packets](../WORK-PACKETS-2026-11-15.md).
It does not release any sibling-dependent implementation or qualification.

No sibling checkout, source, manifest or lockfile is changed or consulted for new
API decisions. No Cargo command, dependency resolution, integration test, worker
launch, GUI/audio/GPU probe or network call is part of this preparation. Existing
Aether-local receipts and immutable captured files may be read and hashed; their
historical source paths are provenance, not instructions to open sibling files.

## Assignments and cost allocation

All three delegated tasks use **gpt-6.1-sol, medium reasoning**. No Astra subagent
was assigned. These are bounded source inventories and acceptance specifications;
they do not require a new high-cost architecture pass. The supervisor integrates
and reviews their results. Escalation requires a specific unresolved question,
not merely a large repository or many files.

| Slice | Role / exact write ownership | Deliverable | Acceptance boundary |
| --- | --- | --- | --- |
| WP00.prep | SUP; this index and appended tracker status | Ownership, hold conditions, evidence review | No dependent packet released |
| WP01.a | SUP/REC; new audit receipts under `target/production-audit/aether-preparation-20261115T065910Z` | Dirty-path preservation baseline and final comparison; frozen capture verification | Read-only source checks, no qualification build |
| WP01.b / WP06.prep | REC; `ARTIFACTS-AND-NOTICES.md` only | Existing notices/artifacts, missing inputs and exact held staging/probe recipes | Authoritative evidence identified; incomplete stage stays incomplete |
| WP08.prep / WP16.prep | RUST/REC; `NATIVE-AND-DISTRIBUTION-MATRIX.md` only | Platform fixture and installed-artifact specifications | Assertions, owner/platform, prerequisites and missing fixtures explicit |
| WP04.prep / WP09.prep / WP14.prep / WP15.prep | RUST; `HOST-MEDIA-AND-COMPATIBILITY-ACCEPTANCE.md` only | Host/media/compatibility scenarios and measurement requirements | Requirements only; no frozen sibling API assumptions or new ABI |

Agents have no shared write paths. Existing implementation, dependency files,
licenses, scripts and historical evidence remain outside their write scopes.
No build or device slot is allocated; all execution recipes are **HELD**.

After the user clarified the licensing policy, the supervisor made one bounded
Aether-only correction: added missing `LICENSE-QUALIA` as an exact byte copy of
the existing `LICENSE`. The existing file and terms were not changed. This does
not execute or release staging.

## Prepared deliverables

- [Artifacts, notices and held runtime recipes](ARTIFACTS-AND-NOTICES.md).
- [Native and distribution fixture matrix](NATIVE-AND-DISTRIBUTION-MATRIX.md).
- [Host, media and compatibility acceptance](HOST-MEDIA-AND-COMPATIBILITY-ACCEPTANCE.md).

These specifications must be reconciled with accepted upstream commits before
execution. A complete-entry format path, archival resource or snapshot interface
must not be silently relabeled as progressive delivery, playback or graph deltas.

## Current license and future transition

The user confirmed that the existing shared license is the current temporary
policy, pending future computational support for the requirements associated with
[mediaprophet/license](https://github.com/mediaprophet/license). That future work
is recorded as a separate transition, not implemented or interpreted here.

Both root notice files now contain the same 1,940 bytes, SHA-256
`a5ce8e4d8c923a725ea3d1ff2743bc18e1997eac544465acfead9a9b4a9486e6`.
The local receipt is
`C:/github/Aether/target/production-audit/aether-preparation-20261115T065910Z/license-alias-receipt.json`.
This resolves the missing root filename, not complete transitive distribution
notices or the old incomplete stage. Future license migration must update both
copies consistently and preserve applicable upstream attribution.

## Preservation evidence

The initial baseline records **537 Git-reported dirty/untracked nonignored paths**.
Every existing regular file in that set was SHA-256 hashed with size/mtime checked
before and after reading; none changed during its read. Deleted paths and symlinks
are recorded separately. This is an inventory, not a backup or a claim to hash
every ignored build artifact.

Baseline receipt:
`C:/github/Aether/target/production-audit/aether-preparation-20261115T065910Z/preservation-before.json`.
SHA-256: `758f83073fc12920c754834d5a3c2b3e39876338cf7830ba5c9ce5e05b85ec6d`.

The supervisor compares all baseline entries again at completion. Only appended
preparation status in the named planning/tracker documents is an intended change
to pre-existing paths; agent deliverables and receipts are new files. Unexpected
drift must be reported, never reset or attributed to this task without evidence.

The frozen-v4 verification record belongs at
`C:/github/Aether/target/production-audit/aether-preparation-20261115T065910Z/frozen-v4-verification.json`.
Its scope is the existing Aether-local capture, not live canonical dependencies.
The verification passed: 4,382 inputs and nine package identities, unchanged
receipt SHA-256 `3bcee7dc35c4882ac02053f7efdf5f45de8024d5aea08a3d4cbf581d175c66b7`.
No captured bytes were modified. This is source-preservation evidence, not a
canonical dependency or native runtime test pass.

## Hold and restart conditions

WP02 remains waiting for the user's confirmation that the relevant sibling fixes
are ready and committed. No accepted upstream intake exists. Preparation does
not change the state of any of the 51 remaining implementation/qualification jobs.

After notification: fill the [upstream register](../UPSTREAM-INTEGRATION-REGISTER.md),
recheck the six review items against delivered commits, agree on a coherent
profile and build owner, and only then release the appropriate execution slice.
Do not rerun old heavyweight commands or use the partial stage merely because
this preparation contains their recipes.

## Completion review

The supervisor reviewed the three deliverables for bounded scope, actual local
fixture anchors, observable acceptance criteria and explicit held execution.
WP01 preservation/artifact preparation and the listed fixture/requirements
preparation slices are complete. No implementation or release job is marked done.

Final preservation and document-validation receipts are recorded in the same
audit directory as `preservation-after.json` and `document-validation.json`.
They distinguish intended append-only tracker updates from unexpected drift.
No owned build, worker or device probe was started. All sibling-dependent packets
remain waiting for user confirmation and accepted intake.
