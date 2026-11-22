# Chora audit

## Assessment

Chora supplies a substantial traditional browser engine inherited from Servo.
Its current JavaScript engine is SpiderMonkey. Standard JavaScript compatibility
does not imply support for Node.js or V8-specific APIs. Preserve the traditional
web path while adding generic, authorized host integration.

Chora still depends on upstream wgpu. Its embedding additions also contain
disconnected hooks and incomplete security/lifecycle contracts. The repository's
descriptions of a capability are not evidence that it is wired into an actual
page, network connection or cryptographic operation.

This review was read-only. Paths below are relative to `C:/github/chora`.
**P0** blocks exposure of privileged local services or the untrusted-browser
release on the affected platform; **P1** blocks intended production functionality;
**P2** is a documentation or maintainability gap. Runtime verification remains
blocked as described in [verification](VERIFICATION.md).

## Findings

| ID | Priority | Evidence | Finding and consequence |
| --- | --- | --- | --- |
| C01 | P1 | Root `Cargo.toml`, `Cargo.lock`; `components/webgpu`, `components/shared/webgpu`, `components/shared/constellation` | Direct wgpu-core/types requirements are upstream version 30. The lockfile includes registry wgpu/core/hal/types 30.0.1. Mandatory Etalon migration is not done. |
| C02 | P1 | Root Vello declaration; `components/canvas/Cargo.toml`; lock reverse edges | Optional canvas Vello 0.11 also pulls upstream wgpu. Migrating only direct WebGPU imports leaves this second path behind. |
| C03 | P1 | `components/servo/webview_delegate.rs`, `EmbedderMessageRequest`, `notify_embedder_message`; `components/servo/lib.rs` exports | The generic message request and callback are declared, but no engine producer/callback call site was found in the component source scan. The request is also absent from the public export list inspected. This is not a functioning page-to-host bridge. |
| C04 | P1 | `components/servo/servo.rs`, builder keystore field and accessor; `components/shared/embedder/keystore.rs` | Builder keystore storage uses a separate opaque object from the typed global provider used by WebCrypto. No call wiring the typed registration from the builder was found. Configuring the advertised builder does not establish WebCrypto delegation. |
| C05 | P0 before provider exposure | `components/shared/embedder/keystore.rs`; `components/script_webcrypto/subtlecrypto.rs`, `parse_host_key_data`, `ImportKeyAlgorithm::import_key`, host signing paths | Host key imports accept string handles; provider calls carry key/algorithm/data but no authenticated origin/frame/profile/grant context. A host can restrict keys, but this contract does not supply enough caller identity for per-origin policy. Never register broad local key authority into this path. |
| C06 | P1 | `components/servo/servo.rs`, `new_resource_threads` and later `add_custom_root_pem`; `components/servo/network_manager.rs` | Builder custom roots are added after resource-thread construction to a stored vector. No propagation/reload to the active TLS verifier was found. Advertised custom-root behavior needs end-to-end implementation. |
| C07 | P1 | `components/net/connector.rs`, `RootCertificatePolicy::from_policy` and TLS construction | `EmbedderAndWebPki` and `EmbedderAndPlatform` collapse into the same combined policy; the actual default-root choice follows separate configuration. Distinct advertised trust modes are not preserved by this mapping. |
| C08 | P1 | `components/net/connector.rs`, `create_tls_config_with_client_auth`, `create_tls_config`; callers in `resource_thread.rs`, `http_loader.rs` | Lower-level client certificate/custom verifier arguments exist, but inspected active callers use the wrapper supplying neither. Builder-to-connection wiring is incomplete. Delegated certificate-chain verification also does not by itself prove post-quantum handshake signature support. |
| C09 | P0 browser release | `components/servo/servo.rs`, platform-specific `create_sandbox`; `components/servo/Cargo.toml`, `embedded_library` | Sandbox creation panics on listed unsupported platforms if invoked, and the embedded profile does not itself enable multiprocess. Feature presence alone would not prove isolation either. Establish a real platform sandbox/process model before shipping untrusted browsing. |
| C10 | P1 | `ffi/capi/lib.rs`, `servo_builder_build`, `chora_builder_build` | Documentation says build consumes the builder, but the implementation borrows the raw allocation and takes fields without reclaiming its box. The documented create/build/engine-drop lifecycle leaks the builder allocation. Ownership and error semantics need regression tests. |
| C11 | P1 distribution | `components/servo/lib.rs`; C API exports; `components/script_bindings/webidls/ServoInternals.webidl`; root/package metadata | Chora aliases coexist with public Servo names and legacy DOM exposure. The intended library branding boundary is incomplete. Preserve provenance while replacing public branding; do not confuse dependency aliases with selecting the fork. |
| C12 | P2, P1 for external consumers | README embedding example; `ServoBuilder::build`; crate manifests | The example's `build()?` does not match the inspected non-Result return. A registry dependency with `package = "servo"` does not select this fork. Documentation needs an external-consumer compile fixture with an explicit fork source. |
| C13 | P1 | `embedded_library`, workspace features, `deny.toml`, `about.toml`, CI | The intended media/license profile is useful but feature unification and final distribution dependencies require verification. Existing license policy does not cover the proposed Etalon dependency set. Upstream test infrastructure is extensive; fork-specific native/mixed-service/Etalon qualification is not demonstrated. |
| C14 | P1 | `components/net/protocols/mod.rs`, privileged URL/path handling; host interface definitions | Internal privileged-protocol allowlists are not a permission model for ordinary HTTPS sites or mixed components. Do not grant site access to local services by reclassifying arbitrary URLs as privileged engine pages. |

