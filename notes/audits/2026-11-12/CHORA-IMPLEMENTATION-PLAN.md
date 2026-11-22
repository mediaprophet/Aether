# Chora implementation plan

## C0 — Reproducible fork and embedded profile

**Scope:** root Cargo/toolchain configuration, embedded profile, external consumer
fixture and fork CI. **Findings:** C12/C13. **Dependencies:** none.

- Restore a dependency-resolved locked build of the actual Chora fork in a clean
  environment. Record subrepository revisions, compiler, system dependencies and
  target requirements. Do not use a registry Servo package as a stand-in.
- Build an external Rust embedder and a C ABI consumer from the intended packaged
  interfaces. Verify library naming, headers, symbols and resources on disk.
- Define explicit profiles for embedded traditional browsing, optional canvas/GPU
  paths and development tooling. Ensure feature unification does not silently add
  media-gstreamer or unsupported embedding capabilities.
- Make fork CI runnable without assuming access to upstream runners or credentials.
  Keep upstream tests and add fork-specific gates.

**Exit:** locked embedded checks and smoke fixtures pass; build recipe and source
manifest are reproducible. This is a baseline, not browser readiness.

## C1 — Mandatory upstream wgpu to Etalon migration

**Scope:** root manifests/lockfile, `components/webgpu`, `shared/webgpu`,
`shared/constellation`, canvas Vello and shader/backend dependencies.
**Findings:** C01/C02. **Dependencies:** C0; coordinate with Aether A0.

1. Agree one reviewed Etalon source revision with QualiaDB and the
   hypermedia-production adapters. Build a small API/feature compatibility matrix
   for core, types, HAL, shaders and each supported backend.
2. Replace direct upstream core/types dependencies with the actual Etalon packages.
   Local aliases can ease incremental code adaptation, but inspect resolved package
   sources to prove the change. Adapt instance/device/queue/resource IDs, error
   handling, polling, map lifetimes and IPC serialization to the actual APIs.
3. Replace the Vello canvas dependency path with its reviewed Etalon-backed
   equivalent or a validated adapter. An Etalon `wgpu` shim may help consumers of
   the public API; it does not supply a missing core-API compatibility layer.
4. Align shader tooling and backend features. Regenerate the lockfile with reviewed
   changes, then inspect the complete optional/target graph and each shipping graph.
   Root-level patches must be repeated where necessary in the final embedding host;
   patches in a dependency's workspace are not inherited.
5. Keep JavaScript WebGPU behavior, limits, validation errors and promise/lifetime
   semantics. Do not remove WebGPU or disable Vello to declare migration complete.
   Document any intentionally unsupported platform feature separately.

**Required tests:** WebGPU WPT/CTS coverage appropriate to supported APIs; WGSL
validation; buffers/textures; mapping and cancellation; device loss; error scopes;
worker/IPC lifetimes; CPU/no-adapter behavior; Vello enabled and disabled; Metal,
Vulkan, DX12 and GLES where advertised; native surface interoperability.

**Exit:** no prohibited upstream wgpu implementation in the final host and standalone
Chora dependency graphs, including optional profiles. Pin-approved Etalon shims are
identified by source rather than name. Tests establish retained browser semantics
and supported backends. License-policy and notices changes are reviewed with the
actual dependency inventory. Migration is not complete after a textual rename.

## C2 — Generic embedding transport and authenticated caller context

**Scope:** WebView delegate, public exports, script/network dispatch and IPC.
**Findings:** C03/C04/C05/C14. **Dependencies:** C0; wire host with Aether A2/A4.

- Make embedding requests publicly usable and implement actual engine producers.
  Define request/reply/cancellation ownership, bounded message sizes, timeouts and
  execution thread. Compile an external consumer rather than testing only internally.
- Supply origin, top-level context, frame/worker identity, profile and navigation
  epoch from trusted engine state. Page payloads cannot choose these identities.
  Use generic opaque host handles; keep QualiaDB contracts out of Chora.
- Define redirects, sandboxed/opaque frames, workers, back/forward cache and teardown.
  Invalidate outstanding authority on navigation, close, revocation and host death.
- Replace global or disconnected provider configuration with explicit lifecycle and
  scope. Design authority context before connecting keystore operations. Avoid
  relying on mutable global registration to separate browser profiles.
- Route requests to the host for authorization. Default-deny unavailable providers
  and unsupported operations. Keep privileged internal protocols distinct from
  permissions for ordinary web origins.

**Exit:** an actual engine-originated page request reaches a generic external
embedder; allowed/denied/cancelled requests return correctly. Spoofed origin text,
stale replies, cross-profile handles and unauthorized subframes fail. No ecosystem
source or prohibited dependency has been added to Chora.

## C3 — Connect and validate TLS and host keystore capabilities

**Scope:** builder, network manager/resource threads, connector, WebCrypto provider.
**Findings:** C04–C08. **Dependencies:** C2 for privileged key authority.

- Pass custom roots and trust policy into network initialization before creating
  active clients. Implement acknowledged live reload or explicitly remove that
  promise until supported. Specify treatment of connection pools and in-flight work.
- Preserve distinct embedder-only, embedder+WebPKI and embedder+platform semantics.
  Propagate parse/validation errors. Test hostname, validity and chain rules; do not
  turn custom trust into a general verification bypass.
