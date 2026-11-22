# WP08 / WP16 preparation: native behavior and distribution

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
Preserve inherited Dioxus and third-party attribution.

## Scope and evidence rules

Prepared on 15 November 2026 from Aether-local documentation and source only.
This document specifies future assignments. No Cargo, build, test, GUI, device,
network, sibling-repository inspection, signing or publication was performed in
this preparation. Existing source, logs, captures and artifacts were preserved.
The parent owns the register, architecture release gates and integrated summary.

Authority: `AGENTS.md`, `notes/implementation/WORK-PACKETS-2026-11-15.md`
(WP08 and WP16), `notes/implementation/resume-native.md`, and
`notes/implementation/native-input.md`. Paths below are relative to Aether.
The latter input note includes an old missing-dependency failure; the recovery
note explicitly supersedes that failure for the recorded Windows CPU profiles.
Neither account is current-source execution evidence.

**HELD** means execution is not released by this preparation. All
dependency-consuming reruns require WP01 preservation/harness preparation and
WP03 acceptance after the relevant WP02 user-notified intake. No pin changes or
adaptation to moving sibling APIs is authorized. A missing fixture is future
assigned work, not a runnable test. SUP must assign exact implementation files,
source hashes, one Cargo owner/cache/profile, platform operator and deadline
before execution. Roles below are proposed execution owners, not active agents.

For every executed row retain: scenario ID, source/lock/capture identity,
target triple, Rust/tool versions, exact features and resolved Etalon/Hypermedia
sources, command, exit code, start/end time, bounded stdout/stderr, artifact hashes,
and observed assertions. Hardware rows also need OS/build, display scale,
adapter/driver, input method/device and operator actions. Record unavailable,
skipped, denied, timed-out and failed separately from passed. A headless editor
test cannot close an OS row; a WebView bundle cannot close a native row.

## Earlier evidence to retain, without promotion

`resume-native.md` records the following earlier outcomes; they were not rerun
or newly certified here:

| Evidence | Recorded result | Boundary that remains open |
| --- | --- | --- |
| `resume-assets.log` | Actual Windows resolver: 8 library and 2 API tests; junction replacement separately passed | Unix execution, Android loader, installed roots, decoder/aggregate memory |
| `resume-dom-clipboard-final.log` | Four actual editor clipboard tests | OS access/permission behavior and application clipboard events |
| `resume-native-ime-editor.log`, `resume-native-dom-ime-final.log` | Six editor regressions; 21 native DOM tests | Actual platform IME and candidate geometry |
| `resume-native-smoke-runtime.log`, `resume-native-multiwindow-runtime-final.log` | Windows CPU softbuffer/SVG/system-fonts/prelude startup/redraw/owner-drop; auxiliary window continues after first closes | Pixels, GPU/device recovery, suspend/resume, scale, installed application and canonical foundation |
| `resume-native-hybrid-runtime.log` | Default hybrid lifecycle probe against the recorded shipping sources | Physical adapter identification, pixels and failure recovery |
| `resume-native-vello-build-receipt.json`, `resume-native-vello-smoke-build.log` | Vello executable built; source gate recorded | Runtime was blocked by Windows Application Control; automatic-review retries timed out before dispatch. No Vello runtime pass |
| `native-input-tests.log`, `native-input-probe/README.md` | Earlier independently locked headless 20-test probe | Shipping lock, GPU/window, minimum Rust and packaging; later root DOM 21-test receipt remains distinct |

The recovery note records 5 Blitz trait, 2 network, 2 DOM completion and 9 native
CPU library tests. These counts describe earlier component evidence only.

## WP08 input, editor and accessibility fixture assignments

All rows are HELD for execution under the prerequisite rule above. Real OS rows
are needed independently on Windows, macOS and Linux for every claimed native
target; Android/iOS rows must be added before mobile claims. Each OS run should
use a disposable text/data fixture and bounded event trace, never personal
clipboard contents. RUST owns fixture work; a platform operator owns delivery;
REVIEW checks unsupported conversion containment.

