# Native input implementation and remaining gates

Resumed qualification: the actual patched root native DOM library suite now passes
all 21 tests, and native CPU builds plus single/auxiliary-window runtime probes
pass. The historical missing-dependency failure recorded below is superseded for
those exact profiles. Six actual Blitz editor tests verify surrounding-text
deletion, including UTF-8, reversed selections and combining preedit boundaries.
Four additional actual editor tests verify clipboard error handling and nonmutating copy; see
[native recovery](resume-native.md). Platform IME and application clipboard event
qualification remain separate.

## Implemented boundary

`packages/native-dom/src/events.rs` now provides `native_event_support(name) ->
Result<(), UnsupportedNativeEvent>`, exported from the crate root. It accepts
supported application event names (with or without one `on` prefix) and reports
unsupported event families explicitly. This is a payload/routing capability check;
it does not establish delivery by a particular OS or browser compatibility.

The exhaustive `NativeConverter::convert_dom_event` pairs each supported Blitz
`DomEventData` variant with its actual Aether native payload. The dispatcher only
routes successful conversions. Mouse/pointer, touch, keyboard, focus, form input,
scroll and wheel data keep their existing native backing. No clipboard, animation,
media, drag, resize or visibility payload is invented.

The inherited `HtmlEventConverter` trait has infallible return types. Its methods
for backend-absent event families still cannot express `Unsupported` and retain
invariant-failure stubs. They are not reachable through the checked native dispatch
boundary. Direct calls with unsupported or mismatched platform payloads remain a
trait/API limitation; replacing those stubs with fabricated primitive values would
misrepresent support. A future shared fallible converter contract must preserve the
other renderers' traditional web behavior.

## Composition routing

Blitz beta.4's `BlitzImeEvent` distinguishes IME availability from composition.
A document now retains a bounded composition state independently of listener counts:

| Native notification | Application observation |
| --- | --- |
| Enabled | No composition event; availability does not start composition |
| First nonempty Preedit | compositionstart with the initial empty composition, then compositionupdate with the actual preedit text |
| Further Preedit, including empty clear before Commit | compositionupdate with the exact supplied text |
| Commit during an active composition | compositionend with the committed text |
| Commit without an active composition | No fabricated composition session; Blitz still handles native insertion |
| Disabled or delivered blur/focusout during composition | One compositionend with empty data for cancellation |
| DeleteSurrounding | Native editor deletes only valid surrounding committed bytes, preserves selection/preedit and emits input for an effective deletion; no invented composition phase |

Composition listener names map to Blitz's Ime handler counter in
`mutation_writer.rs`. Lifecycle notifications are observations; preventing their
default does not suppress or duplicate Blitz's editor insertion. The native editor
continues to generate its single actual input notification. No beforeinput or
cancellable edit contract is claimed.

The state ends the previous target's session before switching targets, resolves
that target's current DOM chain, and skips removed targets. After mutations, stale
state is cleared when its node no longer exists or owns native focus. No callback
is generated for a component that has already been removed.

## Source evidence and tests

Inspected canonical cached sources:

- blitz-traits 0.3.0-beta.4 `src/events.rs`: actual DOM variants, IME availability,
  Preedit/Commit ordering and UTF-8 byte cursor indices.
- blitz-dom 0.3.0-beta.4 `src/events/driver.rs` and `src/events/mod.rs`: dispatch
  targets, editor default actions and generated input notifications.
- blitz-dom 0.3.0-beta.4 `src/node/text.rs`: the original composition/editor handling
  and missing DeleteSurrounding operation. The attributed local patch now implements
  that operation and passes six actual editor regressions.

Durable regressions cover first preedit, subsequent updates, empty clear before
commit, direct commit without a session, repeated disable, cancellation, target
switching/teardown, typed conversion, capability rejection and composition listener
counts. The document regression exercises typed composition handlers while checking
that editor commit produces input once even when compositionend prevents default.

Verification on 12 November 2026: **20 tests passed** against the actual
`packages/native-dom/src/lib.rs` and its source modules, using a separate headless
manifest with real Blitz beta.4 and the minimal Aether facade. The tests include the
new IME/editor regression and the existing DOM, mount, render, touch and textarea
regressions. The IME fixture performs the first layout before native event delivery,
matching editor initialization in the window path. The result used unlocked
resolution of 281 cached packages, with default native features disabled; it is not
verification of the locked root dependency graph, an Etalon renderer or a window.

Receipt: [`native-input-tests.log`](native-input-tests.log).
Reproduction packet: [`native-input-probe`](native-input-probe/README.md), with relative actual-source paths, locked dependency graph and offline probe script.
Source formatting and whitespace checks passed for the four modified source files.

## Remaining release gates

- OS IME delivery, candidate-window placement, cursor byte ranges and composing
  keyboard flags need platform runtime evidence. CompositionData exposes text only;
  its inherited API does not expose preedit selection/cursor ranges.
- Beforeinput, clipboard/data transfer, drag/drop, selection notifications, media, animation,
  resize and visibility events still need backend emission and real data contracts.
- Programmatic focus in NodeHandle still does not queue focus/blur events; native
  blur teardown here applies when the backend actually delivers that notification.
- Existing touch targetTouches approximation, default contact geometry and pointer
  identity behavior still need qualification; these changes do not establish full
  multi-input web parity.
- Accessibility actions, platform assistive-technology interaction and packaged
  native runtime behavior have not been verified. Feature inclusion or headless
  tests do not establish accessibility readiness.
- This work does not establish QualiaDB/VibeScript execution or Etalon resource
  lifecycle readiness and does not remove JavaScript compatibility from other paths.

A separate actual `aether-native` check selected `vello-cpu-softbuffer` with default
features disabled and seeded its isolated lockfile from the current shipping
`Cargo.lock`. It stopped before compilation because the cached crates.io index
lacks `utf8-width`, required by locked `html-escape 0.2.13` through Blitz DOM.
No dependency was substituted or shipping pin changed to bypass this failure.
Receipt: [`native-cpu-check.log`](native-cpu-check.log).
The native window preparation implementation therefore still needs a successful
build against the complete declared dependency graph and runtime validation.
