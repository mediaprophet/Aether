# Resumed native dependency qualification

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

This records local work resumed on 12 November 2026. Compilation and source
identity are separate from native runtime and release qualification.

## Dependency recovery and actual builds

The old cache-only failure was reproduced: root `cargo check --locked --offline
-p aether-native-dom` stopped at missing `pretty_assertions` metadata. An ordinary
online attempt failed DNS. The approved network-capable execution successfully
retrieved actual registry metadata and archives, retaining checksums and package
identities. No fabricated index entries or substituted dependency sources were
used.

With `CARGO_HOME=C:/github/Aether/target/production-audit/cargo-home`, the actual
root `cargo check --locked -p aether-native-dom` **passed** in 4m42s. Compiler:
`rustc 1.99.1 (58cf8f9c3 2026-10-13)`, Cargo 1.99.1, Windows MSVC. This used the
root shipping graph, not the earlier narrow native-input wrapper.

Cargo naturally updated the asset-resolver lock entry after the supervisor added
its Unix libc dependency. The actual default native build **passed** in 18m47s
against the initial registry Blitz graph. After the attributed local Blitz patches,
the locked CPU+SVG native check **passed** in 4m36s. Logs are
`resume-native-default.log` and `resume-native-cpu.log`. Subsequent application cap
configuration, asset-root discovery and paint-feature fixes require the final
profile regressions; an unfinished log is not a passing check.

Actual root asset tests passed seven policy tests and two public API tests. The
previously permission-limited junction replacement fixture was then explicitly
executed through the built test binary and **passed** (one selected test, 2.55s).
This verifies that fixture, not every possible filesystem race. The supervisor
subsequently added an asset-root discovery regression, which needs the final run.

Actual Blitz-traits (five tests) and Blitz-net (two tests) passed, including
multi-waiter cancellation, waker drop reentrancy, exactly-once completion and late
success after cancellation. These maintained local packages are now explicit
workspace members and cannot be published under upstream registry names. Before
that change, selecting the non-member packages inherited Aether's default example
workspace feature roots, including CLI/fullstack TLS/compression features. That
unnecessarily broad build was stopped. Membership fixes the test selection; the
new narrow run passed in 3m51s (`resume-network-tests.log`). Cargo also resolved
14 optional cache-package entries newly exposed by those workspace members.

The shipping GPU source gate and all 40 source-policy/snapshot tests passed. These cover
upstream platform/helper components, exact shim identity, canonical host paths,
shared style-engine identity and host exclusion from the generic engine profile.
Actual patched Blitz DOM completion tests passed 2 tests. Actual native CPU plus
SVG/prelude library tests passed all 9 tests, including transport terminal states,
asset authority/caps and callback registration lifecycle. Windows and WASM targets were installed
at the initial checkpoint; the supervisor owns subsequent Linux qualification.

The full patched root native-dom library suite subsequently passed all 20 tests
(not the earlier isolated fixture), and document passed 2 library plus 3 native
lifecycle tests. Final asset-resolver execution passed 8 library plus 2 public API
tests, with the explicit-junction test ignored in that ordinary run; the separately
executed junction test result above still applies. The new bundle-root discovery
regression is included. Logs are `resume-native-dom-tests.log`,
`resume-document-tests.log` and `resume-assets-final.log` beside this note.

Both `native_lifecycle_smoke` and `native_multiwindow_smoke` built successfully
with CPU softbuffer, SVG, system fonts and prelude. The supervisor executed the
first actual window smoke: exit 0, three redraw callbacks and component-owner
drop (`resume-native-smoke-runtime.log`). The first multiwindow probe timed out;
the corrected harness no longer depends on redraws of an occluded first window.
The supervisor's second real execution exited 0, observing both owners initialize,
the late second window redraw, first-owner drop, continued second-window redraws
and final-owner drop (`resume-native-multiwindow-runtime-final.log`). The four
actual DOM clipboard editor regressions also passed after correcting their test
fixture's KeyState import (`resume-dom-clipboard-final.log`).
These narrow lifecycle checks do not establish
the combined canonical host/GPU path or visual/input/accessibility readiness.

## Explicit local integration profile

