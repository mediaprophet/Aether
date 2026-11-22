# Chora local implementation resumption

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
Chora source changes preserve MPL-2.0 and upstream notices.

## Recovered and applied

The sibling `C:/github/chora` checkout was clean before this work. The saved
C1/C2/C3/C10 patches were independently applied to a fresh staging directory by
`verify_chora_patches.py`, using the actual sibling files. All 62 patch-listed
postimages matched the saved candidate. See
[verification](resume-chora-patch-verification.json).

The four patches were then checked and applied sequentially to the sibling
checkout with `core.autocrlf=false`. No existing working-tree changes were
overwritten. Chora remains a separate workspace; no QualiaDB or Webizen-specific
engine dependencies were introduced. The old exported patches are historical
recovery inputs; the sibling checkout now also contains the subsequent fixes
below. No commit or release was created.

## Implemented and reviewed

- Cargo retrieved exact pinned Stylo `a05fadffd43bc173b2d6e1e2caf44a36b45354af`,
  Etalon/hypermedia sources and missing registry archives, including svg_fmt.
  Cargo regenerated the real root lockfile; no checksums or source identities
  were fabricated. It selects Etalon `8e8ab6c8349bcc6ab1d81138f2c9f2a0c90f0952`
  and hypermedia `af57e82120b30b47fca238edb21a839d2cdb4278`.
- A mandatory Chora main-workflow job checks the complete locked GPU source
  inventory and its regression suite. The aggregate result depends on this job.
  The source gate also requires the reviewed historical wgpu shim version,
  not only its Git source. CI execution has not been claimed.
- A successful GPU map is now retained only when its response is delivered.
  An owner-thread `MapDelivery` guard unmaps on response construction/delivery
  failure. Failed map operations never create the guard and cannot unmap another
  request's existing mapping. The guard holds the resolved resource rather than
  looking it up again through a potentially retired ID.
- Shared-memory size, integer conversions, map/copy ranges and write-back ranges
  are checked against the actual Etalon buffer. Invalid IPC dimensions cannot
  cause unchecked subtraction/slicing. Writes occur while the canonical mapping
  guard pins storage, and malformed write-back still ends in unmap. This does
  not provide a total memory budget or immunity to allocation failure.
- The first genuine owner-module compilation exposed formatting of a
  `TryReceiveError` that has no Debug implementation. Corrected the logging path;
  the next actual root WebGPU check passed. Migration-only unused message fields
  are now explicitly ignored, and the redundant DeviceScope device-id copy was
  removed after verifying that authoritative map keys/routing IDs remain intact.
- Supervisor independently reviewed mapped-delivery guard lifetime, checked
  dimensions, failure cleanup and successful handoff. No additional source-level
  finding was reported; runtime/browser qualification is separate.

## Executed evidence

| Check | Result | Scope |
| --- | --- | --- |
| Saved patch verification | 62 postimages matched | Pre-change source recovery |
| Root GPU source inventory | Passed | Real Cargo-generated lock, including optional renderer |
| GPU source-policy regression suite | 5 passed | Rejected source/revision/shim regressions |
| Actual root `cargo check --locked --offline -p servo-webgpu -j 2` | Passed | Real owner/IPC/shared types, native backend graph and canvas context |
| New mapped-delivery tests | 2 passed | Real pinned Etalon Noop buffers; retained/undelivered maps, valid writes, malformed sizes/ranges |
| Complete cached Etalon lifecycle fixture | 13 passed | Mapping, owner ordering/cancellation, readiness, closed resources and cross-device rejection |
| Updated TLS source-copy fixture | 11 passed | Real rustls handshakes using copied current implementation/tests; not engine/network qualification |
| Actual locked networking library suite | 8 passed | Six live TLS controller regressions plus two existing directory escaping tests; loopback HTTPS permitted explicitly |
| Actual locked TLS integration target | 11 passed | Production networking crate: roots, hostname/chain, mTLS, host verifier, snapshot reload and profile isolation |
| Actual locked config and embedder libraries | 16 passed | Eight config tests (including six admission tests) and eight embedder tests (including seven caller/lifecycle tests) |
| Production sandbox admission module | 6 passed | Actual source compiled directly with Rust on Windows; policy rejection only, not OS isolation |
| `git diff --check` | Passed | Local Chora source changes |

