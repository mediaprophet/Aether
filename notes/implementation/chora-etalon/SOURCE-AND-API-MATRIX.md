# Chora GPU source and API checkpoint

This is the preserved source/API checkpoint. The series is now applied locally,
the actual root WebGPU package compiles, and the expanded lifecycle fixture passes
13 tests. See [current evidence](../resume-chora.md). Full browser/DOM/C-ABI and
hardware qualification remain separate.

Etalon target: `8e8ab6c8349bcc6ab1d81138f2c9f2a0c90f0952`. Renderer/shader target: hypermedia-production `af57e82120b30b47fca238edb21a839d2cdb4278`, whose actual manifests pin that Etalon revision. Rust dependency keys such as wgpu-core/wgpu-types and vello select actual Etalon/vector packages; names alone are not the evidence. The Cargo-generated independent probe lock passes the source-identity gate.

The audit observed Aether using Etalon `575bab54accf1aafb24f67e9ba810c0f3f745aea` and QualiaDB using `6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e`. Cross-host revision/device coherence is not established by this patch. The private closed-resource instance remains wholly internal to Chora.

| Actual package | Version | Declared Rust minimum | Declared license |
| --- | --- | --- | --- |
| `etalon-core` | 0.0.2 | 1.87 | CC-BY-NC-ND-4.0 |
| `etalon-core-remote` | 0.0.2 | 1.87 | CC-BY-NC-ND-4.0 |
| `etalon-core-remote-types` | 0.0.2 | 1.87 | CC-BY-NC-ND-4.0 |
| `etalon-types` | 0.0.2 | 1.87 | CC-BY-NC-ND-4.0 |
| `etalon-hal` | 0.0.2 | 1.87 | CC-BY-NC-ND-4.0 |
| `naga` | 0.0.2 | 1.87 | CC-BY-NC-ND-4.0 |
| `naga-types` | 0.0.2 | unspecified | CC-BY-NC-ND-4.0 |
| `etalon` | 0.0.2 | 1.87.0 | CC-BY-NC-ND-4.0 |
| `vector-compute-renderer` | 0.11.0 | 1.89 | Apache-2.0 OR MIT |
| `vector-compute-shaders` | 0.11.0 | 1.89 | Apache-2.0 OR MIT |

These are observed package declarations, not a final distribution license clearance. Source/MPL notices and upstream attribution are preserved. No QualiaDB or VibeScript dependency is introduced into Chora, and no license allowlist or embedded media profile was broadened.

| Boundary | Source adaptation | Evidence and remaining limit |
| --- | --- | --- |
| Core/remoting | Actual Etalon implementation, canonical remoting Global/IDs/descriptors; content traits depend on remote types rather than GPU core | Real vendor types and actual Chora IPC type/encoder source compile in probe; full browser integration still requires qualification (the resumed root WebGPU package now compiles) |
| Ownership | Rc Global stays GPU owner; backend callbacks only signal; renderer receives CPU-owned copies; map guard ends before unmap | Owner queue/runtime tests and source review; full compositor/native surface tests absent |
| Lifetimes | Explicit paired device/queue removal; failed request and undelivered response cleanup; async pipeline rejection/undelivered ID retirement | Source reviewed; full IPC/browser lifecycle qualification pending |
| Errors/shaders | Canonical device scopes, uncaptured errors and WGSL compilation info with UTF-16 locations | Actual helper/type compilation; WPT/CTS not run |
| Valid unfulfillable request | Private Etalon Noop Device/Queue destroyed before genuine hub import; already-lost DOM resolution | Actual creation/import/closed resource and cross-device validation tests; private adapter absent from browser enumeration |
| Loss delivery | Weak DOM registration sends DeviceReady; owner buffers first loss until readiness, retires pending loss before ID release; stale epoch/document lookup benign | Three event-gate tests and producer/consumer source review; browser navigation/worker races still need runtime tests |
| Completion order | Map and queue-done share owner completion FIFO; registration-ordered cancellation; map errors use canonical classification | Real vendor Abort map + queue ordering and cancellation tests |
| Invalid descriptor | Missing BGL/view conversions still register genuine invalid core resources | Actual nested-scope invalid-view/cross-device regression; remote API lacks public create_texture_view_error, so adapter uses canonical zero-mip validation without duplicating content's event |
| Optional canvas | vector-compute-renderer/shaders and actual Etalon frontend pinned | Probe lock resolves reviewed family; enabled compilation stops at uncached svg_fmt 0.4.5 |
| Backends | Metal/Vulkan/DX12/GLES flags retained; etalon-core/noop added solely for private already-closed resources | No live GPU/backend/platform interoperability qualification; Noop tests are failure-path evidence only |

Normative checks used the current primary [requestDevice algorithm](https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice), [async pipeline creation](https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipelineasync), [mapAsync](https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapasync), and [queue completion](https://gpuweb.github.io/gpuweb/#dom-gpuqueue-onsubmittedworkdone). Unfulfillable valid device requests resolve already-lost devices; lost pipeline/work-done promises resolve, while lost/cancelled mapping aborts. No new default OperationError path replaces the device-loss result.