`scripts/prepare_local_integration.py` creates an isolated consumer under
`target/local-integration`. It selects actual local hypermedia adapters and a
clean canonical Etalon checkout at
`6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e`. The root remote pins remain unchanged.
The generated receipt records the selected checkout, hypermedia HEAD/status and
adapter manifest hashes. Etalon source changes are rejected. A checkout nested
under Aether is rejected because Cargo can capture its inherited workspace fields
when loading it as a path patch. The current qualification source is an independent
Git checkout at `C:/github/etalon/target/aether-canonical-6ace6cc-object`, with clean
detached HEAD at the exact commit and read-only object alternates to the sibling
Etalon repository. This avoids access-denied errors on the user Cargo cache. No
dirty sibling source or archive identity exception is used by the source gate.

Example from Aether (use an existing readable, clean checkout of the exact pin):

```powershell
python scripts/prepare_local_integration.py --etalon-source C:/github/etalon/target/aether-canonical-6ace6cc-object
cargo metadata --format-version 1 --manifest-path target/local-integration/Cargo.toml > target/local-integration/metadata.json
python scripts/prepare_local_integration.py --verify target/local-integration/metadata.json
cargo check --locked --manifest-path target/local-integration/Cargo.toml
```

Full metadata resolution succeeded for the native-only consumer (13 GPU/shader
packages), then the actual native plus canonical host consumer (29 packages).
The latter also passed checks for all four canonical QualiaDB/Vibe/ABI package
paths and all three reviewed local Blitz package paths. The combined native/host
consumer initially had 832 resolved packages; the public facade and two native
graph examples expand it to 859. Actual combined compilation remains separate
from this source identity result. Registry Naga used independently as shader
build tooling is allowed consistently with the shipping policy; Etalon's own Naga
must share the exact canonical checkout. Upstream wgpu implementations, mixed
Etalon source revisions, external renderer adapters and incomplete metadata are
rejected. This is **graph evidence only**, not a completed combined native build
or shared-device runtime qualification.

`--with-host` adds the actual required host package, public Aether/native facade
and explicit production/fixture native graph examples. Use distinct `--output`
directories for different combinations. The host profile is at
`target/local-integration-host`; it must be compiled after the concurrent canonical
core test build finishes to avoid compiling two large core graphs simultaneously.

`--with-chora` is explicitly an **experimental in-process diagnostic**, currently
failing dependency resolution. First, native Blitz's registry Stylo and Chora's
Git Stylo both declare `links=servo_style_crate`. Their versions are both 0.22.0,
but the registry provenance is commit `34d25f741765a0bdb894f8271852e633299352d8`,
while Chora uses `a05fadffd43bc173b2d6e1e2caf44a36b45354af`. The diagnostic consumer
explicitly selects Chora's eight Stylo package pins; this resolves source identity,
but has not established API compatibility through compilation.

Resolution then exposes incompatible ICU constraints: native Blitz requires
`icu_casemap ^2.3`, while SpiderMonkey `mozjs_sys 153.3.0-3` requires
`icu_capi =2.1.2`, which requires `icu_casemap ~2.1.1`. Text transformation and the
JavaScript engine's pinned ICU ABI have both been preserved. **A4/C4 follow-up:**
implement and qualify Chora through a separate process/ABI consumer, with bounded
transport, authenticated host operations, lifecycle cancellation and process
isolation. Do not present a dependency alias or forced version change as working
in-process embedding. Chora's independent build remains a separate gate.

The `--engine-only` consumer provides the separate generic Chora graph. It resolved
881 packages and passed full metadata/source verification for 13 canonical Etalon
core/shader packages, exact Chora/Stylo sources and absence of host authority or
Vibe packages. Chora uses Etalon core directly, so this profile requires the real
`etalon-core` implementation without requiring its unused high-level wrapper.
Authentic missing archives were fetched against the lockfile. Actual engine
compilation at this canonical revision remains distinct from this metadata result
and from Chora's ongoing reviewed-revision build.

## GPU API qualification fixes