The successful root WebGPU check is in
[resume-chora-webgpu-pass.log](resume-chora-webgpu-pass.log). It preceded the
semantics-preserving unused-binding cleanup. The mapping regression, including
write-back validation, is in
[resume-chora-map-delivery-final.log](resume-chora-map-delivery-final.log).
The complete fixture is recorded in
[resume-chora-lifecycle-suite.log](resume-chora-lifecycle-suite.log).
The tests import production helper modules and actual Etalon resources; they do
not exercise physical GPU hardware, browser IPC delivery or WebGPU CTS/WPT.

The first full actual engine command was run locally:
`cargo check --locked -p servo --no-default-features --features embedded_library -j 2`.
Its output is [resume-chora-embedded.log](resume-chora-embedded.log). It has
retrieved the actual locked SpiderMonkey/ICU/crypto/font dependencies; a started
or partially compiled check is not a passing result. The first actual failure
was a leftover `wgpu_core::instance::RequestAdapterOptions` in constellation
messages. Script already sends the canonical remote request type; the message
now uses the existing `webgpu_traits::RequestAdapterOptions` reexport. The
original build completed ANGLE native compilation and exited 1 for that error.
The corrected check then reached constellation and found two missing exhaustive
tracing cases for the newly added message variants. Both now map to static
event-name targets (`ChoraMessage` and `ChoraMessageReply`), without logging
payloads. That attempt exited 1; a further actual engine check is recorded in
[resume-chora-embedded-retry.log](resume-chora-embedded-retry.log). The optional
canvas check is recorded separately in
[resume-chora-canvas.log](resume-chora-canvas.log).
The actual `servo-net` controller and direct TLS integration builds have
completed at one job. The corrected embedded check is now running, recorded in
[resume-chora-embedded-corrected.log](resume-chora-embedded-corrected.log);
the optional GPU canvas check follows it. Final results will be recorded when
observed. The installed compiler for this run is Rust
1.99.1, also selected by Chora's `stable` toolchain.

## Live TLS root updates under qualification

The local implementation now returns a non-serializable, non-cloneable host
TLS controller separately from content `ResourceThreads` channels. It uses weak
references so retaining the host handle cannot keep stopped network profiles
alive. HTTP and WebSocket operations acquire immutable TLS snapshots; the HTTP
pool and prepared TLS configuration are replaced together for each profile.
Both replacements are prepared before either is changed. Writers lock public
then private, swap both, release both locks and only then drop old pools. There
are no host callbacks or network awaits under these locks.

`NetworkManager::add_custom_root_pem` adds roots using the existing policy.
`replace_custom_roots` requires an explicit `TlsReloadScope`: `FutureConnections`
rebuilds both pools and session caches while permitting operations with already
acquired snapshots to finish. Cookies, HTTP cache, history, host certificate
exceptions, custom verifier and client identity remain intact and profile
separated. `ImmediateRevocation` returns Unsupported without mutation because
cancellation of every existing HTTP/WebSocket operation and exception has not
been qualified. Root replacement is not a claim that existing connections,
cached content or independently authorized host exceptions are revoked.

Focused controller tests cover invalid roots, actual second-profile preparation
failure, generations and acquired snapshots, stopped-profile lifetime, and
public/private cookie and HTTP-cache preservation. Real rustls handshake tests
cover old/new root behavior and preserved host-verifier/mTLS identity. All 11
parser/handshake tests passed in the regenerated source-copy fixture using Rust
1.99.1; source hashes are in [resume-chora-tls-source.txt](resume-chora-tls-source.txt)
and final output is [resume-chora-tls-handshakes-final.log](resume-chora-tls-handshakes-final.log).
The fixture has its own locked provider dependencies, so this is supplemental
evidence, not the actual Chora lock/build. The actual locked `servo-net` library
suite now passes all eight tests as detailed below. The direct integration TLS
suite also passes all eleven tests (392 unrelated integration tests filtered),
0.52s after a 4m10s build. See
[actual TLS integration](resume-chora-tls-integration.log). It compiles the final
networking import cleanup and covers host-verifier denial, mTLS admission,
hostname/chain validation, retained verifier/client identity on root reload,
old/new snapshots and independent profile overrides/resumption stores. Full
engine, WebSocket, platform trust and keystore requirements remain; C3 is not
complete.

