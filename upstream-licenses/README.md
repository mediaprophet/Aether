# Upstream License Notice

Aether is an independent project founded and maintained by Timothy Charles Holborn.
Its starting source was a fork of the Dioxus `0.8.0-alpha.3` snapshot. The fork was taken
under the MIT and Apache-2.0 licenses before the architectural updates, `refract` zero-heap
lens reactivity runtime, Etalon WebGPU integration, and spatial desktop shell systems that
established Aether.

Some foundational VirtualDOM, reactive signal, RSX macro, and renderer components originated
in that snapshot, authored by the Dioxus developers and contributors. These credits describe
the inherited material's authorship; they do not identify the contributors or maintainers of
the independent Aether project.

Upstream Dioxus is dual-licensed under the Apache License 2.0 and the MIT License.
This directory preserves those upstream license texts and copyright notices:

- [`LICENSE-APACHE`](LICENSE-APACHE) — Apache License 2.0
- [`LICENSE-MIT`](LICENSE-MIT) — MIT License

Material that originates from upstream Dioxus remains available under those
upstream terms, wherever it appears in this repository.

### Aether modifications and additions

Development from that snapshot established Aether as a distinct sovereign application framework
for the Webizen and QualiaDB ecosystems. It includes improvements to inherited functions and
substantial new libraries and systems outside Dioxus's original scope, including `refract`,
`aether` / `aether-*` sovereign crates, W3C WebGPU device-limit shims, and Etalon GPU integration.
Its subsequent architectures, modifications, refactorings, extensions, and integrations are governed by:

**Copyright (c) 2026 Timothy Charles Holborn**  
**https://www.linkedin.com/in/ubiquitous/ | timothy.holborn@gmail.com**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)

See the root [`LICENSE`](../LICENSE) and [`LICENSE-QUALIA`](../LICENSE-QUALIA) files for those terms.
The upstream licenses preserved here do not grant rights to the Aether-specific modifications,
architecture, or Qualia-specific systems.