The actual no-SVG CPU check passed after making the existing blitz-paint dependency
mandatory for native custom widgets. Vello compilation then found two previously
hidden adapter API gaps at the reviewed shipping pins: boxed Naga validation
errors and three missing texture-view swizzle fields. The exact hypermedia
`a46b4e2ba5a9d57fc1a68e20ae87ce4bc2d076cf` shader and renderer packages are maintained
under `vendor/hypermedia` with original licenses, source migration records and
manifest provenance. Fixes preserve validation and use identity channel mapping.
The corresponding sibling source receives the same focused fixes. Etalon pins
remain unchanged. Final GPU compilation results are recorded separately from
these fixes; the original failure log is `resume-native-gpu.log`.

The following **locked/offline** shipping checks all passed before the subsequent
IME surrounding-text implementation (that later change has separate test evidence):

- Native Vello plus SVG: 4m09s (`resume-native-gpu-final.log`).
- Native default plus CPU softbuffer: 7m14s (`resume-native-combined.log`).
- Public Aether facade with native and refract: 5m26s (`resume-native-facade.log`).
- Vector compute renderer with its real optional GPU profiler: 5m12s
  (`resume-native-profiler.log`).

The default hybrid plus prelude lifecycle example built successfully in 35m09s
(`resume-native-hybrid-smoke-build.log`), replacing the earlier CPU binary. After
final locked graph/source verification (`resume-native-hybrid-locked-tree.log`),
the supervisor ran that actual GPU profile: exit 0, three redraw callbacks and
component-owner teardown (`resume-native-hybrid-runtime.log`). This establishes
selected-profile window lifecycle, not pixel correctness, physical-adapter choice
or device-loss behavior.

Adding the optional profiler's Git dependency initially advanced the cached
Hypermedia branch during resolution. This was caught before readiness handoff;
Cargo restored the exact reviewed commit and every incidental registry version
bump. The original profiler package is now retained unchanged with its original
relative dependency path. Cargo metadata confirms it is excluded from Aether's
workspace development targets while the renderer remains a member. Its optional
API was actually checked as recorded above; its source and complete manifest are
still included in source-policy inspection.

Policy version 2 now independently pins the Hypermedia commit and actual package
inventory, even if a newer adapter commit keeps the same Etalon version. Its only
local exceptions are the three named vendor manifests with exact path, package
name and version checks. Regressions cover adapter drift, unknown local paths,
changed manifest identity and approved overrides. Canonical opt-in consumers
continue to use their separate explicit source policy.

The retained Vello package manifests require Rust 1.89. This profile has been
tested with Rust 1.99.1; the repository's broader Rust 1.85 claim is not established
for this native renderer graph. Minimum-version qualification remains outstanding.

## CI coverage

The `native-assets` job now executes the actual asset-resolver library and public
API regressions on Linux, macOS and Windows. Windows also explicitly runs the
junction replacement fixture. Existing actual native renderer profile checks
remain in place, now including minimal CPU without SVG. Native's existing
blitz-paint dependency is required so its custom-widget feature matches the DOM
even in that minimal profile; this does not enable SVG or another backend. Typed
network cancellation and filtered DOM completion tests are explicit CI steps.
Clipboard editor regressions, the actual public native/refract facade and the
optional profiler-enabled vector renderer are also explicit checks. Windows runs
the supervisor's two real-child watchdog regressions (bounded process lifecycle,
not a sandbox claim). All vendor/script changes trigger these checks. Remote CI
execution has not been claimed.

The workflow's minimal CPU matrix entry had an indentation defect caught during
independent review. It is corrected, and the complete YAML parsed successfully
with PyYAML 6.0.3, asserting all five expected peer profile/argument entries. The
actual Blitz IME editor tests are also an explicit CI step. The parser was
installed only in the ignored workspace `target/ci-yaml-tools` directory.

## Vello lifecycle build and frozen host preflight

The actual Vello+SVG+system-fonts+prelude lifecycle example built in 7m28s and
passed the shipping source gate. Its executable SHA256 is
`48b8ae6d10f826f4c53c9806c12231b989fb151946301212a432b8b345477717`;
the build receipt is `resume-native-vello-build-receipt.json`. This executable
predates the later IME source changes. Windows Application Control prevented the
supervisor's launch before process creation, and two normal execution approval
reviews timed out. There is no Vello runtime pass and no policy bypass.