A sixth controller regression uses an ephemeral loopback HTTPS server and a
generated CA. It checks that replacing a root creates new HTTP pools which
reject the removed trust even when an acquired old client has a reusable
connection; the explicitly retained old snapshot may continue. Actual locked
compilation completed in 115m38s. Five controller tests passed immediately; the
first loopback run failed at its initial trusted connection with Windows socket
error 10013 before TLS/cutover. The exact already-built test passed with
explicitly permitted loopback access (1/1, 0.18s), followed by the full actual
library binary (8/8, 0.49s). See
[initial run](resume-chora-tls-reload-unit.log),
[loopback retry](resume-chora-tls-loopback.log), and
[full library suite](resume-chora-net-library-tests.log). This is actual local
HTTP/TLS/pool qualification, not WebSocket, platform trust or whole-engine
qualification. The only subsequent networking source change removes an unused
logging import; the passing direct integration build includes that cleanup.

Independent supervisor review found no remaining actionable issue in the host
capability, dual-lock cutover, pool destruction, fresh resumption configuration,
caller bookkeeping or focused tests. A comment was corrected to acknowledge
that cloning a configuration under the read lock may allocate; fallible
preparation and client creation occur outside locks. Source review does not
establish live-network, WebSocket or platform trust behavior. Library-test
discovery is enabled for `servo-net` so the new private-controller tests are not
silently excluded by its inherited `test = false` setting.
The existing CI path was inspected: pull requests select `linux-unit-tests` in
`.github/workflows/main.yml`, `linux-common.yml` invokes `./mach test-unit` with
the CI nextest profile, and `python/servo/testing_commands.py` explicitly selects
`servo-net` without restricting its test targets. Existing nextest filters do not
exclude these controller or TLS integration tests.

## Integration constraints and outstanding gates

The joint native/host/browser consumer found two genuine source constraints:
Blitz's registry Stylo and Chora's Git Stylo share a native link identity, and
Blitz text-transform requires ICU casemap 2.3 while SpiderMonkey's pinned ICU C
ABI requires the 2.1 family. Preserve both supported feature sets. Continue the
mixed application through a separate Chora process/ABI boundary; dependency
resolution or mere process separation does not establish sandbox isolation.
See [dependency evidence](resume-dependencies.md).

The separate engine-only canonical consumer has now resolved 881 packages and
passed its source gate: 13 Etalon-family packages use the exact clean `6ace6cc`
source, with the actual Chora manifest/Stylo pin and no Aether/QualiaDB/Vibe host
dependencies. This proves graph identity only. It does not replace the shipping
Chora `8e8ab6c` build result or establish canonical-revision API/runtime
compatibility; see `target/local-integration-engine` and dependency evidence.

Remaining work includes completed full engine/script/DOM/C-ABI compilation,
optional GPU canvas compilation, actual protocol tests/external consumers,
browser WPT/CTS and native hardware behavior, platform sandboxing and scoped
host-authorized mixed-site messaging. Live TLS future-connection root reload is
implemented and awaiting qualification as described above; immediate revocation
and scoped keystore dispatch remain explicit unsupported requirements.
Cross-repository GPU sharing needs a jointly qualified source/API and device
ownership contract. Installed artifacts, notices, signing/update/recovery and
platform runtime tests remain release gates.

## Required sandbox policy and process composition

The original Windows multiprocess path launched an ordinary child and returned
`Process::Unsandboxed` without checking the requested sandbox flag. Following the
bootstrap showed that `sandbox=true` then called a Windows `create_sandbox`
implementation which panicked. This was unsupported launch followed by a panic,
not demonstrated continuing unprotected content execution. In-process content
had no sandbox activation path. Neither behavior establishes an OS boundary.

The focused C4 fix adds `servo_config::sandbox::validate_sandbox_request` and
typed `SandboxPolicyError` reasons for missing build support, disabled process
mode and unsupported platform. `ChoraBuilder::try_build` now returns
`ChoraInitializationError::{Tls, Sandbox}` and validates required isolation
before global options/preferences or threads are created. Both process launcher
branches validate before creating a bootstrap channel or child and preserve the
existing IPC error path as `io::ErrorKind::Unsupported`. Child bootstrap also
rejects unsupported policy before script initialization. The shell parser
reports unsupported requests as configuration errors before its legacy
panicking builder path.