| ID / existing executable source | Setup and observable assertions | Failure variants | Acceptance evidence / remaining work |
| --- | --- | --- | --- |
| I01: `packages/native-dom/src/events.rs`: `ime_preedit_clear_and_commit_preserve_text_and_phase_order`, `ime_availability_empty_preedit_and_direct_commit_do_not_invent_composition`; `packages/native-dom/src/aether_document.rs`: `ime_composition_routes_to_typed_handlers_and_commits_input_once` | Headless layout before delivery; enable, nonempty preedit, updates, empty clear, commit. Exact text/phase order and one actual input per editor commit; availability/direct commit must not fabricate sessions | Repeated disable, empty preedit, prevented compositionend default | Test-name/output receipt closes routing only. Real-window event recorder is **missing**, assign WP08 input slice to create it after WP03 |
| I02: `events.rs`: `ime_disable_and_focus_loss_cancel_once_without_leaking_between_targets`, `ime_target_switch_ends_previous_target_and_teardown_clears_stale_state`; `mutation_writer.rs`: `composition_listener_aliases_share_the_native_ime_counter` | Switch focused target during composition, remove target, reuse component slot; one cancellation on delivered blur/disable, no removed-owner callbacks, listener counts balanced | Target disappears before commit; duplicate disable; new target direct commit | Headless assertions plus future OS trace. Programmatic `NodeHandle::set_focus` does not currently queue focus/blur; do not count it as OS focus coverage |
| I03: `vendor/blitz/blitz-dom/src/node/text_ime_tests.rs`: `surrounding_delete_preserves_unicode_selection_and_direction`, `surrounding_delete_rejects_split_utf8_on_either_side_atomically`, `surrounding_delete_clamps_large_lengths_without_deleting_selection`, `surrounding_delete_preserves_preedit_cursor_and_later_commit`, `surrounding_delete_preserves_preedit_across_a_combining_cluster_boundary`, `rejected_surrounding_delete_preserves_active_composition`; `events.rs`: `surrounding_delete_does_not_end_or_duplicate_composition` | Seed Unicode/reversed selections and combining preedit; valid surrounding deletion preserves selected/preedit bytes and remaps endpoints; effective deletion emits input once | Split UTF-8 on either endpoint, oversized/zero request, invalid request during composition; no partial mutation or duplicate event | Existing editor tests, earlier 6/6 receipt. OS-generated DeleteSurrounding evidence missing; RUST/platform operator must capture actual supported IME delivery |
| I04: no real OS IME fixture identified; routing sources above are starting points | Future window with two editors; named installed IME composes CJK and combining Unicode; record native cursor byte ranges, application text, composing keyboard flags and candidate-window placement before/after scale/focus changes | Cancel, blur, target removal, commit without preedit, IME disable, resize while composing | OS trace plus screenshots/video and exact expected committed text. CompositionData exposes text only; do not invent range fields or claim application cursor-range support. HELD on missing fixture, available IME/platform and WP03 |
| I05: `vendor/blitz/blitz-dom/src/node/text_clipboard_tests.rs`: `copy_preserves_selection_and_never_emits_input`, `unavailable_clipboard_cannot_destroy_cut_or_paste_selection`, `successful_cut_and_paste_emit_one_edit_result_each`, `empty_clipboard_does_not_delete_selection` | Disposable Unicode selection; copy leaves text/selection and emits no edit; cut deletes after accepted write; paste emits one edit | Failed read/write, empty clipboard, uppercase shortcut; text/selection unchanged on denied access | Existing editor tests use a test ShellProvider, not an OS clipboard. Future OS fixture missing; platform operator must verify named OS backend and permission/unavailability outcomes using known disposable text |
| I06: `events.rs`: `unsupported_native_events_report_capability_errors_without_fake_data`; `native_event_support` and checked dispatch | Request copy/cut/paste, drag/drop and beforeinput capability; explicit unsupported result and no checked dispatcher route into infallible unsupported conversion stubs | Listener aliases, mismatched payload, unknown family; review reachable conversion paths | Existing negative capability test plus source review. Application clipboard/data transfer, drag and cancellable beforeinput emission fixtures are **missing** and require backend work before positive qualification; no fabricated payload acceptance |
| I07: `events.rs`: `typed_composition_and_pointer_converters_preserve_real_payloads`, `touches_reports_all_active_pointers`, `touches_is_empty_when_no_pointers_are_active` | Known pointer IDs/coordinates; active and changed contacts preserved; future OS window logs mouse, pen, touch, wheel and focus routing across overlapping targets | Cancel/lost contact, two fingers across targets, scale change, button transitions, removed target | Headless converter evidence separate from real-device traces. Multi-input OS fixture missing. `targetTouches`, default geometry/pressure and identity assumptions require explicit comparison with actual supported behavior |
| I08: accessibility feature/source entry points documented in `packages/native-dom/src/lib.rs`, `packages/native/src/lib.rs`; no assistive-technology acceptance fixture identified | Future accessible form with label, edit, button and focus order; platform operator uses named AT to inspect semantics and perform actions; compare visible value/focus with AT observation | Removed/disabled target, focus migration, scale/window transition; no stale action delivery | AT interaction recording and action/result trace per OS. Feature compilation is insufficient. HELD on missing fixture, operator/tool availability and WP03; assign WP08 DOM/accessibility slice |