The canonical host's immutable v2 source capture passed byte verification, but
Cargo metadata exposed an inherited-workspace lookup error. The host agent
derived v3 solely from the captured bytes, retaining the parent receipt and
original manifests and explicitly binding the seven canonical packages to their
captured Qualia workspace. The v3 receipt contains 4,363 inputs with SHA256
`613fd195b04fb3328365ea707531b02a6b77531a96a52d1fee72196f2e2a7346`.
The frozen consumer is `target/local-integration-frozen-native-host/Cargo.toml`;
its full offline metadata/source verification passed: 859 resolved packages,
eight captured canonical packages and 29 canonical Etalon packages at exact
`6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e`. The resolved `qualia-core-db` feature
list is empty, preserving the explicitly disabled default GPU features. Canonical
security tests take the shared debug0/jobs1 build slot before native host example
compilation. Live source changes after capture do not invalidate the immutable
snapshot, and are not silently substituted into it.

The actual host package tests require their own workspace because Cargo does not
admit an outside-workspace dependency's development dependencies for `-p` tests.
The immutable v4 derivation retains v3 implementation bytes and original locks,
records naturally generated host/core test locks, and supplements the capture
with the canonical workspace's `imap-proto` vendor patch (16 files/77,963 bytes,
separately pre/post hashed). That patch is not used in the selected CPU graph;
its canonical source and optional/target semantics are nevertheless preserved.
The v4 receipt contains 4,382 files/nine package identities, SHA256
`3bcee7dc35c4882ac02053f7efdf5f45de8024d5aea08a3d4cbf581d175c66b7`.

Both actual v4 test-root source gates passed: host metadata contains 536 packages
and eight resolved captured packages; canonical workspace metadata contains 558
packages and seven resolved captured packages. Both select 28 Etalon-family
packages from the clean exact canonical revision, with `qualia-core-db` features
empty. Generated consumer and test-root patch tables matched exactly, including
the captured canonical ABI and IMAP patches. The tests run from real captured
workspace members, not synthetic API fixtures. Results of actual compilation and
execution are recorded by the canonical host agent separately.

The dependency/source regression suite passed 53 tests at this stage, including
captured workspace membership, source substitution, preserved unused canonical
patches, strict adapter paths and fresh per-invocation ancestry checks. Snapshot
verification now caches repeated ancestry checks only within one invocation;
every input leaf is still checked and hashed, and the independent inventory walk
rechecks all directories/files for links.

## Explicit Etalon naming migration

The user requested that active Aether names reflect Etalon's substantive fork and
QualiaDB-origin implementation. Root workspace/native/example dependency names
are now `etalon` and `etalon-context`. The native GPU profiles depend directly on
Etalon for Features/Limits. The texture example directory is now `etalon-texture`
(package `aether-etalon-texture`), and the
child-window example is `etalon_child_window`; their original upstream attribution
URL is preserved. The immutable shipping context adapter's historical type name
is confined to an explicit import alias exposing `EtalonContext` in owned code.

Maintained renderer defaults/features/dependencies use `etalon`, `etalon-default`
and `etalon-profiler`. Three documented legacy feature forwarders remain for
immutable external adapter callers. The profiler's library target is now
`etalon_profiler`; actual package identity and upstream URLs/licenses/original
manifests remain intact. Shipping source revisions remain unchanged. Naming-stage
actual renderer build verification is pending; earlier runtime receipts
identify their original source stages and are not relabeled as this revision.

Naming-stage verification passed 54 dependency/source tests and the full lockfile
policy. The actual Windows metadata graph contains 1,175 packages and no `wgpu`,
`wgpu-*` or `wgpu_*` package identities. Its Etalon runtime and Naga family retain
the exact reviewed shipping revision. All 17 actual renderer-wrapper selections
compile, including combined profiles and the explicit no-renderer diagnostic.
Only the first hybrid contract executable ran before Windows Application Control
blocked the next generated executable (error 4551); the remaining selector
forwarding executions are unclaimed. CI still executes the normal full test;
`--compile-only` explicitly separates local compilation evidence.

The workspace Etalon dependency now disables defaults, and the direct native
edge explicitly inherits that restriction. Existing root GPU/example edges
explicitly retain their previous defaults. Two minimal actual native consumers,
hybrid and Vello, exclude unrelated workspace examples from feature unification.
Their before/after metadata show no added or removed Etalon features after the
direct-edge restriction: both retain `default`, `dx12`, `etalon-core`, `gles`,
`metal`, `std`, `vulkan`, `web`, `webgl`, `webgpu`, `wgsl` through their selected
adapters. Inputs are under `target/production-audit/etalon-feature-union`, with
the exact shipping source asserted in both comparisons.