`Opts::default` and ordinary parser defaults remain `sandbox=false` and
`multiprocess=false`; no flags are silently changed, and explicitly unsandboxed
embedding remains available. `--sandbox` now requires a compiled/enabled
multiprocess mode and a supported launcher/activation platform. Six tests in
the actual production policy source passed when compiled directly with `rustc
--test`; see [resume-chora-sandbox-policy.log](resume-chora-sandbox-policy.log).
The same tests are discoverable through the existing `servo-config` CI package.
Builder and shell-parser regressions are added but await their actual targets.
Independent supervisor review confirmed the validator matches the compiled
launcher branches, admission precedes global initialization and child launch,
defaults remain unchanged, and typed TLS/sandbox error sources are preserved.
No actionable source finding remained; actual engine/parser execution is still
required in addition to the six pure policy tests.
No Windows OS sandbox has been implemented or qualified by this admission fix;
untrusted Windows browsing remains an explicit release gate.

Reusable worker entrypoints inspected in the actual Chora source:

- `ChoraBuilder::default().event_loop_waker(...).try_build()` and
  `WebViewBuilder::new(&engine, Rc<dyn RenderingContext>).url(...).delegate(...).build()`.
  `EventLoopWaker` must cause `spin_event_loop()` on the owning engine thread;
  engine/context handles use `Rc`. `notify_new_frame_ready` calls `WebView::paint`.
- `SoftwareRenderingContext::new(PhysicalSize)` is the actual headless path used
  by `ports/servoshell/desktop/headless_window.rs` and the engine test harness.
  It is explicitly software, not Etalon GPU qualification. The GPU offscreen
  factory is `Rc<WindowRenderingContext>::offscreen_context(size)` and requires
  a window rendering context parent.
- `RenderingContext::read_to_image(DeviceIntRect)` returns `Option<RgbaImage>`;
  `read_pixels_into` returns a byte count. Read after painting and before
  `present()`, since readback is defined on the back buffer. Existing readback
  produces RGBA8 and flips the GL image vertically. The default buffer-copy
  method still allocates internally, including the flip copy; it is not zero-copy.
  Host frame dimensions, rectangles and checked byte counts need an explicit
  memory budget before invoking these APIs.
- `WebView::take_screenshot(None, callback)` asynchronously returns
  `Result<RgbaImage, ScreenshotCaptureError>` after load/fonts/images and render
  stability. Host deadlines and stale-generation checks are needed; it is not an
  unconditional live-frame polling API. Navigation uses `load`, resize uses
  `resize`, and input uses `notify_input_event`.
- `embedded_library` omits internal `multiprocess`. `run_content_process(token)`
  is the constellation IPC bootstrap, not a standalone external worker main.
  `ServoInner::Drop` sends Exit and spins until completion without a bounded
  public shutdown deadline; an external worker owner needs a bounded graceful
  shutdown followed by process cleanup. Process separation alone supplies no
  caller identity, filesystem/network restriction or sandbox qualification.

The existing generic worker messaging boundary is
`WebViewDelegate::notify_embedder_message(WebView, EmbedderMessageRequest)` and
`notify_embedder_message_cancelled`, in `components/servo/webview_delegate.rs`.
The request exposes `caller()`, `id()`, `is_cancelled()` and consuming
`respond()`/`cancel()` methods; dropping an undecided request denies it.
`ChoraCallerContext` carries engine instance, WebView, pipeline, navigation epoch,
exact origin and private-profile state separately from the page's channel/body.
The constellation validates the current active top-level HTTP(S) document and
origin, and rejects stale replies after route/deadline synchronization. The
worker must retain a bounded map of live request handles, forward only the
engine-supplied context over its owned private channel, and cancel host work when
the engine cancels a request. Serializable caller fields are not credentials;
the host must bind the worker session to its profile/grants and authorize every
operation. The delegate documentation now explicitly distinguishes document
eligibility from host authority. No competing scripting ABI is required.

## Current Etalon naming and compilation follow-through

