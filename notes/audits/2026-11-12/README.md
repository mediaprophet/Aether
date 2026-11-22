# Aether and Chora production audit

Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.

This audit continues the native production review and records the requirement to
replace upstream wgpu with Etalon. It is a source review and implementation plan,
not a production certification or an implemented migration.

## Read in this order

1. [Aether audit](AETHER-AUDIT.md)
2. [Aether implementation plan](AETHER-IMPLEMENTATION-PLAN.md)
3. [Chora audit](CHORA-AUDIT.md)
4. [Chora implementation plan](CHORA-IMPLEMENTATION-PLAN.md)
5. [Verification and limitations](VERIFICATION.md)

## Required architecture

QualiaDB is the foundation supplied by the host application. Native Aether UI,
VibeScript execution and Etalon rendering use canonical ecosystem interfaces.
Chora provides traditional web rendering and JavaScript compatibility through its
generic embedding boundary. The Webizen browser can compose both kinds of content
within a site, with explicit authorization for local services.

Keep Chora in its own source repository and workspace. Do not copy it into Aether's
`vendor/` as the integration mechanism. Pin the actual fork and expose an
Aether-side adapter; use an isolated process where required by the security model.
Source/license separation and process isolation solve different problems. A vendor
directory alone establishes neither permission enforcement nor license compliance.
Retain upstream notices and produce a distribution-specific license inventory.

## GPU result

| Repository | Inspected dependency state | Required work |
| --- | --- | --- |
| Aether | `wgpu` aliases package `etalon` 0.0.2 at `575bab54accf1aafb24f67e9ba810c0f3f745aea`; no registry wgpu/core/hal/types in the lockfile | Align Etalon source identities with QualiaDB and prove every shipping profile |
| Chora | Registry `wgpu-core` and `wgpu-types` declarations; lockfile has wgpu/core/hal/types 30.0.1 | Migrate WebGPU and the optional Vello canvas path to Etalon |
| QualiaDB integration reference | Relevant Etalon declarations pin `6ace6cc6413a87ca10a1da58397ddc7c0b2fc41e` | Select one reviewed revision with Aether, Chora and hypermedia-production |

The [inventory](gpu-dependency-inventory.json) covers 122 Aether and 79 Chora
manifests, with zero parse errors. Full lockfile entries include optional and
target-specific paths; they do not prove a package is active in every build.
The [inspection script](inspect_gpu_dependencies.py) can reproduce this inventory.
Its registry-package count is an inventory aid, not a complete source-policy gate.

## Delivery sequence

Establish reproducible baselines and coherent GPU pins first. Fix Aether lifecycle
and asset boundaries, then integrate the canonical QualiaDB/VibeScript host.
Complete Chora's Etalon migration and generic embedding/authority hooks before
enabling mixed-site local services. Finish platform isolation, native interaction,
accessibility, performance and packaged-release qualification.

The current audit found no evidence sufficient to declare either the combined
native application path or the mixed-service browser production-ready. Existing
native DOM tests and Chora's extensive upstream test infrastructure are useful
foundations. Full native checks attempted here stopped before compilation.

## Scope and evidence

Aether base HEAD: `d5ccce3bb9d6cab2aa5d7ef185d57ad786737c27`, with working-tree
changes present. Chora HEAD: `f42e01c686ba24aa6f25d24ad308a6c004077220`;
its working-tree cleanliness could not be established through Git in this session.
The review used the local files, not an assumed pristine commit. Chora was read
only; both reports are saved in Aether. Lockfile and root manifest hashes are in
the inventory. File paths and named symbols identify evidence; line numbers may
move as implementation proceeds.

This is a cross-cutting architecture and release-gap audit, with deeper inspection
of native lifecycle, GPU dependencies, assets and privileged embedding paths. It
is not an exhaustive review of every inherited browser or framework function,
an up-to-date vulnerability database scan, or a substitute for runtime testing.