## User-requested pause — 2026-11-13

All owned long-running build/metadata sessions had finished when the pause arrived.
No owned Cargo process is intentionally left running. Session 86162 exited 0;
session 5233 exited 101 at the profiler build-script OS denial; sessions 1468
(adapter metadata/tests) and 30304 (independent Chora source gate) exited 0.

Final post-naming shipping checks passed locked/offline: native default 44.29s,
Vello+SVG 36.37s, minimal CPU without SVG 1m37s, default+CPU 12.12s, public
native+refract facade 17.81s. The profiler-enabled check was stopped by Windows
Application Control error 4551 before its generated shader build script executed;
no alternative launch was attempted and that current profile is not qualified.
The prior pre-naming profiler check remains separately recorded.

The current default-hybrid+prelude lifecycle example built in 9m01s and has NOT
been launched. It replaces the old pre-IME Vello executable. Exact SHA256:
`53c6715a9d8836a199ab7b5de746a7088fdae0d27f6640ce71ca6e15c189179e`. Profile: `native-default-hybrid-prelude-post-etalon-naming-post-ime`.
Receipt: `resume-etalon-naming-hybrid-build-receipt.json`; executable:
`C:/github/Aether/target/debug/examples/native_lifecycle_smoke.exe`.

The dependency/source suite now passes 55 tests. The new explicit
`--gpu-adapters-only` consumer is prepared at
`target/local-integration-adapters-naming/Cargo.toml`; actual Windows metadata and
source verification passed with nine Etalon-family packages at canonical 6ace6cc.
It selects the sibling context, renderer, shader and profiler libraries and
rejects application/database core dependencies. Its actual Cargo check has NOT
started. The canonical generator now covers all twelve adapter package paths and
requires a captured manifest hash for every resolved adapter. It no longer emits
an unused legacy compatibility-shim patch.

The disjoint Chora source-gate work is complete: only
`support/etalon/check_gpu_sources.py` and `test_gpu_sources.py` were edited under
Chora's MPL rules. Fifteen regressions passed, the Chora agent independently
reviewed them, and the actual gate independently passed against Chora Cargo.lock,
`target/etalon-profiler-metadata.json` and the finalized 124-file af57 source receipt.
This covers all three attributed local overrides and exact Etalon8e8/Naga sources;
full engine/all-features compilation is not inferred from that source result.

Worker handoff: no worker manifest was authored by this agent before pause
(manifest currently exists: False). Requested standalone manifest:
`chora={package="servo",path="../../../chora/components/servo",default-features=false,
features=["embedded_library"]}`,
`protocol={package="aether-chora-bridge-protocol",path="../chora-bridge-protocol"}`,
serde_json, dpi `0.1` and url `2.5` (the latter two read from actual Chora workspace).
Chora currently has an empty crates-io patch table and exactly three Hypermedia
path patches under `vendor/hypermedia/crates/`: vector-compute-renderer,
vector-compute-shaders and gpu-command-profiler. Preserve those exact af57 overlay
sources in the standalone worker, retain8e8 Etalon, and do not link worker/Chora
into the native workspace or enable internal multiprocess without real bootstrap.
Root owns worker Rust/launcher/frame flow; host agent owns standalone protocol and
parent adapter. Worker source-gate/manifest work remains to be implemented.

Resume order: inspect shared tracker/agent changes; qualify the sibling GPU-only
consumer normally without a heavy canonical core build; regenerate/source-verify
the frozen v4 native+host consumer after current naming (the v4 actual host/core
test-root gates passed, but fresh execution remains OS-blocked); perform the new
hybrid bounded runtime only when user resumes; complete the standalone worker
manifest, workspace-boundary and source-policy wiring using root's actual APIs.
Do not retry blocked helpers through alternative launch methods or relabel old
runtime evidence. Preserve all captured sources, originals, locks and logs.

## Resumed separate worker dependency boundary — 2026-11-14