The next real embedded check exposed three script WebGPU API mismatches and six
WebCrypto compilation errors. The two map-failure call sites now provide an
explicit operation error, and the render-pass depth slice uses the canonical
`FfiOption::None`. The crypto errors led to the host-authority audit below;
no new WebIDL usages were added and no software crypto algorithms were disabled.
The optional canvas check independently reached the exact `af57e82`/`8e8ab6c`
graph and failed at the boxed Naga validation error conversion in
`vector-compute-shaders`. Its real log is `resume-chora-canvas.log`.

Following the user's explicit naming instruction, owned dependency keys,
imports and internal helpers now use Etalon names. The backend module is
`components/webgpu/etalon_thread.rs`, its worker type is `Etalon`, and the backend
preference is `dom_webgpu_etalon_backend`. Standard WebGPU DOM/WebIDL names and
historical upstream URLs are unchanged. The inventory is
`resume-chora-wgpu-inventory.txt`. Earlier GPU/engine/probe compilation results
predate this naming pass and must be repeated; the current engine command is
recorded in `resume-chora-embedded-etalon-naming.log`.

The attributed renderer overlay was prepared from 120 exact Git-object
files at `af57e82120b30b47fca238edb21a839d2cdb4278`, with original normalized
manifests additionally retained as `Cargo.toml.af57`. It contains only the
renderer, shader and relative optional profiler closure. Its final source receipt
hashes 124 files, including the three preserved normalized manifests and the
attribution README. The original files were checked against the immutable af57
Git object; `resume-chora-af57-overlay.diff` records the 26 changed source/manifest
files, including a private module rename. Etalon stays pinned to
`8e8ab6c8349bcc6ab1d81138f2c9f2a0c90f0952`; no mutable revision or upstream GPU
fallback is admitted. The two generic source-gate files were independently
reviewed, and their 15 regressions pass. The actual source gate passed against
the regenerated root lock and full Cargo-generated API-probe metadata with
`canvas,vello/etalon-profiler`, verifying all three exact local manifests and the
Etalon/Naga family. A second agent independently repeated that gate successfully.
This is source identity evidence, not engine or hardware readiness. Full root
all-features metadata remains blocked by uncached accesskit packages; two exact
download requests timed out in automatic approval review. No substitute metadata
was invented. The CI inventory command still requires full root metadata.

The actual config/embedder library suite passed 16/16 after a 26m21s build;
see `resume-chora-embedding-policy-tests.log`. The embedding package has been
added to the existing `mach test-unit` selection so caller, budget, cancellation,
duplicate ownership, timeout and shutdown regressions are not omitted from CI.
These tests do not qualify a browser protocol or OS isolation boundary.

## Unscoped host-keystore admission correction

The WebCrypto audit found that reserved page-controlled host-key bytes could
select a process-global provider key without caller-scoped admission. The
import parser/construction shortcut is removed. Independent review caught that
rejecting lexical prefixes would also reject valid ordinary raw HMAC/HKDF/PBKDF2
material, so that intermediate guard was removed before qualification. All bytes
now follow the existing software algorithm import and validation paths; lexical
markers never select a provider or grant host authority. All four host-operation
helpers also reject existing `HostKeystore` handles through a shared denial
guard, covering handles already in memory. Ordinary software WebCrypto/KEM and
the existing four key/bits encapsulation WebIDL usage values remain unchanged.
The obsolete unsuffixed usages and Rust-2024 borrowed-pattern errors disappear
with the unsafe shortcut, rather than through new public API variants.

Only that import shortcut constructed `Handle::HostKeystore`; the four helpers
were the only WebCrypto global-provider dispatch sites. Structured clone already
rejects the host variant, and the serialized handle enum has no host variant to
restore. The remaining shared `HostKeyHandle` is data, not admitted WebCrypto
authority. New production tests install a counting provider in an isolated test
subprocess and cover existing-handle denial, clone denial, prefix-looking software
handles and real HMAC primitive roundtrips without provider calls. Browser WPT
cases separately cover prefix-looking raw HMAC import/sign/verify and non-UTF-8
software material. These new tests are not yet executed; unit tests alone do not
prove the DOM import route. Scoped host-keystore dispatch
remains an unsupported implementation gate, not a completed feature.

## Continued actual engine and canvas compilation