The keystore finding describes a deficient authorization contract, not a
demonstrated remotely exploitable key operation in the current default build.
The disconnected registration path currently limits which advertised behavior is
reachable. Repairing that wiring without first fixing authority would increase risk.

## GPU migration scope

The inspected lockfile establishes both paths:

| Consumer | Upstream GPU path | Migration requirement |
| --- | --- | --- |
| `servo-webgpu`, `servo-webgpu-traits`, `servo-constellation-traits` | `wgpu-core` and `wgpu-types` | Adapt to pinned Etalon core/types; preserve DOM WebGPU semantics and IPC ownership |
| `servo-canvas` optional Vello renderer | `vello` → `wgpu` → core/HAL/types | Use the reviewed Etalon-backed renderer or compatible adapter; test this enabled profile |
| Shader/backend dependencies | Naga, bridge and platform dependencies in GPU graph | Align source identities and backend features with the selected Etalon revision |

The inspected Etalon repository provides a compatibility crate named `wgpu` in
`compat/wgpu`, but no equivalent `compat/wgpu-core` was found. Therefore a blanket
Cargo patch or search-and-replace is not an established solution to Chora's
direct core integration. API compatibility, features and compiler requirements
must be checked. The compatibility crate inherits a different MSRV from several
core Etalon packages in the inspected checkout.

Moving WebGPU to Etalon does not automatically replace WebRender/Surfman or the
entire compositor. WebGL, page composition and native-surface sharing need their
own ownership and interoperability design. The requirement is to eliminate
upstream wgpu implementations, not to pretend every browser renderer is wgpu.

## Host boundary and licensing

Keep the source/workspace boundary required by Chora's AGENTS.md. Generic messages,
protocols, certificate/keystore providers and C ABI belong in Chora. QualiaDB,
VibeScript authority, canonical data and UI bindings belong in the host. Do not
copy ecosystem implementation into Chora or add prohibited ecosystem dependencies.

Etalon integration must include a reviewed inventory of actual dependency licenses
and preserved source notices; directory placement alone is not enough. Keep
GStreamer excluded from the embedded profile and verify the elected FreeType
license in the shipped artifact. Do not broaden a license allowlist globally just
to silence the new dependencies. These are engineering release tasks, not a legal
conclusion about every possible distribution.

## Existing foundations and verification limits

The fork retains extensive browser, unit, WPT and C API testing infrastructure.
Generic embedding types, network policy enums and WebCrypto provider call paths
exist and are worth completing. The embedded profile already excludes media-
gstreamer and includes browser resources and JavaScript JIT support.

The audit did not run WPT, GPU workloads, TLS handshakes or a browser sandbox test.
The attempted embedded build stopped at an unavailable offline Stylo revision.
The current security-advisory status and fork CI runner availability were not
verified. A functioning traditional browser substrate is not evidence for the new
local-service hooks. See the [implementation plan](CHORA-IMPLEMENTATION-PLAN.md).
