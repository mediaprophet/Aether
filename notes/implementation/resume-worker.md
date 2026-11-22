# Separate worker implementation resumed — 14 November 2026

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

The user authorized resuming the saved pause checkpoint. Existing local changes
and immutable qualification captures are preserved. No commits, pushes, branch
creation/renames, co-author trailers or AI bylines were made.

Verified totals: protocol 7, client 14, live host 34, source policy 86;
all pass. The native CPU/client feature compiles, and the real-worker harness
builds. Full read-only verification confirms all 4,382 frozen-v4 inputs and nine
package identities. The actual Chora worker check and full executable build pass;
actual page/pixel runtime remains unexecuted. The consolidated current status,
final staging result and remaining jobs are in
[the 14 November handoff](HANDOFF-2026-11-14.md).
Final cached build passes in54.85s. Actual staging then exits1 because the required
root `LICENSE-QUALIA` is absent; the partial stage has no completed receipt.
The handoff records its exact path/hash and the first remaining acceptance job.
Live integration-profile generation currently rejects QualiaDB's changed Etalon
patch table; frozen native/client composition separately rejects mixed source
identities. Details and exact source paths are in `resume-host.md`.

## Implemented

- Standalone bounded protocol, actual Chora worker, parent process owner and native
  copied-frame widget. Chora remains outside the native dependency graph.
- Explicit parent navigation retires the old view/delegate before creating the
  next generation. Independent review identified and fixed the async
  `load()` callback relabeling race; retired callbacks now cannot touch a new
  view. Close/crash cancels requests and frames promptly.
- Parent session/order checks, one outstanding capture/one latest frame, 64
  retained page requests plus eight control slots, 30-second startup/capture/navigation/resize deadlines and independent
  two-second shutdown/kill/reap. UI teardown only requests shutdown.
- Canonical parent-only read bridge and concrete client pump, with explicit signed
  registration and exact origin/caller/profile/stamp matching. See host packet.
- Worker output pipe is duplicated before threads; Rust and CRT stdout diagnostics
  are redirected to stderr. Only the saved file emits protocol frames.
- Required sandbox/private requests reject before creating rendering/engine state.

## Executed evidence

- Protocol: 7/7 tests in its standalone workspace (host packet has command/log).
- Client: 6/6 unit tests and 2/2 real private-process/pipe tests; full log
  `resume-chora-client-pipe-tests.log`. The process tests verify frame delivery,
  navigation invalidation, wrong-session rejection, nonblocking shutdown and
  reaping an intentionally uncooperative test peer within the deadline. This
  fixture is explicitly not Chora.
- Actual native CPU + prelude + Chora-frame feature check passed in 3m43:
  `cargo check --offline -p aether-native --no-default-features --features vello-cpu-softbuffer,chora,prelude`.
  Log: `resume-native-chora-check.log`. Cargo added only the two local client and
  protocol packages to the root lock. Nested standalone workspaces are explicitly
  excluded from root membership. This is compilation, not pixel/runtime evidence.
- The exact saved post-naming/post-IME hybrid lifecycle executable, SHA256
  `53c6715a9d8836a199ab7b5de746a7088fdae0d27f6640ce71ca6e15c189179e`, ran
  successfully with a 30-second bound. Startup, three redraw callbacks and owner
  teardown passed in 17.646s. Receipt `resume-hybrid-runtime-20261114.json` and
  matching stdout/stderr logs retain executable identity. This qualifies that
  saved profile's lifecycle only, not the new Chora widget or pixels.
- Full Chora embedded-engine check, current production-source identity fixture
  (15 tests), and canvas with explicit optional profiler check passed. Exact
  commands/results remain in `resume-chora.md`.
- Worker source gate passes against real metadata and a lock naturally regenerated
  from Chora's current lock; zero incidental dependency identity drift and exact
  reviewed Etalon8e8 sources. See `resume-dependencies.md` for final receipts.

## Open verification

Actual worker compile/build/runtime and current canonical-host tests are running
or pending at this entry; no result is inferred. Internal same-view navigation
still lacks a public document-to-paint commit identity; URL notifications alone
are not proof of a particular pipeline's pixels. Input forwarding is not yet
implemented. Readback alpha/orientation, installed artifacts, OS containment,
canonical frozen-v4 execution, signed native fixture and release/hardware matrix
remain distinct gates. This is not production qualification.