The corrected post-naming engine check reached three real worker-thread errors:
Etalon's remote `IdentityManager` owns a `RefCell` and is not `Sync`, while browser
workers share the Chora hub through `Arc`. The hub now wraps each resource-family
allocator in a mutex. Each operation holds one lock only during allocation/free;
there is no IPC, callback or nested hub operation under that lock, and no unsafe
thread-safety declaration. New tests compile this actual production source against
the pinned Etalon code and exercise concurrent live-ID uniqueness, concurrent
freeing and reuse with advanced epochs, plus independent resource families.
Actual engine and fixture reruns remain pending.

The optional canvas check compiled the attributed shader and renderer, then found
Etalon's fallible mapped-range API at the browser readback call site. That call
now matches the result, supplies `None` to the existing callback on error and
still reaches the existing unmap after the range has dropped. The actual canvas
rerun and optional-profiler check remain pending; software compilation alone does
not establish successful hardware readback.


## User-requested pause checkpoint — 2026-11-13

Work is paused at the user's request to charge the battery. No new build should
start until the user resumes. No changes were committed.

- The source inventory passed against the actual regenerated root Cargo.lock,
  finalized af57 receipt and `target/etalon-profiler-metadata.json`; a second
  agent independently repeated it. All 15 source-policy regressions passed.
- The embedded check in `resume-chora-embedded-final.log` reached the three real
  shared-ID allocator thread-safety errors. The first synchronized retry in
  `resume-chora-embedded-synchronized.log` exited 101 because the generic wrapper
  expected Marker/Id exports absent from Chora's narrowed trait module. The saved
  correction uses concrete `Mutex<EtalonIdentityManager<Resource>>` fields and
  locks every allocation/free directly. It has no unsafe Sync implementation or
  extra public exports; it is formatted but has not been recompiled.
- The default actual API/lifecycle fixture rerun, including the two new production
  identity-hub tests, was stopped with Ctrl-C on pause (session 84651, exit 1).
  `resume-chora-api-probe-etalon-naming.log` is interrupted evidence, not a pass.
  The original 13-test result predates naming/identity changes; a full current
  15-test run remains required.
- The actual attributed shader and renderer compiled during the optional canvas
  check; Chora canvas then failed at its fallible mapped-range call. The saved
  match handles failure and still reaches unmap. There is no successful corrected
  canvas check yet (`resume-chora-canvas-overlay.log`, exited 101, session 15196).
  An explicit optional-profiler actual compile also remains required.
- The unsafe lexical host-key import shortcut is removed. Prefix-looking and
  non-UTF-8 bytes now use ordinary software import validation. Existing in-memory
  host operations and structured clone remain denied. The new counting-provider
  test isolates its global registration in a test subprocess; WPT separately
  covers actual software HMAC imports and roundtrips. Neither new crypto tests
  nor WPT have executed. Respect Application Control if subprocess execution is
  denied; do not use an alternate launch workaround.
- Existing completed evidence remains actual net library 8/8, TLS integration
  11/11 and config/embedder library 16/16, as logged above. These do not establish
  complete engine, browser protocol, hardware or OS isolation readiness.
- Parent-requested diagnostic cleanup remains unapplied: only
  `print_diagnostics_information_on_context_creation_failure` in
  `components/shared/paint/rendering_context.rs:917` should change its println!
  calls to eprintln!, preserving messages and attribution. No new patch was
  started after the pause request.
- Confirmed the current public WebViewBuilder has no private-browsing setter and
  new top-level constellation paths use `is_private = false`. An initial host
  worker must reject private_browsing=true instead of claiming profile isolation.

All owned long-running sessions are closed: embedded 31057 exited 101 before the
stop request, fixture 84651 was interrupted, canvas 15196 exited 101. Source-read
sessions 16856, 58240 and 72453 completed. No Chora build is intentionally left
running. On resume, first inspect current source/lock status and these logs,
then compile the corrected hub/crypto, run the complete fixture, corrected canvas
and optional profiler checks, and continue actual embedded/builder/CLI/WPT gates.

## Resumed qualification â€” 2026-11-14

The current API/lifecycle fixture passed **15/15**, including both production
IdentityHub concurrency/resource-family tests, against the pinned Etalon source.
The cached run built in 3m17s and executed the tests in 0.07s. See
[current fixture log](resume-chora-api-probe-resumed-cached.log). The initial
resumed fixture attempt used a fresh target directory and was stopped before
completion to reuse the original cache; it is not a second test result.