`packages/chora-worker` is now an independent workspace with its own Cargo lock.
It selects the actual sibling `chora/components/servo` package under the `chora`
dependency key, disables defaults and enables only `embedded_library`. The worker
has the standalone protocol dependency plus dpi 0.1, euclid 0.22, url 2.5,
serde_json, libc and Windows standard-handle APIs. Chora's three attributed af57
Hypermedia patches and required development profile settings are retained at this
workspace boundary. No worker/engine edge is added to Aether's native workspace.

The first standalone resolution selected 99 identities absent from Chora's lock.
That initial lock is retained at
`target/production-audit/chora-worker-initial-resolution.lock`. To avoid incidental
upgrades, the worker lock was then seeded from the actual current Chora lock and
naturally regenerated by Cargo offline. The final lock has 894 packages; its only
identities absent from Chora's lock are the worker and protocol. No dependency
versions, checksums or package metadata were fabricated.

Actual Windows metadata generation passed using the existing production-audit
Cargo home: `cargo metadata --offline --format-version 1 --manifest-path
packages/chora-worker/Cargo.toml --filter-platform x86_64-pc-windows-msvc`.
The final document is `target/production-audit/chora-worker-metadata.json` and
contains 762 resolved packages. `python packages/chora-worker/check_sources.py
target/production-audit/chora-worker-metadata.json` passed against that real
document and lock. It verifies the actual sibling engine manifest, embedded
features, separate workspace, exclusion of privileged/native dependencies,
Etalon/Naga sources at exact 8e8ab6c, all three overlay patch paths and Chora's
complete attributed source receipt and the exact sibling protocol manifest.
The eleven adversarial policy tests passed (12.40s on the final run);
their synthetic inputs are policy fixtures, not source/build evidence.

The embedded feature selection does not resolve the optional renderer/profiler,
so Cargo correctly warns that those three patches are unused in this worker
profile. Their complete static overlay remains receipt-verified; optional
renderer/profiler actual metadata and compilation are Chora's separate gates.

The outstanding normalized overlay comparison was completed using UTF-8 Python
output: all 120 baseline files were read from actual af57 Git objects, and the
three retained normalized manifests matched those objects after line-ending
normalization. The 26 changed files contain the intended Etalon naming, narrow
compatibility exports, three identity-swizzle defaults and boxed shader-error
dereference; no unexpected behavioral change was found in the normalized diff.
Artifact: `target/production-audit/chora-af57-normalized.diff`, SHA256
`91eaed00db2e3404e309a217a2828b698f9c7f63a6471552845983e82cdf409b`.

Actual worker compilation uses the existing Chora target cache with jobs=1 and
the Chora working directory/configuration, shared production-audit Cargo home,
and `LIBCLANG_PATH=C:/Program Files/LLVM/bin`. The command is `cargo check
--locked --offline --manifest-path
C:/github/Aether/packages/chora-worker/Cargo.toml -j 1`. Its current evidence is
`resume-chora-worker-check.log`; compilation result is recorded when complete.
The Chora agent paused its separate crypto codegen while this check uses the
target. Source gates and compilation do not qualify transport runtime, pixels,
input, host authority integration or OS containment.

The worker check also entered a genuine native ANGLE rebuild. The existing
ANGLE output identity, features and Rust flags match the reviewed Chora profile;
all development profile overrides match exactly. Native object timestamps showed
steady compilation rather than a stalled build lock. No competing heavy build
was started, and no incomplete check is counted as passing.

`packages/chora-worker/stage_runtime.py` prepares a Windows development stage
only from successful actual `cargo build --message-format=json` output and the
source-gated worker metadata. It selects the exact worker package executable
and exact mozangle build-script OUT_DIR, rejects ambiguous/substituted records,
and copies the worker plus that OUT_DIR's libEGL.dll/libGLESv2.dll to a unique
Aether target directory. It records source paths and SHA256, copies actual
mozangle/ANGLE and repository license notices, and retains dependency license
declarations. LLVM readobj inspects actual PE imports; required installed MSVC
runtimes are recorded without copying system DLLs. Independent root review found
no actionable staging issue. Full distribution notices remain a release gate.

All 20 worker policy/staging-selection tests passed (7.94s): eleven source-policy
regressions plus nine artifact-selection regressions. These use explicit test
fixtures and establish no executable or browser runtime result. Read-only import
inspection of the older libEGL artifact verified the installed LLVM parser; it
was not staged or relabeled as the current build. Actual final build JSON,
staging receipt and real-worker readback execution remain to be recorded after
the worker compiles and links.