Final client verification includes the actual worker `private_pipe.rs` helper,
imported by the explicit test-only peer. Both children emitted stdout diagnostics
after redirection while protocol frames remained parseable; all six unit and two
process tests passed (4.32s process execution). Earlier rebuild attempts hit
MSVC LNK1104 while opening generated test executables. Those failures are retained
in `resume-chora-client-tests*.log`; they are not passing results and no security
settings or executable permissions were changed. The final source adds the shared
stdio-helper test and its test-only dependencies; its successful normal Cargo run
is recorded in `resume-chora-client-pipe-tests.log`.

The root GPU source policy passes after adding the transport dependencies, and
all 55 source/snapshot policy regressions pass (`resume-root-gpu-policy-20261114.log`
and `resume-policy-tests-20261114.log`).

Actual native-profile metadata confirms client/protocol are present and neither
`servo` nor `aether-chora-worker` is linked. Full graph data is retained under
`target/production-audit/resume-worker-native-metadata.json`.
A real-engine loopback harness is provided as the client's `worker_readback`
example: it expects an actual engine-derived request, denies it, and requires the
page's JavaScript rejection handler to produce a known opaque 64x64 pixel image.
Its compilation/execution results must be recorded separately from the pipe peer.

The final client library run passes 7/7 tests, including synchronous escaped-reply
wire-size rejection without queueing or closing the client. Exact log:
`resume-chora-client-final-lib-tests-retry.log`. The final native CPU feature
check also passes after this change (`resume-native-chora-final-check.log`).
A separate worker-side input watchdog now covers parent disappearance while the
engine thread is stuck: EOF or bounded queue overload starts an independent
two-second process-exit deadline. Its direct-child regression is being verified
separately, using the actual shared worker input helper rather than a replica.
The real-engine loopback harness compiles (`resume-chora-readback-harness-check.log`);
its execution remains pending the actual worker executable.

The final direct-child process suite passes 3/3 in 6.17s, including the real worker
input helper terminating an intentionally stuck child after parent-pipe EOF,
without a ChoraClient monitor. Together with the final 7/7 library tests this is
10 executed client tests. Exact process log:
`resume-chora-client-eof-tests-retry.log`. Its preceding ordinary Cargo attempt
hit MSVC LNK1104 opening the generated process-test executable; the same command
and same target then passed, with no permission/security changes.

Final queue/cleanup verification now passes 8/8 library plus 3/3 process tests
(`resume-chora-client-queue-tests-retry-2.log`, process execution 6.51s). A complete
64-request burst fits alongside eight bounded lifecycle/control messages; parent
retained PageRequest payloads remain capped at64. The worker rejects pixel frames
before its input queue, and its exit watchdog performs no potentially blocking
logging. The EOF regression closes stderr after a bounded real handshake and
still verifies independent worker termination. A normal same-command retry was
needed after the retained LNK1104 linker attempt; no security changes were made.

A read-only follow-up found that the previous source-capture generator does not
include the newly added host client/protocol path packages. That generator/profile
closure is being updated in its own packet before any new qualification receipt
is accepted. Existing immutable v4 remains unchanged, and the current live host
run is not being relabeled as frozen qualification.

Current live host integration now passes all 34 library tests with the concrete
`chora-client` pump feature enabled (8.68s test execution). The actual current
canonical core and Vibe dependencies compiled; signed bridge tests executed.
Log: `target/production-audit/resume-chora-live-host-tests-standalone.log`.
This does not execute canonical dependency unit tests, qualify frozen-v4 inputs,
or run the signed native window fixture. The earlier duplicate root-target
attempt stopped before tests; the confirmed standalone cache was used for this
successful run. The immutable v4 receipt still has its original SHA256.

