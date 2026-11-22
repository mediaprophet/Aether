# Native recovery implementation — 12 November 2026

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

## Asset boundary

Packaged asset reads now have a 64 MiB default and public per-request limit APIs.
Native applications can select their admitted limit with
`Config::with_asset_max_bytes`; large authorized assets are not permanently
restricted to the default.
Metadata rejects already-oversized files; a bounded read also catches growth
after that check. Exact-boundary and zero-byte behavior have regression coverage.
Trusted-file serving uses the same bounded bytes for MIME inference and response,
avoiding a second pathname open. Android buffer length is checked before copying;
its debug asset cache uses the checked packaged-file path.

Unix opens each canonical descendant through retained directory descriptors,
using no-follow flags at each component and nonblocking leaf opens. Symlink
replacement and directory-to-symlink replacement tests are present. The trusted
root and its ancestors must remain stable. This does not authorize arbitrary
origins, prevent owner-installed hardlinks, freeze file contents, or impose an
aggregate/decoder memory budget. Ubuntu has now been installed through WSL and
its distribution inventory is readable. Execution inside Ubuntu remains blocked:
automatic approval review timed out twice before the read-only command started.
Linux/macOS CI runs the actual library tests; no local Unix pass is claimed.

Executed against the actual resolver: seven Windows path/limit tests and two
public API tests passed. The explicit Windows junction-swap fixture also passed
in a separate run; the ordinary suite continues to exclude it. The final actual
resolver run passes 8 library and 2 API tests, including deterministic
Linux bundle-root selection. Root discovery
selects only the configured product or executable stem, never an unrelated
directory under the installation's lib folder.
The isolated fixture lock was naturally refreshed by Cargo and retained under
`scripts/contract-locks/assets.lock`. Three document lifecycle tests also passed.
See [full workspace asset log](resume-assets.log).

## Terminal network completion

The published Blitz `NetHandler` contract could deliver bytes but could not report
failure or cancellation. Native errors therefore left critical stylesheets pending.
The exact inherited packages are retained with provenance and license texts under
`vendor/blitz`, and selected explicitly through Cargo patches.

The local changes add typed terminal errors, cancellation futures with per-waiter
unregistration, terminal completion when transports drop requests, and rejection
of late success after observed cancellation. Wakers run/drop outside their mutex.
All public Blitz network request entrypoints use the same cancellation wrapper.
DOM resource failures clear pending critical state; failed/malformed imports are
set to a terminal refused state on the document thread.

Independent source review found and corrected the initially missing async/callback
cancellation and pending-import cases. Tests cover these behaviors using the actual
libraries, alongside native delegated-transport failures. Actual results now pass:
5 Blitz trait tests, 2 network tests, 2 DOM completion tests and 9 native CPU
library tests. The latter include asset authority/limit/transport cases and callback
registry lifecycle. Legacy third-party handlers retain
a default no-op error method for compatibility; handlers that track their own
pending work must implement that method.

## Clipboard review

The actual native text editor emitted an input notification for copying and
deleted selected text when cut/paste clipboard access failed. The attributed
Blitz patch now leaves text/selection intact on failed access, emits no edit for
copy, handles uppercase shortcut characters, and treats an empty clipboard as
no paste. Successful cut deletes only after the clipboard accepts its text.
Four real editor tests exercise successful/failed access, Unicode selection,
event results and empty clipboard behavior and now pass against actual Blitz
([receipt](resume-dom-clipboard-final.log)). This does not
add application clipboard events or claim OS clipboard permission qualification.

## Runtime qualification

The actual default native workspace build passed before the network changes, after
dependency recovery. It is a baseline, not a post-change test result. The new
bounded native lifecycle example exercises window startup, redraw dispatch and
component teardown; it does not certify pixels, multiple windows, device loss,
input, packaging, or the required QualiaDB application profile. A separate
`native_multiwindow_smoke` probe adds an auxiliary window after the first redraw,
requires per-window document/history/shell contexts, closes the first window,
then requires the second to dispatch and close with both component owners dropped.
Both CPU examples now build and run successfully. The single-window run observes
three redraw callbacks and owner teardown; the auxiliary-window run observes
window 0 redraw, late window 1 creation, owner 0 teardown, continued window 1
redraws and owner 1 teardown, exiting successfully. The first auxiliary probe
timed out; its revised sequence closes the first window after the second starts,
without relying on redraw delivery to a potentially occluded first window.
Receipts: [single window](resume-native-smoke-runtime.log) and
[auxiliary window](resume-native-multiwindow-runtime-final.log). These are actual
Windows CPU softbuffer/SVG/system-fonts/prelude lifecycle runs, not pixel,
GPU-device, suspend/resume, installed or canonical application qualification.
The actual patched native DOM library suite also passes all 20 tests.

The default hybrid/prelude profile subsequently built against exact shipping
Etalon575/hypermediaa46 sources and passed the same three-redraw/owner-drop probe:
[hybrid runtime receipt](resume-native-hybrid-runtime.log). This validates that
selected renderer's lifecycle path, not physical adapter identity, pixel output,
or device-loss recovery. The independent Vello GPU compilation also passes after
the attributed shader-error and identity-swizzle fixes in `vendor/hypermedia`.
## Bounded native probe execution

`scripts/run_native_probe.ps1` launches a workspace-owned probe with an explicit
deadline, drains both output streams and records the executable SHA-256, selected
profile and process outcome. It stops only the process it launched when the
deadline expires. The two final Windows tests passed after the parameter-name
correction: normal completion with both streams and hash receipt, and termination
of a deliberately sleeping child with timeout exit 124 (13.556 seconds total).
Output capture has a separate deadline. This is a test watchdog, not a sandbox or
proof of containment of an adversarial process.

## Vello lifecycle launch blocked

The Vello/SVG/system-fonts/prelude lifecycle executable built in 7m28s, and its
locked graph passed the exact Etalon/Hypermedia source policy. Its SHA-256 is
`48b8ae6d10f826f4c53c9806c12231b989fb151946301212a432b8b345477717`;
`resume-native-vello-build-receipt.json` records the build profile and pins.
The bounded runner's first launch failed before process creation because Windows
Application Control blocked the executable. The ordinary execution escalation
and its single retry both timed out in automatic approval review before dispatch.
No application-control policy was disabled and no Vello runtime pass is claimed.
Previously completed CPU and default-hybrid runtime results remain separate.

## IME surrounding-text deletion

The actual attributed Blitz editor now implements `DeleteSurrounding`. It protects
the selected text or exact preedit region, validates both UTF-8 endpoints before
mutation, clamps oversized requests, preserves cursor visibility and remaps both
selection endpoints. Reconstructing the shorter buffer avoids Parley's grapheme
snapping from deleting composing bytes. The temporary buffer is reserved fallibly;
layout allocation and overall input memory are not bounded by this change.
Six editor regressions, a native composition-state regression and real document
input-routing checks were added. Independent source review found no new issue.
The actual locked editor run passes 6/6 (5m27s build, 0.24s execution;
`resume-native-ime-editor.log`). The changed actual native DOM suite passes 21/21
(6m00s build, 2.06s execution; `resume-native-dom-ime-final.log`), including real
committed-value input delivery and no duplicate event for invalid/zero requests.
OS IME qualification remains open.

