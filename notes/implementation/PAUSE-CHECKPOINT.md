# Aether local pause checkpoint — 13 November 2026

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

Paused at the user's request to charge the battery. Resume the full local overhaul,
with reviewed swarm work and the live tracker; do not restart from the original
failed-session baseline. All edits are local and uncommitted. Preserve unrelated
user changes in every sibling repository. Etalon is the implementation in its own
right; historical upstream references are provenance, not its current identity.

## Read first

- `../architecture/14-IMPLEMENTATION-TRACKER.md` and `13-NATIVE-PRODUCTION.md`.
- `resume-dependencies.md`, `resume-host.md`, `resume-chora.md`, `resume-native.md`
  and `REVIEW.md`. Their final pause entries carry profile-specific results.
- Local `AGENTS.md` in each repository before editing that repository.

## Saved implementation and evidence

- Aether and maintained Hypermedia imports, dependency aliases, module/type names,
  examples and current documentation now use Etalon. Standard WebGPU interfaces,
  original licenses/attribution, historical links and required narrow forwarding
  exports remain. Hypermedia asset-production-vibe edits were limited to its
  existing Etalon dependency alias, imports and naming; concurrent work is retained.
- Post-naming Windows metadata: 1,175 packages, no wgpu package identities; reviewed
  Etalon family remains at `575bab54accf1aafb24f67e9ba810c0f3f745aea`.
  Minimal hybrid and Vello consumer feature sets are unchanged. Source/snapshot
  policy tests pass 54/54; 17 wrapper configurations compile. Do not call the
  incomplete wrapper runtime matrix passed.
- Actual post-naming default, Vello+SVG, minimal CPU and combined default+CPU
  checks passed. See dependency packet for the final facade/smoke build status.
  The post-naming profiler check was stopped by Windows Application Control 4551
  before its shader build script executed. Earlier profiler/runtime passes apply
  to their recorded pre-naming builds only.
- Independent review compared all 55 Aether vendor Rust files to their source:
  intended naming, three swizzle defaults, boxed shader-error fix and compatibility
  exports only. The context helper root export and two historical comments were
  corrected. Root formatted 74 owned files and validated ten sibling manifests.
  Mechanical source hashes are in
  `target/production-audit/hypermedia-etalon-naming-final-receipt.json`; that is
  not a complete dependency snapshot or runtime qualification receipt.
- Actual native DOM 21 tests, surrounding-delete editor six tests and earlier
  clipboard four tests passed before naming. Earlier CPU single/multiwindow and
  default hybrid lifecycle runs passed; Vello launch remained OS-blocked.
- Frozen canonical v4 receipt:
  `target/qualification-sources/native-host-20261112-v4/source-receipt.json`, SHA256
  `3bcee7dc35c4882ac02053f7efdf5f45de8024d5aea08a3d4cbf581d175c66b7`.
  Its 4,382 files/nine identities are immutable. Do not rename captured sources.
  Current frozen host compilation stopped before core at getrandom 0.3.4's
  generated build script (Application Control 4551). Two approval reviews timed
  out before execution; no frozen host/core/Vibe/native fixture tests passed.
  Earlier 30 host tests belong to their earlier source checkpoint.
- Chora uses its separate Etalon `8e8ab6c8349bcc6ab1d81138f2c9f2a0c90f0952` and
  Hypermedia `af57e82120b30b47fca238edb21a839d2cdb4278` graph. Its attributed local
  three-package overlay passed the actual source gate with real probe metadata,
  124-file receipt and 15 policy tests. Root's additional normalized diff review
  did not complete: writing its comparison artifact failed on Windows text
  encoding. Do not count that comparison as a completed review.
- Actual Chora network library 8/8 and TLS integration 11/11 passed earlier.
  Actual config/embedder policy libraries 16/16 passed. These do not establish
  full engine runtime, Windows sandboxing or native production readiness.
- Latest full Chora build exposed a real cross-thread Etalon identity allocator
  issue: RefCell-backed managers inside shared IdentityHub were not Sync. Agent
  added safe per-manager mutex ownership, without unsafe Sync. Actual concurrency
  and engine rerun status must come from the final Chora pause packet.
- Crypto review removed page-selected host-key import interpretation. Ordinary
  raw key bytes, including marker-looking prefixes, must remain ordinary software
  key material. Existing HostKeystore operation and structured-clone paths deny
  unscoped authority. Do not reintroduce lexical prefix rejection. New tests are
  not passed until the packet records their actual execution.

## Next implementation: A4 separate Chora worker

In-process native + Chora linkage has real Stylo and ICU conflicts. Keep a separate
worker workspace/process; do not drop native text features or traditional web JS.
Process separation is not OS containment. Required sandbox requests must reject
unsupported profiles before browser/context creation. No worker has been qualified.

Assigned ownership at pause:

- canonical_host: standalone `packages/chora-bridge-protocol` draft and then the
  parent authorization/cancellation adapter in `packages/qualia-host`.
- dependencies: standalone `packages/chora-worker/Cargo.toml` and source gates,
  using actual sibling Chora package `servo` under dependency alias `chora`,
  embedded_library feature, existing 8e8/af57 graph; no native-root linkage.
- root: actual worker launcher/event loop/readback/native frame integration.
  No root worker source has yet been written.
- chora: generic engine fixes and actual engine/canvas/crypto/identity tests.

The protocol draft may contain exports for unwritten modules. Read the host
packet before treating it as buildable. It is not a completed transport or ABI.
No canonical parent adapter has been implemented at this pause checkpoint.