Windows GUI-parent launch review found that inherited stderr may be NULL. The
client now creates its own stderr pipe and continuously drains to a sink with
bounded memory; no parent console/log sink can backpressure the worker. Diagnostics
are deliberately not retained by this initial client. A synchronous 256 KiB
fixture diagnostic burst before navigation acknowledgement proves continued drain.
All 8 library and 3 process tests pass after this correction, plus doc tests:
`resume-chora-client-stderr-tests.log` (process execution 11.61s, serial tests).
This run used the standalone `packages/chora-client/target` cache; earlier client
runs used `target/chora-client`. The independent engine reviewer found no further
lifecycle or memory-bound issue in the drain.

The final native CPU/prelude/client feature check passes after the GUI stderr
correction (32.02s): `resume-native-chora-stderr-check.log`. The real-engine
`worker_readback` example also builds successfully (19.53s):
`resume-chora-readback-harness-build.log`. The executable is ready under
`target/chora-client/debug/examples`; no actual engine runtime result follows
until it runs against the staged current worker.

A final startup-route review confirms Ready must match the current full stamp,
including after any prior valid Invalidate. Actual worker writes Ready before its
first event-loop spin; no current callback requires mismatched readiness stamps.
Client now enforces that equality. Nine library plus three private-process tests
pass (`resume-chora-client-ready-tests.log`, process execution11.48s), including
rejection of mismatched startup generation/document/viewport. This is protocol
consistency, not a claimed observed authority exploit.

Actual standalone worker check now passes (Cargo exit0,5m41s):
`resume-chora-worker-check-final.log`. It includes the current generic Chora
software-resize failure fix and actual worker API corrections: structured
Rendering failure for Surfman make_current errors, and Box2D construction via
from_origin_and_size. Initial check's two compiler errors remain recorded in
`resume-chora-worker-check.log`. Current executable build is separate; successful
checking does not establish linking, launch, bridge delivery or pixels.

Further lifecycle review fixed terminal status and image cleanup. Closing client
state is idempotent, preserving the one Failed/Stopped event across monitor
shutdown and reaping. Ready operations now return Closed after shutdown or peer
closure instead of indefinite Busy, including unchanged-size resize. The final
client suite passes11 library+3 process tests (`resume-chora-client-closed-tests.log`,
process execution8.70s). The harness consumes terminal errors before checking exit.
The native widget requests one final redraw while an old image remains and clears
it when the client closes/exits, before resize can return Busy. Native compilation
of this change is being verified separately; no terminal-window runtime proof is
inferred from source review.

Final lifecycle sources pass the actual native CPU/prelude/client check in51.46s
(`resume-native-chora-terminal-check.log`). The updated baseline/supplemental
readback harness builds in18.32s (`resume-chora-readback-closed-build.log`). An
independent review confirms terminal-event retention, Closed readiness operations
and final native image-erasure scheduling; actual terminal-window pixels remain
unqualified. Worker executable build is separate and remains in progress.

Final reviewed-overlay/source policy suite passes86/86 (64.759s). Full immutable
v4 verification still passes4382 inputs/nine identities with unchanged receipt.
Only two reviewed non-GPU overlays have a new explicit source-verified path: all
143 original overlay files are anchored, IO gets a25-file standalone derivation
from pinned6ace metadata and notices, and write ancestors are checked before
staging. An isolated real offline/locked Cargo metadata run qualifies this narrow
mechanism; it does not qualify the complete live SDK/native graph. During this
run the external Qualia patch table expanded to14 entries including GPU packages.
Actual live generation rejects that broader table before writing output. See
`resume-host.md` for exact paths, review digest, evidence and source limits.

Hard-deadline cleanup review changed the worker's independent watchdog to
TerminateProcess(GetCurrentProcess(),1) on Windows and libc::_exit(1) on Unix,
with an abort fallback, rather than std::process::exit. Rust1.99.1 runtime cleanup
and Windows ExitProcess DLL detach can wait on locks held by a stuck engine;
the watchdog must bypass those callbacks. It targets only its own process and
performs no logging. See Microsoft's
[ExitProcess](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-exitprocess)
and [TerminateProcess](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess)
documentation.
The actual shared-helper suite passes all11 library+3 process tests after the
change (`resume-chora-client-hard-exit-tests.log`, process execution10.34s).
This is not a Windows DLL-detach fixture or Unix execution claim. Worker metadata
already enabled Win32_System_Threading; the explicit manifest declaration adds no
resolved package/feature. Current full build will compile this helper before stage.