## WP08 assets and terminal completion assignments

RUST asset slice owns resolver/transport fixture work; platform operators own
Unix/Android execution. Each row is HELD on WP01/WP03; unavailable platform rows
stay open. REVIEW checks handle/FFI containment changes independently.

| ID / existing source or fixture | Setup and observable assertions | Failure variants | Acceptance evidence and platform |
| --- | --- | --- | --- |
| A01: `packages/asset-resolver/src/asset_policy.rs`: `rejects_traversal_and_platform_path_escapes`, `malformed_encoding_never_panics`, `resolves_only_files_inside_the_bundle`, `manganis_absolute_development_urls_require_trusted_access`, `packaged_read_uses_the_resolved_file`; `tests/native_assets.rs`: `trusted_application_sources_remain_explicit`, `malformed_asset_url_is_not_a_panic` | Temporary bundle root with known bytes; packaged reads remain inside configured root; trusted development access explicit | Encoded traversal, malformed escapes, absolute/drive/UNC input and missing file | Per-test results on each OS; installed-root row D02 still required |
| A02: `asset_policy.rs`: `per_request_limit_rejects_large_assets_and_accepts_exact_boundary`; `packages/native/src/asset_transport_tests.rs`: `explicit_host_byte_limit_is_applied_to_trusted_asset_requests` | Known file at zero/exact/over admitted request limit; compare bytes and MIME/body from same bounded read | Growth after metadata check, empty asset, oversize; explicit terminal error, no unbounded read | Existing boundary fixture plus future deterministic growth fixture if not covered by assigned test; Windows/macOS/Linux. Limits do not qualify aggregate/decoder memory |
| A03: `asset_policy.rs`: `rejects_symlink_replacement_after_path_resolution`, `rejects_directory_replacement_after_path_resolution`, `permits_internal_symlinks_resolved_before_opening`, `rejects_symlink_escape`; Unix `open_beneath` implementation | Unix temporary root, resolve then replace leaf/directory with escape symlink; retained descriptors and no-follow opens reject replacement; allowed pre-resolved internal link stays contained | Directory swapped, nonregular/nonblocking leaf, outside-root link | Actual Linux and macOS test receipts needed. Earlier note reports CI configuration, not a local Unix pass; WSL distribution inventory does not supply Unix execution evidence |
| A04: `asset_policy.rs`: `handle_check_rejects_a_file_outside_the_captured_root`, ignored Windows `rejects_junction_replacement_between_resolution_and_open` | Explicitly run junction fixture; replacement between resolution/open cannot serve outside root | Junction creation unavailable/denied must be blocked or failed, never passed | Windows operator and exact ignored-test result; retain earlier separate Windows receipt without implying new success |
| A05: `packages/asset-resolver/src/native.rs`: `to_java_load_asset_bounded`, debug cache branch | Future Android harness packages zero/exact/overlimit known assets; release AssetManager and debug cache paths return matching bounded bytes; oversize error distinct from absence | Missing asset, malformed path/NUL, unavailable VM/context/asset buffer; debug cache escape; no panic or unchecked copy | No Android loader executable test identified. Assign WP08 asset slice to add device-capable fixture; Android operator records APK/build mode/device/JNI outcome. HELD also on missing fixture/Android environment |
| A06: `packages/native/src/asset_transport_tests.rs`: `failed_assets_complete_with_status_instead_of_bytes`, `denied_unavailable_and_cancelled_requests_complete_once`, `legacy_transport_drop_finishes_the_pending_request`; `vendor/blitz/blitz-dom/src/net_completion_tests.rs`: `failed_resource_releases_critical_render_wait`, `failed_or_malformed_import_reaches_a_terminal_cssom_state` | Deliver missing/denied/cancelled resources and malformed imports; terminal handler once, critical stylesheet wait released, refused CSSOM state | Transport drops handler, late success after cancellation, failing import graph | Component test logs plus future native app resource-failure trace. Legacy handlers may retain no-op error callback; no claim that external handler pending work is automatically repaired |