The GL context-failure diagnostics in the generic rendering-context helper now
write to stderr with their messages preserved. The unexpected-WebView diagnostic
in the constellation also writes to stderr. Other engine diagnostics can still
write to stdout, so these changes do not establish clean binary framing on an
unredirected stdout stream. The standalone worker must own its protocol pipe and
redirect ordinary process stdout before engine/thread creation.

The resumed full embedded check has compiled the concrete synchronized
`servo-script-webgpu` hub and the corrected `servo-script-webcrypto` library.
The overall embedded check, corrected optional canvas/profiler check and actual
crypto admission tests are still running or queued; no success is claimed for
those complete targets until their final process results are observed.

The full actual embedded check subsequently **passed** (exit 0, 10m59s):
`cargo check --locked --offline -p servo --no-default-features --features embedded_library -j 2`.
See [embedded result](resume-chora-embedded-resumed.log). This includes the real
engine, script/DOM, corrected WebCrypto and synchronized WebGPU source graph;
it is compilation evidence, not browser or OS sandbox runtime qualification.

The corrected canvas and explicit optional profiler check also **passed**
(exit 0, 5m13s including build-lock wait):
`cargo check --locked --offline -p servo-canvas --features vello,vello/etalon-profiler -j 1`.
See [canvas/profiler result](resume-chora-canvas-profiler-resumed.log). The log
shows the actual local attributed gpu-command-profiler, vector-compute-renderer
and corrected servo-canvas sources checked successfully. The fallible mapping
branch therefore compiles against the pinned Etalon API. Actual mapped-readback
failure/success and hardware canvas behavior remain runtime gates.

The source-policy regressions were repeated after these checks: **15/15 passed**,
and the actual root lock plus retained Cargo-generated
`target/etalon-profiler-metadata.json` passed the source inventory. Targeted
`git diff --check` also passed for the synchronized allocator, crypto, canvas,
rendering diagnostics and constellation source paths.

The actual `servo-script-webcrypto` admission-test command started but was
interrupted with Ctrl-C (exit 1) while compiling dependency libraries to release
the shared Chora target for the standalone worker check. No crypto admission
test result follows from that interrupted run. It is to resume after the worker
check; the WPT DOM import/sign/verify route remains separately unexecuted.

A subsequent independent worker review found that the actual software rendering
context recorded requested dimensions before Surfman resize succeeded and
ignored its Result. The generic fix now updates its size only on successful
swap-chain resize, logs failure and retains the previous size on error, matching
the existing window-context behavior. This source edit follows the passing full
engine/canvas checks above; those passes do not include this later correction.
The standalone worker adds a context.size admission check after WebView::resize.
Its final actual check must be repeated after both edits. Runtime surface-resize
failure and readback behavior remain to be qualified on actual contexts.

The final actual standalone worker check subsequently **passed** (exit 0,
5m41s), including a fresh `servo-paint-api` check after the software resize
correction, its downstream engine graph and the actual worker source. See
[final worker check](resume-chora-worker-check-final.log). Thus the current
size-on-success correction is compiled against actual Surfman and engine APIs.
The touched rendering-context source also passes rustfmt and git diff --check.
No actual surface-resize failure/success runtime result is inferred from this.

Actual crypto unit execution remains explicitly deferred while the same worker
consumer performs its real build/staging/runtime sequence, to preserve the native
build cache. The resumed crypto command should select `--features js/jit` to
match embedded_library's actual SpiderMonkey JIT profile rather than trigger a
second no-JIT native build. Genuine WPT (including dedicated-worker globals) and
counting-provider unit execution remain separate from any supplemental page
checks that execute the unchanged raw-key promise bodies.

The subsequent actual worker **build** stopped before producing a worker executable
at `mozjs_sys 153.3.0-3`: its prebuilt static archive was unavailable to the build
helper, and source fallback reported `MozTools or MozillaBuild not found`.
See [actual build failure](resume-chora-worker-build.log). The final successful
worker check remains compilation/metadata evidence only. No actual browser,
readback or supplemental WebCrypto page result is claimed from that check.
Normal exact-asset/cache recovery is being investigated by the dependency owner;
crypto unit builds remain held because they need the same native link input.
No alternate executable launch, source substitution or OS policy bypass was used.