Agreed protocol: private parent-created pipes; parent session nonce binds an owned
worker but grants no authority; bounded versioned control frames and separate
binary RGBA frames (16MiB cap); exact route stamps (session/view/generation/document/
viewport), increasing sequences, at most 64 retained real requests and 30s deadlines.
The worker derives caller context only from actual Chora EmbedderMessageRequest.
Parent registers exact origin/profile routes backed by existing NativeGraphBridge
and signed canonical authority; only bounded read-only qualia.graph.snapshot is
initially intended. Unregistered routes/channels deny. No worker credentials or
inferred script/mutation authority.

Navigate advances parent generation and cancels old work before load. Actual
notify_url_changed advances worker document identity and emits invalidation.
Resize advances viewport and cancels all pending requests/captures, keeping exact
full-stamp matching. Per-request cancellation does not invalidate the whole view.
Late replies/frames never reach a new owner. One capture remains outstanding until
its frame is written; one latest-frame mailbox bounds UI ownership. Paint then
read before present; enforce checked dimensions and readback-copy budget before
calling Chora. Alpha/orientation, input and runtime remain qualification gates.

Actual APIs inspected: ChoraBuilder is the branded ServoBuilder alias;
try_build rejects unsupported sandbox/TLS configuration; WebViewBuilder and
SoftwareRenderingContext are reexported. Context uses dpi 0.1; URL uses url 2.5.
Event/request handles stay on the engine thread. Software context still requires
Surfman/ANGLE. Shutdown may spin in engine Drop, so parent deadline cleanup is
required outside UI callbacks. Initial private-browsing requests must reject if
no actual embedding API can set that mode; never mislabel a public view private.

## Outstanding gates

Frozen canonical host/core/Vibe tests and signed native fixture; post-naming runtime
and optional profiler; complete Chora engine/canvas/crypto/identity tests; actual
separate worker/authorized bridge/native frame path; supported OS containment;
Unix asset tests under WSL; device loss, media, performance, installed artifacts,
signing/update/recovery and cross-platform release matrix. No production-ready claim.

## Final pause updates

- Dependency agent confirms post-naming public native+refract check passed (17.81s)
  and default-hybrid+prelude lifecycle executable built (9m01s). It was not launched.
  Read resume-dependencies for its exact new executable hash before any future run.
- Latest policy suite is 55/55. The lightweight sibling GPU consumer passed real
  metadata/source verification with nine Etalon packages at canonical 6ace6cc;
  its actual compile was not started. The worker manifest has not been created.
- Chora's actual build found the proposed generic allocator wrapper referenced
  unavailable Marker/Id exports. Agent replaced it with concrete per-resource
  Mutex<EtalonIdentityManager<Resource>> fields and formatted; that correction
  remains untested. Canvas readback now handles fallible mapped ranges and unmaps
  on error; its correction also remains untested. API probe was deliberately
  interrupted for the pause. No owned Chora builds remain running.
- Chora confirmed new top-level views are public and have no private builder
  setter: reject private_browsing=true in the initial worker. Moving generic GL
  failure diagnostics from stdout to stderr is recorded but not yet applied.
- At checkpoint creation the host agent's initial protocol-file write was still
  pending. Do not assume those draft files exist; inspect its final packet and
  disk on resume. No codec, state guard, mailbox, tests or host adapter exists yet.
- All agents confirmed no owned Cargo/worker execution remains active. Only final
  checkpoint/file-write bookkeeping may still be settling; no new builds or
  implementation are authorized during this user-requested pause.

Final dependency handoff: lifecycle executable SHA256
`53c6715a9d8836a199ab7b5de746a7088fdae0d27f6640ce71ca6e15c189179e`,
52,608,512 bytes; profile `native-default-hybrid-prelude-post-etalon-naming-post-ime`.
It is built but unexecuted. Receipt:
`resume-etalon-naming-hybrid-build-receipt.json`.
The host agent's initial protocol write subsequently settled successfully; the
new standalone protocol draft still lacks codec/state/mailbox/tests and must not
be treated as buildable. No canonical parent adapter or worker implementation
has been written. Resume from the exact on-disk draft and the host pause packet.

## Resume continuation — 14 November 2026

The user authorized resuming this checkpoint. Current implementation/evidence
continues in `resume-worker.md`, the individual agent packets, and the live
architecture tracker. The pause statements above are historical; immutable source
captures and unrelated user work remain preserved.

The user subsequently requested a comprehensive handoff. Read
[HANDOFF-2026-11-14.md](HANDOFF-2026-11-14.md) for the consolidated current status,
verified evidence, exact runtime inputs and ordered remaining jobs. The actual
worker now builds; page/pixel execution and release gates remain separate.

## Current continuation — 15 November 2026

Read [Intake 1](UPSTREAM-INTEGRATION-REGISTER.md) and the updated
[work-packet board](WORK-PACKETS-2026-11-15.md) before resuming. The three sibling
commits were verified; an explicit live profile and bounded client diagnostics
were implemented. Actual staged-worker localhost denial/RGBA and three supplemental
page-window crypto cases now pass. The combined native/host library check passed,
but QualiaDB changed during the run and failed the post-run source gate. Refresh
its receipt before gated continuation; do not treat that compile as immutable
qualification. The register preserves exact evidence, restrictions and remaining
jobs. No user confirmation barrier remains; preserve ongoing sibling changes.