The initial actual worker check completed the native ANGLE stage and the whole
embedded Chora dependency graph, then reached the actual worker and exited Cargo
101 on two API mismatches: Surfman's error does not implement std::error::Error,
and DeviceIntRect is an euclid Box2D whose `new` takes two points. The original
`resume-chora-worker-check.log` is retained. Root corrected make-current failure
to a structured rendering failure and used `from_origin_and_size`. The next
same-consumer check writes `resume-chora-worker-check-final.log`; it also rebuilds
the independently corrected generic software-context resize implementation.

That final actual worker check **passed**, Cargo exit 0, in 5m41s. It rebuilt the
changed servo-paint-api source and its dependent engine crates before checking
the real worker. This closes the current worker compilation gate, including its
private-pipe helpers and corrected readback API usage; it is not a linked worker
or browser runtime result. A same-consumer `cargo build --locked --offline
--manifest-path C:/github/Aether/packages/chora-worker/Cargo.toml
--message-format=json -j 1` is now running. Its full exact Cargo output is
`target/production-audit/chora-worker-build.jsonl` and progress/error log is
`resume-chora-worker-build.log`. Chora's separate crypto test remains held until
this worker build/staging sequence releases the shared target cache.

The first actual build exited Cargo 101 before linking because a new mozjs_sys
OUT_DIR could not download its native archive in the restricted network. Its
normal source-build fallback then reported missing MozTools/MozillaBuild. The
failed JSON and log above remain intact; no executable or runtime result was
invented from the passing check.

Ordinary exact-version input recovery found the existing 90,609,446-byte archive
under Chora's older mozjs_sys OUT_DIR. A normal API request reproduced DNS error
11001; the standard network-capable request was approved and retrieved the
[official release metadata](https://api.github.com/repos/servo/mozjs/releases/tags/mozjs-sys-v153.3.0-3).
Asset 597143905 identifies
`https://github.com/servo/mozjs/releases/download/mozjs-sys-v153.3.0-3/libmozjs-x86_64-pc-windows-msvc.tar.gz`
with SHA256
`4668c61b92b178b65f555d3eb36652f16681bfeedbd1e2f41e063d4a78caace1`,
which exactly matches the cached archive and its size. No archive download or
source/profile weakening was needed. GitHub artifact attestation is not claimed;
the `gh` executable is unavailable.

The verified archive was copied to an independently created stable input folder:
`target/production-audit/native-inputs/mozjs-153.3.0-3-dpoz2sa1/`.
Its `source-receipt.json` SHA256 is
`2c0080c84476143e637a822e5e64c73d87b6d03eec627fd06c59827adf7ef554`.
The receipt preserves the official metadata copy, source/cache paths, pre/post
copy hashes and actual `intl,jit,libz-sys` features. The unmodified crate's
documented `MOZJS_ARCHIVE` local-path contract supplies this file to the build.

The same-consumer retry preserves distinct
`target/production-audit/chora-worker-build-verified-native.jsonl` and
`resume-chora-worker-build-verified-native.log`. Its real Cargo JSON has now
recorded successful mozjs_sys build-script execution and a newly built mozjs_sys
library artifact, clearing that missing-input blocker. The remaining worker
codegen/link and real runtime checks are not yet complete. The staging tool's
`--native-archive-receipt` option reverifies this package/features/archive identity
and retains its provenance and official metadata in the development stage.
All 20 worker policy/staging tests passed again (8.29s).

Independent review identified a concrete runtime-output ancestor gap: the fixed
logical Aether target path could follow an existing junction. The staging tool
now checks every existing output ancestor for symlinks, junctions/reparse points,
non-directory components and resolved repository escape before creating paths;
it rechecks the base and allocated unique directory before copying. Four narrow
regressions include a real Windows junction ancestor and verify its external
test-owned directory remains unwritten. Ordinary sandboxed junction creation was
denied with WinError 5; normal approved execution then passed all 24 worker
policy/staging tests (0.55s). No automatic approval review was rejected. This is
a prewrite ancestry guard, not a claim of race-free filesystem containment.
