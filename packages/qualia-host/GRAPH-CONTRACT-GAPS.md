# Canonical graph boundary findings

## Local continuation, 12 November 2026

The initial findings below describe the saved baseline. The local canonical
NativeAuthority now has an independently signed, exact-context project graph
contract, journalled writes and bounded snapshots/subscriptions. Aether binds that
contract through NativeGraphBridge, native_graph_runtime and a component-owned
use_qualia_graph task. See [resumed implementation](../../notes/implementation/resume-host.md)
for source ownership, verification and remaining gates. These graphs are owned by
the native authority journal; they do not grant access to the global daemon graph
or establish arbitrary scoped SPARQL, generic extension receipt persistence,
cross-store transactions or native platform readiness.

## Saved baseline

Inspected 2026-11-12 against QualiaDB local revision
`afbe431195167fde0fadc233ed79fd2fe85f73f0`. The implemented native source-authoring
bridge uses actual signed admission, native proof sessions and journalled records.
Those rights do not imply graph access. This file identifies the concrete missing
graph integration contract rather than substituting a permissive local policy.

| Canonical source | Available behavior | Missing contract for this native profile |
| --- | --- | --- |
| `crates/qualia-core-db/src/poet_host/scoped_effects/host.rs` | Real scoped host for pulse/HID/key effects | Explicitly rejects `graph_query`, `graph_stage` and `graph_commit`; no scoped graph methods are exported by this host. |
| `crates/qualia-core-db/src/poet_host/mod.rs` | `PoetSnapshot` reads the daemon graph, stages quins and commits them; `graph_abort` clears staging | Raw graph methods do not take native actor/context leases or invoke the extension Sentinel. `graph_begin` is a no-op and `graph_commit` returns `Value::Receipt`; this is not proof of a durable, authorized transaction. |
| `crates/qualia-core-db/src/services/daemon_graph.rs` | Bounded global in-process store, update APIs, revision counter and 64-slot broadcast | `subscribe_graph_revisions` is global and carries no per-subject/context scope. Mutation functions are storage primitives, not native authenticated admission APIs. |
| `crates/qualia-core-db/src/services/daemon_query.rs` | SPARQL/bytecode query over supplied NQuins, with classified-egress checks | Accepts graph data rather than an opaque native subject/context lease. It cannot authenticate caller-supplied graph scope. |
| `crates/qualia-core-db/src/poet_host/native_authority` and `crates/qualia-cooperative-core/src/authoring/types.rs` | Signed project adoption, key-possession sessions, exact source read/edit/review authority and durable document/evidence journal | `AuthoringAction` has no graph read/write/subscription resource scope. `transact` authorizes source access and tracks source/evidence successors; it must not be reused as blanket graph permission. |
| `crates/qualia-core-db/src/governance/cohort_access_gate.rs` | Canonical context/sensitivity filtering of NQuins using `PrincipalAndAgentGrant` | Its public grant data and active-field checks do not establish issuer authenticity or bind a native session to a graph resource. A legitimate native owner must supply that missing linkage; nonzero DID hashes are not proof. |
| `crates/qualia-client-core/src/qapp_api.rs` | Queries bounded data under registered app-ID capability claims, producing a Flutter geometry projection | Not a native subject/lease-bound graph transaction/subscription API. Its six `f32` geometry lanes are not a lossless canonical NQuin data contract. |
| `crates/qualia-core-db/src/poet_host/vibe_sentinel/policy_gate.rs` | Actual deontic extension gate and bounded canonical receipt slice | `commit_receipt` writes two NQuins to caller RAM, without a durable journal acknowledgement. The source-authority journal has no public generic extension-receipt append operation. |

The canonical owners need to expose or extend their existing interfaces to provide:

1. A native-owner-issued graph operation scope linking authenticated session,
   current policy generation, actor/delegation, resource/context and expiry. Reuse
   canonical authority and ABI types; do not treat source `requires`, app manifests,
   raw hashes or constructible grant records as authentication.
2. Bounded canonical query/update methods under that scope, preserving all NQuin
   lanes and provenance, with explicit staging and durable receipt acknowledgement
   before success/publication. Revocation and cancellation need transaction-owner
   semantics; a failed generic provider receipt cannot undo prior side effects.
3. A bounded change subscription derived from the same scope, with source/context
   epochs, revocation, lag reporting, teardown and snapshot reconciliation. A global
   revision number can trigger a fresh authorized read, but does not authorize one.

Until those canonical seams exist and are tested, `scoped_native_runtime` keeps
graph methods denied. `NativeAuthoringBridge` separately delivers the legitimate
signed/admitted, persisted source-review workflow. It does not synthesize graph
norms or write arbitrary quins into either authority journal.
