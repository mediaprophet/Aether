# Chora Etalon and C API review candidates

Status: **applied locally, uncommitted and runtime-unqualified**. The resumed session verified all 62 combined patch postimages and applied the series to C:/github/chora. Cargo regenerated the root lock and its GPU source gate passes. See [current evidence](../resume-chora.md). Source migration and focused real-vendor tests do not establish browser/native production readiness. The preserved isolated source snapshot excludes WPT and .git and is not a complete source distribution.

## Final patches and order

1. Apply `C1-etalon-source-migration.patch`.
2. Apply C2 `../chora-embedding/chora-c2-embedding.patch` and C3 `../chora-tls/chora-c3-tls.patch`.
3. Apply `C10-capi-startup-ownership.patch` **after C3**: it calls the new consuming `try_build`.

Use `git -c core.autocrlf=false apply ...` when comparing exact file hashes on Windows. Both final patches apply to copied original files, and all 41 C1/C10 patch-stage postimages match their manifest hashes. Parent independently verified the combined C1/C2/C3/C10 series across 62 files in `../chora-patch-verification.json`.

`baseline-and-candidate-sha256.json` records original and candidate hashes per file. For shared constellation source/manifests, C1 exports only its GPU edits; observed combined snapshot hashes are separate fields where C2/C3 also changed the file. C2 omits the duplicated GPU import hunk. `patch-sha256.json` covers the final patch files. Earlier PARTIAL/old C10 checkpoints are superseded and removed.

## Implementation

Actual Etalon core/types/HAL/Naga/remoting sources and Etalon-backed optional canvas renderer/shader packages are pinned to the reviewed target described in `SOURCE-AND-API-MATRIX.md`. Core Global no longer crosses threads. Completion callbacks marshal work back to its owner, map guards pin memory through copies/writes, and external images own CPU bytes. Queue/device/pipeline identities retire only after their real resources are removed; request-send and response-delivery failures release reservations.

Canonical remote types, ID allocation, descriptors/FfiOptions, encoder APIs, shader diagnostics, scopes and errors replace the old direct core assumptions. Valid backend device-creation failure now resolves an already-lost DOM device backed by genuine already-destroyed private Etalon resources. The private Noop adapter never enters ordinary adapter selection. DeviceReady follows weak DOM registration before loss notification can cross the separate channel. Map replies and queue completion preserve owner ordering; Abort vs Operation errors follow canonical map classification.

C API build consumes/reclaims the raw builder before Rust startup, calls `try_build`, returns null and logs TLS configuration errors, and catches Rust startup unwinding before the C ABI. The C10 patch depends on C3's Result-returning builder. It does not establish atomic cleanup of every possible partially started engine or safety for invalid caller handles/foreign callback unwinding.

## Original checkpoint verification

- `api-probe-runtime.txt`: **11 passed**, executing actual production helper modules and pinned Etalon resources: private closed device creation/import, buffer/map/encoder/queue/scopes, cross-device rejection, genuine invalid-view propagation, lost async pipeline registration, map-before-queue completion, three registration-barrier tests and four owner-queue tests.
- `owned-handle-runtime.txt`: **2 passed**, ownership transfer and Rust panic cleanup.
- `source-policy-tests.txt`: **5 passed**, wrong registry/revision/optional upstream paths rejected; only reviewed public shim identity permitted.
- `source-policy-probe.txt`: **passed** for the Cargo-generated independent graph, including optional renderer source resolution.
- `rustfmt-check.txt`: **passed**, owned Rust source parses and matches rustfmt.
- `patch-postimage-check.json`: **41 exact patch-stage postimages matched** with core.autocrlf=false.

These focused checks compile the real Chora type/encoder/helper source against real Etalon; they do **not** compile the entire browser dispatch/DOM/IPC module graph or execute SpiderMonkey/WPT/CTS. Independent source review found no further concrete C1/C10 finding in the reviewed paths; it is not merge or production approval.

## Open release gates

- Historical `root-webgpu.txt` stopped at unavailable Stylo; local resumption retrieved the exact `a05fadffd43bc173b2d6e1e2caf44a36b45354af` source and regenerated root Cargo.lock with Cargo. The current root GPU inventory passes; the old `source-policy-root.txt` failure is historical. Complete full engine/DOM/IPC/C API compilation and all target/optional graphs before closing C01/C02. Current compiler output is recorded in `../resume-chora-webgpu-final.log`.
- Historical `api-probe-canvas.txt` stopped at uncached `svg_fmt 0.4.5`; local resumption retrieved that crate. Complete enabled/disabled canvas compilation and real backend tests.
- Run browser GPU WPT/CTS and WGSL, limits, resources, mapping/cancellation, loss/scopes/promises, workers, early/late notification/navigation/teardown, allocation/lifetime, CPU/no-adapter and platform-backend/surface tests. No WebView or Noop result substitutes for native GPU evidence.
- Align reviewed source revisions across Aether, QualiaDB and hypermedia before shared devices/resources. Review final distribution licenses/notices, media exclusions and FreeType election with actual resolved packages. Preserve separate Chora/WebRender/Surfman interoperability and sandbox qualifications.