- Wire configured client-certificate resolvers and custom verifiers through every
  active TLS creation path. Verify signature scheme negotiation separately from
  certificate chain callbacks. Advertise PQ modes only after real handshake evidence.
- Connect the builder to the typed keystore provider after caller authority is
  available. Replace freely supplied key IDs with scoped authorized handles. Define
  non-extractability, allowed algorithms/usages, revocation and provider teardown.

**Required tests:** positive/negative custom CA fixtures, platform versus WebPKI
roots, root reload, expired/wrong-host certificates, mTLS allow/deny, unavailable
provider, unauthorized key import/sign, revoked/stale handle, profile separation,
and each advertised classical/PQ operation. Verify failure effects and error mapping.

**Exit:** builder configuration demonstrably changes actual network/crypto behavior;
all advertised policies have executable tests. Capability absence is explicit,
and no page gains ambient host key authority.

## C4 — Browser isolation and traditional compatibility

**Scope:** process model, sandbox, browser/platform integration, WPT and crash recovery.
**Findings:** C09 and traditional-compatibility requirements.
**Dependencies:** C0; integrate C1/C2 as available.

- Choose and implement a supported untrusted-content process/sandbox model for each
  release platform. Do not call unsupported sandbox constructors or substitute a
  feature flag for an isolation test. Keep the privileged host outside the page's
  authority domain. Define child startup, IPC validation and crash cleanup.
- Establish regression baselines for JavaScript/DOM, navigation, forms, workers,
  storage, service workers, permissions, downloads and network behavior required by
  target sites. Preserve SpiderMonkey compatibility during the Etalon changes.
- Inventory concrete V8/Node-specific requirements independently. Add a separate
  compatibility mechanism only for demonstrated API needs, with its own permissions
  and lifecycle; do not inject it into native human-centric execution.
- Verify compositor/surface ownership separately from the WebGPU migration. Define
  cross-process rendering, focus and accessibility behavior with the Aether host.

**Exit:** adversarial IPC/origin tests and platform isolation probes pass; engine
crashes do not expose host resources; named traditional sites and browser fixtures
meet the compatibility baseline. Unsupported platforms/capabilities are stated.

## C5 — Public API, ABI and branding completion

**Scope:** public Rust API, `ffi/capi`, bindings/headers, DOM branding and docs.
**Findings:** C10–C12. **Dependencies:** C0; coordinate evolving interfaces with C2/C3.

- Correct builder ownership so create/build/drop releases each allocation exactly
  once. Specify failure ownership, thread affinity and callbacks. Avoid unwinding
  across C boundaries; return actionable startup/runtime failures.
- Test real external C and Rust consumers, repeated create/destroy, failed build,
  callback cancellation and resource teardown. Check symbol and library names agree.
- Complete intended Chora public branding while retaining all upstream attribution.
  Review transitional aliases explicitly rather than mechanically replacing source
  history. Migrate privileged DOM exposure consistently with the local guide.
- Update examples to actual signatures and pin the actual fork. Verify docs from
  a clean consumer outside the engine workspace; define API/ABI version policy.

**Exit:** external fixtures compile and run, allocation/lifetime checks pass, headers
match exports, and documented API usage selects Chora rather than registry Servo.

## C6 — Mixed-service and distribution qualification

**Scope:** Chora integration fixtures, CI, packaged resources and security maintenance.
**Dependencies:** C1–C5 and Aether A4–A6 for the combined browser.

- Add a generic embedding conformance suite inside Chora and ecosystem integration
  tests on the host side. Exercise one mixed HTTPS site with legacy JS and native
  components, denied frames, revocation, navigation and crash/restart.
- Package resources/fonts, GPU support and required dynamic libraries for each
  advertised platform. Verify media exclusions and FreeType license selection in
  the resolved distribution. Retain engine/upstream notices and source provenance.
- Establish an update cadence for upstream engine/SpiderMonkey/security fixes and
  dependency advisories. Existing suppressions are not evidence of current safety.
- Publish the tested feature/target matrix, compatibility limitations and evidence
  for each previously advertised custom embedding capability.

**Exit:** installed browser artifacts pass traditional and mixed-service tests,
GPU dependency policy, isolation, resource/ABI tests and notices checks. Open P0/P1
items block the affected capability's release.

## Cross-repository ownership

| Concern | Implementation location | Required partner |
| --- | --- | --- |
| WebGPU-to-Etalon engine adaptation | Chora | Etalon and hypermedia-production |
| Generic caller/context transport | Chora | Host adapter conformance fixtures |
| QualiaDB/VibeScript policy and execution | Aether/Webizen host | Canonical QualiaDB SDK/runtime |
| Mixed native/web composition | Host adapter plus generic Chora surface API | Etalon ownership and platform isolation |
| Native data/EMF preservation | Canonical ecosystem and host renderer | Output conversion adapters |
| Release evidence | Each repo and final host | Target-specific packaging and security tests |

Do not place Chora source inside Aether merely to share a dependency workspace.
Do not introduce a competing VibeScript engine, provider ABI or QualiaDB schema
into Chora. Implement the engine's generic facilities and prove them through the
host integration that supplies the mandatory ecosystem foundation.