## WP08 window lifecycle assignments

| ID / existing source | Setup / assertions / failure variants | Execution owner and acceptance evidence | Status |
| --- | --- | --- | --- |
| L01: `packages/native/examples/native_lifecycle_smoke.rs` | Starts window, requests three redraw callbacks, exits event loop, asserts component owner dropped. Startup failure/deadline must not pass | RUST/platform operator on each claimed desktop OS/profile; hash, runner receipt, redraw/owner-drop output and exit code | HELD WP01/WP03; earlier Windows CPU and hybrid evidence above. This fixture does not assert pixels |
| L02: `packages/native/examples/native_multiwindow_smoke.rs` | Adds late second window after first redraw; both receive document/history/shell contexts; closes first after second starts; second redraws at least three times and closes; both owners dropped. Occluded first window must not be required to keep redrawing | RUST/platform operator per OS/profile; per-window trace and exit result. Reversed close, pending-work close and repeated open/close variants require future assigned fixtures | HELD WP01/WP03; earlier Windows CPU success only |
| L03: multiwindow example forwards `resumed`, `suspended`, `destroy_surfaces`, `can_create_surfaces`; no scenario assertions for those transitions | Future fixture drives actual platform suspend/resume/surface destruction repeatedly, then valid redraw and one teardown; minimize/restore, zero size, scale/monitor transfer while composing and pending auxiliary window. No stale surface/use-after-close or callbacks after owner drop | WP08 window RUST + GPU for renderer lifetime; operator on desktop/mobile. Timestamped lifecycle trace, named display/device and controlled post-resume pixel assertion; failure/device loss also WP14 | HELD WP03 and missing transition/pixel fixture; forwarding methods alone are not evidence |
| L04: `packages/native/tests/launch_errors.rs`: `runtime_failure_preserves_io_error_details`, `event_loop_failures_preserve_stage_and_winit_error`, `legacy_and_fallible_entry_points_have_distinct_return_contracts` | Headless error/source chain and API return contract; future actual denied startup probe must retain error stage and bounded completion | RUST, existing unit receipts separate from actual launch result | HELD WP03; no window/GPU created by these tests |

## WP16 packaging and installed-native target assignments

SUP selects supported architectures; rows below are candidate qualification cells,
not newly declared support. Each claimed architecture is a separate cell. Native
release acceptance requires WP08 for that platform plus WP13 foundation/media,
WP14 renderer recovery/budgets and WP15 compatibility/containment results.
QualiaDB is required; VibeScript uses canonical host/SDK contracts. Installed
launch must exercise that real foundation rather than a UI-only smoke example.

