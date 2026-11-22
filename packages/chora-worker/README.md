# Separate Chora worker — local implementation

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

`aether-chora-worker` owns the actual sibling Chora engine on its engine thread.
It is an independent Cargo workspace because Chora's SpiderMonkey/Stylo/ICU
requirements cannot currently be unified with native Blitz text dependencies.
Only copied RGBA frames cross to the native process; there is no shared GPU device.

The parent uses `packages/chora-client` and an explicit absolute executable path.
`ChoraClient::launch` requires an explicit execution profile. This initial worker
supports **TrustedEmbedding, public browsing only**. Required sandboxing and
private browsing reject before engine/context creation (and before launch in the
parent). A separate process does not provide OS containment.

The parent creates private stdin/stdout pipes. The worker immediately duplicates
its output pipe to an owned file, redirects Rust/CRT diagnostics to stderr, and
writes framed protocol bytes only through that file. Neither command arguments
nor inherited environment contain graph credentials; the environment is reduced
to explicit OS/display/path variables. The nonce identifies the owned session and
confers no authority. `check_sources.py` checks actual engine/GPU package sources.

The parent supplies a separate stderr pipe even when its GUI process has no
console. A background reader continuously discards diagnostics with bounded
memory, so engine logging cannot fill that pipe and stall protocol delivery.
This initial client does not expose a diagnostic log.

Control JSON is versioned and bounded at 128 KiB; binary frames are bounded at
16 MiB. Every message carries a sequence and session/view/generation/document/
viewport stamp. Only 64 real engine requests may remain pending, for at most
30 seconds. Captures use one outstanding slot; the parent retains one latest
frame. Startup/capture/navigation/resize timeouts and a two-second shutdown grace are enforced by a
parent thread that kills and reaps the child independently of UI callbacks.
If the parent disappears, a worker-side private-input watchdog independently
force-terminates its own process after two seconds without Rust/C cleanup or DLL
detach callbacks; input queue overload also fails
closed instead of preventing EOF detection.

Explicit parent navigation retires the old WebView/delegate and creates a fresh
view for the new generation. It therefore resets that view's history. Retired
callbacks cannot acquire the new owner's stamp. Actual URL notifications advance
the worker document counter and cancel pending work; resize advances viewport.
Close/crash cancels and signals the parent. Paint precedes readback, which precedes
present; dimensions and the conservative four-buffer readback budget are admitted
before readback. Chora already flips GL readback vertically.

The native `chora` feature adds `chora_frame::ChoraFrameWidget`, a copied-image
AnyRender widget, plus the parent client; it never links the engine into native.
The host must service client events separately. `qualia-host` supplies the optional
`chora-client` pump and explicit `ChoraGraphRoutes` registration backed by a real
signed `NativeGraphBridge`. Only `qualia.graph.snapshot` with payload `{}` is
supported. Unknown routes and channels deny. All six graph lanes use decimal
strings to preserve their full u64 range in traditional web JavaScript. The worker
holds actual engine request handles, never inferred page claims or credentials.

See `notes/implementation/resume-worker.md` for executed checks. Full real-worker
launch, internal-navigation document-to-pixel correspondence, alpha/orientation,
input, accessibility, GPU failure and containment qualification remain gates.
The transport fixture tests are real process/pipe tests, not Chora engine tests.
No production-readiness claim follows from this implementation or compilation.
