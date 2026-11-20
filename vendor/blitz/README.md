# Native fetch and editing patches

These are local copies of the published `blitz-traits`, `blitz-dom` and
`blitz-net` 0.3.0-beta.4 packages, from DioxusLabs/Blitz revision
`92b11f1bd8ab7975411328b0406a95a09445b4e7`. Original package manifests,
source headers, assets and `.cargo_vcs_info.json` are retained. Those VCS records
identify the inherited source, not the modified working tree.

Inherited code remains **MIT OR Apache-2.0**; the original license texts are
included here. Preserve Dioxus/Blitz attribution when redistributing. Aether
modifications are copyright (c) 2026 Timothy Charles Holborn and governed by
the repository's LICENSE and LICENSE-QUALIA. This notice does not relicense
the inherited code.

The patches add a typed terminal failure method to the existing `NetHandler`
contract, complete failed DOM resources, and wake/unregister cancellation
waiters. They do not replace the network abstraction or introduce ecosystem
dependencies into Blitz. The default handler method preserves legacy external
implementations; clients that track pending resources must implement it.

The native editor also preserves the selection when clipboard access fails,
does not emit input for copy, and only deletes cut text after a successful
clipboard write. These changes retain the existing shell provider contract;
they do not claim application clipboard events or OS permission qualification.

IME surrounding-text deletion validates both UTF-8 removal boundaries before
mutation, preserves the selected text or exact preedit bytes, and restores cursor
visibility and selection direction. It emits input only for an effective deletion.
Preedit protection does not use a temporary selection that could snap across a
combining grapheme. The temporary result buffer is reserved fallibly; this is not
a general memory bound on text layout or the application. Platform IME qualification
remains a separate gate from the editor and event-routing regressions.

Why local source is necessary: the published bytes-only interface cannot report
a denied or failed asset without either leaving a critical resource pending or
misrepresenting an error body as successful content. Editing a Cargo cache would
make the correction irreproducible. Root Cargo patches select these attributed
sources explicitly; external consumer workspaces must carry the same overrides.

The native production tracker records build/test evidence and remaining gates.