| ID / target and current local source | Setup and observable assertions | Failure variants / acceptance evidence | Owner / prerequisite and status |
| --- | --- | --- | --- |
| D01: dependency/source/build receipt, all targets; `.github/workflows/main.yml` `native-profiles`, `packages/native/tests/renderer_feature_matrix.py` | Record intended renderer/features and actual resolved sources, target, minimum-Rust result independently; build selected native profile from accepted sources and archive receipt | Mixed live/captured ABI clients, upstream GPU source, missing native libraries, unavailable MSRV; explicit failure. Feature-matrix Python uses small backend contracts and cannot qualify upstream API or GPU | REC + RUST, SUP source acceptance; HELD WP01/WP03 |
| D02: installed assets, fonts, libraries and notices; resolver `native.rs::linux_bundle_discovery_never_selects_an_unrelated_application`; packaging source `packages/cli/src/bundler/{mod,windows,macos,linux}.rs`, `build/assets.rs` | Future native fixture bundle launched from installation root and unrelated working directory, paths containing spaces/non-ASCII. Verify known stylesheet/image/font bytes, intended libraries/worker location and fallback behavior. Manifest hashes every shipped file and associates complete transitive notices with exact source identity | Missing font/library/worker/notice, unrelated lib-directory sibling, read-only install directory; diagnostic and bounded refusal, never unrelated asset fallback. Require authoritative notice resolution from WP01; no invented license text | WP16 RUST + REC, per-platform operator; HELD WP01/WP03, missing native installed fixture; WP06 for selected Chora stage |
| D03: Windows candidate `x86_64-pc-windows-msvc`, separately `aarch64-pc-windows-msvc`; existing `main.yml::windows-bundle-smoke` creates MSI/NSIS for desktop hotdog | Future native installer on clean disposable machine; ordinary user launch, required native foundation/media profile, WP08 input/windows/assets, uninstall/reinstall without losing application data | Missing runtime DLL, denied install/start, interrupted update, corrupt asset. Retain installer/exe hashes, file inventory, machine baseline, install logs, actual native backend/adapter and application assertions | WP16 RUST + Windows operator; HELD native packaging fixture/profile and upstream gates. Existing desktop bundle generation is compatibility evidence only |
| D04: macOS candidates `aarch64-apple-darwin` and `x86_64-apple-darwin`; `main.yml::bundle-macos-smoke` creates desktop `.app`/DMG | Future native app installed on named clean OS/architecture; verify bundle resources/library resolution, real launch, required foundation/media and WP08 including named IME/AT | Missing library/font, unsigned/denied launch, interrupted update/recovery, scale/suspend. App/DMG hashes, architecture inspection, install and platform traces | WP16 RUST + macOS operator; HELD profile/fixture/platform. Signing/notarization only with available authorized credentials; none used here |
| D05: Linux candidates `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`; `main.yml::bundle-linux-smoke` creates desktop AppImage/DEB/RPM | Future native bundle per chosen distribution/session; clean-machine dependency baseline, installed root deterministic, required foundation/media and WP08 on named compositor/session | Missing runtime/font/system library, unrelated bundle lib directory, permissions, corrupt update; package/file hashes, dependency inventory, install/start/uninstall and recovery trace | WP16 RUST + Linux operator; HELD profile/fixture/platform. WSL inventory and desktop package existence do not close a native installed cell |
| D06: Android candidate `aarch64-linux-android`; `main.yml::matrix_test` builds `aether-desktop`, `packages/cli/src/build/android.rs`, resolver JNI path | Future actual native app packaging path must first be selected and verified; on device, required foundation/media plus A05, IME/touch, activity/surface pause/resume and data persistence | Missing assets/JNI context, denied permissions, background/foreground, update failure, device loss; APK/source hashes, device/OS/ABI and bounded lifecycle trace | WP16 RUST + Android operator; HELD native packaging route/fixture, SDK/device and WP03/WP08/WP13–15. Current cross-build is WebView package evidence, not native mobile support |
| D07: iOS candidate `aarch64-apple-ios` and separately any chosen simulator target; `main.yml::matrix_test` builds `aether-desktop`, `build/apple.rs`, `bundler/ios.rs` | Future verified native packaging route on simulator then named device; required foundation/media, keyboard/IME, touch/AT, assets and app/surface resume; preserve data across authorized update | Missing resources/provisioning, denied launch, interruption/background, revoked authority after resume; artifact/device/source and interaction receipts | WP16 RUST + Apple operator; HELD missing native fixture/route, SDK/device/provisioning and upstream gates. Simulator/cross-build cannot close physical-device acceptance |
| D08: update/recovery all claimed targets; `packages/cli/src/bundler/updater.rs` is packaging source, no installed native data-preservation fixture identified | Future disposable canonical data store; install version A, produce durable authorized state, update to B, reopen and compare expected state; update failure leaves recoverable prior data | Truncated/wrong-target update, unavailable storage, interruption, incompatible artifact and rollback; before/after durable state receipts, artifact identities and terminal outcome | WP16 RUST + platform operator; HELD WP04 durability/WP13 and missing installed update fixture. Presence of updater packaging code does not establish runtime migration/recovery |
| D09: signing/notice/reproducibility all shipped targets; `.github/workflows/publish.yml` is **Publish CLI** with six target triples | Future release preparation inventories exact binaries, libraries, worker assets, fonts and authoritative transitive notices; record unsigned artifact and reproducible source/build receipt separately from signing. Verify selected application artifact signature with named tool when credentials exist | Missing notice, unknown source revision, unavailable signing identity or nonmatching artifact hash remains blocked; credentials never copied into receipts | REC + WP16 RUST; HELD WP01 notice decision/WP03/staging and credentials where required. CLI release matrix is not installed application qualification; no dispatch, signing or publication authorized here |

