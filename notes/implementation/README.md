# Aether and Chora implementation evidence

Copyright (c) 2026 Timothy Charles Holborn. Aether repository licensing applies.
Chora patch payloads preserve the fork's MPL-2.0 source headers and attribution.

This is an implementation checkpoint, not a production release declaration. The
[task tracker](../architecture/14-IMPLEMENTATION-TRACKER.md) owns the remaining work;
[REVIEW.md](REVIEW.md) records independent findings and corrections.

For the consolidated current status and comprehensive remaining-job inventory,
start with [the 14 November handoff](HANDOFF-2026-11-14.md). It distinguishes
the completed actual worker build from outstanding runtime and release gates.

The [15 November continuation packets](WORK-PACKETS-2026-11-15.md) define agent
roles, relative cost, deliverables and acceptance dependencies. Sibling-dependent
execution waits for the user's update and the
[upstream intake](UPSTREAM-INTEGRATION-REGISTER.md); these plans do not dispatch
implementation work.

Local recovery continues in [native recovery](resume-native.md),
[dependency recovery](resume-dependencies.md), [canonical host](resume-host.md)
and [Chora recovery](resume-chora.md). These records supersede the original
checkpoint's access/cache restrictions and distinguish newly executed builds
from remaining runtime and release qualification.

## Implemented in the Aether working tree

- Source-aware GPU policy, regression fixtures, CI path coverage and actual native
  renderer build profiles. Owned GPU imports and dependencies use Etalon names;
  policy verifies the actual resolved sources and rejects inherited wgpu aliases.
- Renderer feature selection, non-WASM networking guards and fallible runtime/event
  loop startup APIs with preserved underlying errors.
- Callback dispatch permits removal/registration/nesting without retaining a
  registry borrow across user code. Initial and auxiliary windows share context
  preparation and rebuild; pending replacements survive unavailable surfaces.
- Unsupported document evaluation has stable Unsupported results; real evaluator
  owner expiry remains Finished. Public DocumentContext matches Rc lookup.
- Packaged asset decoding/containment, explicit trusted source-file compatibility,
  scheme allowlist, correct unsuccessful-response behavior, no wildcard CORS, and
  safer MIME defaults. Windows checks the actual opened file handle before reading;
  MIME inference uses those same bytes. Android JNI/buffer errors return failure.
- A required canonical QualiaDB/Vibe/ABI host composition crate at
  [packages/qualia-host](../../packages/qualia-host/README.md), with bounded execution,
  authority tickets, receipts, cancellation and native-thread delivery. Its separate
  workspace is intentional while the complete dependency graph is qualified.

Native IME typed routing and editor commit semantics pass 20 actual native-dom
source tests. The required host adapter passes 25 tests, including real signed
authoring sessions, disk journal/reopen, actual Aether task ownership and reentrant
waker teardown. Chora embedding and TLS records distinguish tested code from
outstanding engine integration evidence. The reviewed Chora patches are now
applied to the actual sibling checkout; its recovery record contains validation.

## Reproduce targeted library tests

From the repository root, with Rust/Cargo and a valid writable cache:

```powershell
python scripts/check_gpu_policy.py
python -m unittest discover -s scripts/tests -p 'test_*.py' -v
python packages/native/tests/renderer_feature_matrix.py
python scripts/verify_native_core.py --offline
```

`verify_native_core.py` compiles the actual document and asset-resolver path crates,
uses repository public API test files, and compiles the actual private asset policy
as a module. Its two checked-in fixture locks deliberately describe smaller graphs,
not the root shipping graph. The verified cache used here is
`target/production-audit/cargo-home`; the script respects CARGO_HOME without changing
system configuration. Remove `--offline` only when dependency retrieval is intended.

Observed results: 11 GPU policy tests; 17 renderer-wrapper contract configurations;
3 document API tests; 6 asset policy tests (1 junction fixture ignored); 2 asset API tests; 2 transport scheme
policy tests; 3 callback registry tests. The renderer contract uses narrow mock
backend interfaces and does not prove compilation of renderer packages. Host and
Chora results are recorded beside those implementations, with their precise inputs.

- [Document test log](document-api-tests.log)
- [Asset policy and public API log](asset-api-tests.log)
- [Callback registry test log](event-registry-tests.log)
- [Native input verification](native-input.md)
- [Chora embedding candidate](chora-embedding/README.md)
- [Chora TLS verification](chora-tls/README.md)
- [Canonical host verification](../../packages/qualia-host/VERIFICATION.md)
- [Combined Chora patch series](chora-patches.md)
- [Chora Etalon candidate](chora-etalon/README.md)
- [Missing dependency inventory](dependency-cache-report.json)

At the original checkpoint, the Windows junction replacement fixture failed at
fixture creation (`New-Item: Access is denied`), before containment was exercised.
[Its log](asset-junction-attempt.log) is retained. It is explicitly ignored in the
ordinary local suite and explicitly selected by the Windows native CI job. Run
`python scripts/verify_native_core.py --offline --include-junctions` on a machine
with directory-junction permission; an inability to create it is not a passing
containment result. The resumed Windows run now passes this fixture; its new log
is linked from dependency recovery. Unix runtime validation remains separate.

## Qualification work remains explicit

The original checkpoint could not resolve registry metadata and recorded 614 of
1363 archives absent. Authorized retrieval has since restored inputs for actual
native-DOM, default native and patched CPU builds, which pass. Remaining profile
checks are tracked individually; no package checksums were substituted.

Aether, QualiaDB and Chora's remote shipping manifests still have different Etalon
pins. An explicit local native-plus-host consumer resolves one canonical Etalon
source with actual sibling SDKs/adapters. Shared GPU objects remain unqualified.
Chora's Cargo-regenerated root lock now passes its GPU source gate and its actual
production WebGPU package builds. Full embedded-browser and optional renderer
builds, conformance and hardware tests remain separate gates.

Remaining work includes actual native window/input/device runtime tests, scoped
canonical graph authority and subscriptions, durable host transaction integration,
source/profile coherence, caller authorization across mixed sites, platform sandbox
qualification, packaged launches and release/update evidence. Windows opened-handle
validation assumes a stable trusted root; Unix now uses directory-relative,
no-follow opening with runtime validation pending. It is
not protection against malicious hardlinks installed by the root owner or admitted
file-content changes. Attributed local Blitz sources now supply typed terminal
fetch errors and waking cancellation; the actual traits/network regressions pass.
DOM/native integration results are recorded separately in native recovery.


Independent preparation results are in
[preparation-2026-11-15](preparation-2026-11-15/README.md): preservation receipts,
local artifacts/notices, native/distribution fixtures and host/media/compatibility
acceptance scenarios. No sibling-dependent work or runtime qualification was run.
