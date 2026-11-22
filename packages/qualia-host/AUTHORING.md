# Implemented durable source-authoring path

`NativeAuthoringBridge` binds an actual `Arc<NativeAuthority>` and its opaque
`SessionHandle`. Construction revalidates that native session. It exposes the
existing canonical `WorkflowEvidence` through `NativeAuthoringView`, and the view
can call the canonical `request_source_review` operation. This is a persisted
source/evidence operation; it is not a semantic graph query/update grant, a review
acceptance decision, or a VibeScript extension receipt.

Create a view on the native owner thread with an explicit snapshot representation
limit. Call `refresh` on the application's bounded tick schedule, then
`take_current` to update view state. Only one pending canonical snapshot is retained;
repeated refreshes coalesce. Each delivery uses the actual authority's exact
source/generation/session publication check. Close the view on navigation or
component teardown. No global graph revision stream is exposed as scoped authority.

`request_source_review(id, evidence_ref)` returns the actual canonical evidence
record only after the service's journal append and file-sync acknowledgement. The
method clears an older pending view before mutation. Refresh is separate so a
refresh error cannot hide an acknowledged write. Canonical errors propagate,
including journal exhaustion and reconciliation. Reopened services require a new
key-possession session proof; old opaque session handles cannot be reused.

The native authority retains its own journal/state bounds. The adapter's Debug
representation bound is checked after that bounded canonical snapshot is produced;
it is not an allocator quota. The canonical publication check uses `try_lock`, but
the subsequent `workflow_evidence` call may block if another transaction obtains
the owner lock. These are bounded count/representation semantics, not a guaranteed
nonblocking or real-time UI tick. A busy publication check clears pending data and
allows a later refresh retry. The canonical portable journal syncs file contents;
Windows directory-entry survival under machine power loss remains unqualified.

The bridge never creates or signs admissions in production code. Tests use
explicit signed software-instrument fixtures and actual native challenge/signature
verification, then write/reopen real temporary journals. These fixtures do not
claim human enrollment or production trust. See [verification](VERIFICATION.md)
for the executed test results and build-provenance limits.

The [remaining graph contracts](GRAPH-CONTRACT-GAPS.md) are separate from this
supported source/evidence workflow. No arbitrary graph access or extension-receipt
persistence is inferred from successful source-authoring admission.