`packages/cli/src/platform.rs` documents desktop/mobile aliases as WebView
renderer selections. Existing bundle-smoke jobs invoke `--platform desktop` and
verify output existence, which is useful compatibility packaging preparation but
cannot be relabeled as native distribution. Establish the actual supported native
packaging route before proposing a native bundle command. No speculative CLI
switch is introduced here.

## CI acceptance gaps and future bounded recipes

`main.yml` configures `native-assets` on Ubuntu/macOS/Windows and
`native-profiles` on Windows. Its five profiles are default-hybrid, gpu-vello,
cpu-softbuffer, cpu-minimal and default-plus-cpu. It configures editor/network/DOM
tests and the renderer selection/watchdog checks. These are source observations,
not evidence that a workflow ran. Bundle smoke jobs inspect desktop artifacts;
no native installed launch/update fixture was identified. The cross matrix's iOS
and Android cells explicitly build `aether-desktop`.

WP16 CI owner should add future fixture results only after their implementations
exist. Preserve prerequisite state: path-filtered/draft skips, missing SDK/GPU/IME,
unavailable credentials, policy-denied process creation and runtime timeouts must
never become a generic green native qualification. Keep component, compilation,
actual window, installed app and physical device result classes distinct.

The following are code-supported **future recipes**, not commands executed here.
They remain HELD; SUP chooses the existing accepted target/cache and run owner,
with an outer command deadline where the command itself lacks one. Do not create
a fresh target or change pins to evade a failure.

```powershell
# Existing independently locked headless source probe; not the shipping graph.
./notes/implementation/native-input-probe/verify.ps1

# Actual resolver tests as configured in CI, after WP03 releases this profile.
cargo test --locked -p aether-asset-resolver --features native --lib --test native_assets
cargo test --locked -p aether-asset-resolver --features native --lib rejects_junction_replacement_between_resolution_and_open -- --ignored

# Existing attributed component fixtures, with the accepted root graph.
cargo test --locked --lib -p blitz-dom clipboard_tests
cargo test --locked --lib -p blitz-dom ime_tests
cargo test --locked --lib -p blitz-dom completion_tests
```

For Windows real-window runs, use an **already built, reviewed** executable
inside Aether, identified from the accepted build receipt. The runner requires
PowerShell 7, `.exe` paths and receipt paths inside this workspace; its timeout
range is 1–600 seconds. Example syntax with caller-selected real paths:

```powershell
# Set these to receipt-verified paths/profile; do not guess an executable path.
./scripts/run_native_probe.ps1 -Executable $acceptedProbeExecutable -LogPrefix $newReceiptPrefix -ProbeProfile $acceptedProbeProfile -TimeoutSeconds 90
```

Use this only for the existing `native_lifecycle_smoke` or
`native_multiwindow_smoke` behavior described above. It drains stdout/stderr,
hashes the executable, bounds process and capture duration, and stops only its
owned process tree. Exit 124 is timeout; 125 is incomplete capture. Process
creation denial is a blocked run, not a pass; it may occur before a JSON receipt
is written, so preserve the launch error independently. The runner is not a
sandbox. macOS/Linux need an assigned platform-appropriate bounded runner before
runtime scheduling; the Windows `.exe` wrapper must not be advertised as portable.

No executable recipe is supplied for missing OS input, Android loader, lifecycle
transition, installed native bundle or update/recovery fixtures. Their respective
WP08/WP16 owners must implement and review those fixtures before running them.
