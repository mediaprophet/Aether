# Reviewed Chora patch series

On local resumption, the complete four-patch series was verified against the
clean `C:/github/chora` checkout and applied there. All 62 patch-listed files
matched the saved candidate before further work. The original checkpoint remains
available in these patches and the separate snapshot. MPL headers and inherited
attribution are preserved; no QualiaDB/Webizen implementation or dependency was
added inside the engine. These are uncommitted local changes, not a qualified
browser release. Current build and follow-up evidence is in
[resume-chora.md](resume-chora.md).

## Apply order

Use a checkout matching the packet baseline hashes. The verified order is:

| Order | Patch | Scope |
| --- | --- | --- |
| 1 | [C1-etalon-source-migration.patch](chora-etalon/C1-etalon-source-migration.patch) | Etalon core/types/canvas sources, owner thread, mapping, closed devices, error scopes and registration ordering |
| 2 | [chora-c2-embedding.patch](chora-embedding/chora-c2-embedding.patch) | Generic page/host request transport, trusted caller routing, bounds, cancellation and navigation epochs |
| 3 | [chora-c3-tls.patch](chora-tls/chora-c3-tls.patch) | Root/trust/client identity/verifier configuration, HTTP/WebSocket wiring and profile isolation |
| 4 | [C10-capi-startup-ownership.patch](chora-etalon/C10-capi-startup-ownership.patch) | Consuming builder ownership and fallible C startup; requires C3 try_build |

The supervisor applied this entire series in a fresh temporary directory using
actual original file bytes. All **62 patch-listed files** match the reviewed
combined candidate source. Raw SHA256 and EOL-normalized comparisons are recorded
in [chora-patch-verification.json](chora-patch-verification.json). A full inventory
of the partial working snapshot found no unexported source changes; its one
excluded extra file is a working-copy provenance marker. The snapshot excludes
WPT and Git metadata, and is not a complete source distribution.

[verify_chora_patches.py](verify_chora_patches.py) reproduces sequential application
and candidate comparison without modifying either input checkout. It uses
`core.autocrlf=false`; hashes distinguish raw bytes from source text on Windows.
Individual packet manifests preserve their original and patch-stage hashes.

## Evidence and limits

| Packet | Executed evidence | Still required |
| --- | --- | --- |
| [C1/C10](chora-etalon/README.md) | Original 11 helper/2 ownership/5 policy tests; resumed actual root WebGPU check, root source inventory and expanded 13-test Etalon lifecycle fixture pass | Full DOM/engine/CAPI build, optional canvas build, WebGPU CTS/WPT and hardware |
| [C2](chora-embedding/README.md) | Original 5 ledger tests and actual selected WebIDL generation; shared interfaces compile in resumed root WebGPU graph | Full engine/external Rust consumer build, protocol test execution and browser requests/navigation/teardown |
| [C3](chora-tls/README.md) | 9 actual rustls source-copy tests, including genuine TLS/mTLS handshakes and profile resumption isolation | Complete network engine integration, other platforms, PQ handshakes, live reload and scoped keystore |

The generic bridge initially permits only active, unsandboxed top-level HTTP(S)
documents; host authorization is still mandatory. It is not a general iframe or
worker capability grant. Chora retains its traditional JavaScript path. No V8 or
Node-specific API compatibility is inferred from SpiderMonkey support.

At the original checkpoint, missing Stylo source prevented the full engine graph
from compiling. The
optional canvas graph resolves to the intended Etalon-backed renderer, but its
build lacked svg_fmt. Local resumption retrieved both exact inputs and Cargo
regenerated Chora's root lock; the root GPU source inventory now passes. This
does not establish a successful full engine build or shipping migration.
Aether, QualiaDB and the candidate renderer pins still
need joint qualification before sharing GPU resources.

The [live task list](../architecture/14-IMPLEMENTATION-TRACKER.md) retains those
steps plus mixed-site host integration, scoped graph authority, platform isolation,
installed native packaging, licensing inventory and update/recovery tests.
