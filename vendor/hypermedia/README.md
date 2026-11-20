# Hypermedia shader compatibility patch

`vector-compute-shaders` and `vector-compute-renderer` are copied from hypermedia-production commit
`a46b4e2ba5a9d57fc1a68e20ae87ce4bc2d076cf` as resolved in Aether's pre-patch
Cargo.lock. Its original source,
Vello attribution, `.cargo_vcs_info.json`, `SOURCE-MIGRATION.md`, manifests,
Apache-2.0 and MIT license files are preserved.

The maintained patch dereferences Etalon Naga's boxed validation error before
converting it to the crate's existing diagnostic type. Shader validation remains
enabled and unchanged. Both Naga dependencies retain Aether's reviewed Etalon
`575bab54accf1aafb24f67e9ba810c0f3f745aea` source. The corresponding local
hypermedia source receives the same correction for canonical integration.

The renderer patch supplies the new texture-view `swizzle` field at three existing
view creation sites. Etalon's default maps RGBA to RGBA, preserving prior behavior
without requiring the optional component-swizzle feature. Its original optional
profiler path dependency is retained: `gpu-command-profiler` is copied unchanged
from the same exact hypermedia commit with its original licenses. This avoids
introducing a floating Git reference for an optional prerequisite. The unmodified
hypermedia manifest is also retained as `Cargo.toml.hypermedia-original`.

Qualia-specific modifications retain the attribution and terms documented by
the repository's LICENSE and LICENSE-QUALIA. No release publication is implied.
