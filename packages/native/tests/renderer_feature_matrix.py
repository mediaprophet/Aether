"""Compile the actual renderer wrapper against small backend API contracts.

Run with Python 3.11+ and rustc on PATH. This offline regression test checks cfg
selection and configuration forwarding, not upstream renderer API compatibility,
GPU initialization, or native production readiness. Full Cargo/platform checks
remain required. No Cargo dependency resolution or shared target directory is used.
"""

import argparse
from pathlib import Path
import subprocess
import tempfile
import tomllib


NATIVE = Path(__file__).resolve().parents[1]

# Independent expected outcomes, including the default hybrid overridden by a
# CPU/Skia choice and the all-features case that previously hid those failures.
CASES = [
    ("hybrid", ["vello-hybrid"], "hybrid"),
    ("vello", ["vello"], "vello"),
    ("cpu", ["vello-cpu"], "cpu"),
    ("cpu-pixels", ["vello-cpu-pixels"], "cpu"),
    ("cpu-softbuffer", ["vello-cpu-softbuffer"], "cpu"),
    ("skia", ["skia"], "skia"),
    ("raster-pixels", ["skia-pixels"], "raster"),
    ("raster-softbuffer", ["skia-softbuffer"], "raster"),
    ("default+cpu", ["vello-hybrid", "vello-cpu"], "cpu"),
    ("default+cpu-softbuffer", ["vello-hybrid", "vello-cpu-softbuffer"], "cpu"),
    ("default+skia", ["vello-hybrid", "skia"], "skia"),
    ("default+raster", ["vello-hybrid", "skia-pixels"], "raster"),
    ("default+raster-softbuffer", ["vello-hybrid", "skia-softbuffer"], "raster"),
    ("gpu+cpu", ["vello", "vello-cpu"], "vello"),
    ("cpu+skia", ["vello-cpu", "skia"], "cpu"),
    ("skia+raster", ["skia", "skia-pixels"], "skia"),
    ("all-renderers", ["vello", "vello-hybrid", "vello-cpu", "skia", "skia-pixels"], "vello"),
]

# Same macro semantics as cfg-if: later branches exclude earlier predicates.
# Compiled separately so the wrapper's cfg_if::cfg_if! calls are unmodified.
CFG_IF = r'''
#[macro_export]
macro_rules! cfg_if {
    (if #[cfg($($meta:meta),*)] { $($tokens:tt)* } $(else if #[cfg($($more:meta),*)] { $($other:tt)* })* else { $($last:tt)* }) => {
        $crate::cfg_if! { @items () ;
            (($($meta,)*) ($($tokens)*)),
            $((($($more,)*) ($($other)*)),)*
            (() ($($last)*)),
        }
    };
    (@items ($($not:meta,)*) ; ) => {};
    (@items ($($not:meta,)*) ; (($($meta:meta,)*) ($($tokens:tt)*)), $($rest:tt)*) => {
        #[cfg(all($($meta,)* not(any($($not),*))))]
        $crate::cfg_if! { @identity $($tokens)* }
        $crate::cfg_if! { @items ($($not,)* $($meta,)*) ; $($rest)* }
    };
    (@identity $($tokens:tt)*) => { $($tokens)* };
}
'''

CONTRACTS = r'''
extern crate self as anyrender;
extern crate self as peniko;
extern crate self as anyrender_vello;
extern crate self as anyrender_vello_hybrid;
extern crate self as anyrender_vello_cpu;
extern crate self as anyrender_skia;
extern crate self as etalon;

use std::{any::Any, sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq)] pub struct Color(pub u8);
#[derive(Clone, Copy, Debug, PartialEq)] pub enum CompositeAlphaMode { Opaque }
#[derive(Default)] pub struct RendererConfig {
    pub base_color: Option<Color>, pub composite_alpha_mode: Option<CompositeAlphaMode>,
}
#[derive(Clone, Copy)] pub struct Features;
#[derive(Clone, Copy)] pub struct Limits;
pub struct ResourceId;
pub struct RegisterResourceError;
pub trait WindowHandle {}
pub trait RenderContext {
    fn try_register_custom_resource(&mut self, r: Box<dyn Any>) -> Result<ResourceId, RegisterResourceError>;
    fn unregister_resource(&mut self, r: ResourceId);
    fn renderer_specific_context(&self) -> Option<Box<dyn Any>>;
}
pub trait WindowRenderer: RenderContext {
    type ScenePainter<'a> where Self: 'a;
    fn resume<F: FnOnce() + 'static>(&mut self, w: Arc<dyn WindowHandle>, x: u32, y: u32, f: F);
    fn complete_resume(&mut self) -> bool;
    fn suspend(&mut self);
    fn is_active(&self) -> bool;
    fn is_pending(&self) -> bool;
    fn set_size(&mut self, w: u32, h: u32);
    fn render<F: FnOnce(&mut Self::ScenePainter<'_>)>(&mut self, f: F);
}
#[derive(Debug, PartialEq)] pub struct Observed {
    backend: &'static str, color: Option<Color>, alpha: Option<CompositeAlphaMode>,
    features: bool, limits: bool,
}
pub struct GpuOptions { config: RendererConfig, features: bool, limits: bool }
impl From<RendererConfig> for GpuOptions {
    fn from(config: RendererConfig) -> Self { Self { config, features: false, limits: false } }
}
impl GpuOptions {
    pub fn features(mut self, _: etalon::Features) -> Self { self.features = true; self }
    pub fn limits(mut self, _: etalon::Limits) -> Self { self.limits = true; self }
}
pub type VelloRendererOptions = GpuOptions;
pub type VelloHybridRendererOptions = GpuOptions;
macro_rules! renderer {
    ($name:ident, $options:ty, $backend:literal, $convert:expr) => {
        pub struct $name(Observed);
        impl $name {
            pub fn with_options(options: $options) -> Self {
                let (c, features, limits): (RendererConfig, bool, bool) = ($convert)(options);
                Self(Observed { backend: $backend, color: c.base_color,
                    alpha: c.composite_alpha_mode, features, limits })
            }
        }
        impl RenderContext for $name {
            fn try_register_custom_resource(&mut self, _: Box<dyn Any>) -> Result<ResourceId, RegisterResourceError> { Ok(ResourceId) }
            fn unregister_resource(&mut self, _: ResourceId) {}
            fn renderer_specific_context(&self) -> Option<Box<dyn Any>> {
                Some(Box::new(Observed { backend: self.0.backend, color: self.0.color,
                    alpha: self.0.alpha, features: self.0.features, limits: self.0.limits }))
            }
        }
        impl WindowRenderer for $name {
            type ScenePainter<'a> = ();
            fn resume<F: FnOnce() + 'static>(&mut self, _: Arc<dyn WindowHandle>, _: u32, _: u32, f: F) { f() }
            fn complete_resume(&mut self) -> bool { true }
            fn suspend(&mut self) {}
            fn is_active(&self) -> bool { true }
            fn is_pending(&self) -> bool { false }
            fn set_size(&mut self, _: u32, _: u32) {}
            fn render<F: FnOnce(&mut Self::ScenePainter<'_>)>(&mut self, f: F) { f(&mut ()) }
        }
    };
}
renderer!(VelloWindowRenderer, GpuOptions, "vello", |o: GpuOptions| (o.config, o.features, o.limits));
renderer!(VelloHybridWindowRenderer, GpuOptions, "hybrid", |o: GpuOptions| (o.config, o.features, o.limits));
renderer!(VelloCpuWindowRenderer, RendererConfig, "cpu", |o| (o, false, false));
renderer!(SkiaWindowRenderer, RendererConfig, "skia", |o| (o, false, false));
renderer!(SkiaRasterWindowRenderer, RendererConfig, "raster", |o| (o, false, false));
pub mod raster { pub use crate::SkiaRasterWindowRenderer; }
'''

MAIN = r'''
fn main() {
    use actual::{AetherNativeWindowRenderer, RendererOptions};
    let mut options = RendererOptions::default();
    options.base_color = Some(Color(19));
    options.alpha_mode = Some(CompositeAlphaMode::Opaque);
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    { options.features = Some(etalon::Features); options.limits = Some(etalon::Limits); }
    let renderer = AetherNativeWindowRenderer::with_options(options);
    let observed = renderer.renderer_specific_context().unwrap().downcast::<Observed>().unwrap();
    let backend = std::env::args().nth(1).unwrap();
    let gpu = backend == "vello" || backend == "hybrid";
    assert_eq!(*observed, Observed { backend: Box::leak(backend.into_boxed_str()),
        color: Some(Color(19)), alpha: Some(CompositeAlphaMode::Opaque), features: gpu, limits: gpu });
}
'''


def run(*args):
    return subprocess.run(args, capture_output=True, text=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compile-only", action="store_true", help="Compile every selection without claiming configuration-forwarding execution")
    args = parser.parse_args()
    manifest = tomllib.loads((NATIVE / "Cargo.toml").read_text())
    tracing = manifest["features"]["tracing"]
    assert "blitz-html?/tracing" in tracing and "blitz-net?/tracing" in tracing
    for dependency in ("blitz-net", "tokio"):
        assert dependency not in manifest["dependencies"], dependency
        assert dependency in manifest["target"]['cfg(not(target_arch = "wasm32"))']["dependencies"]

    with tempfile.TemporaryDirectory(prefix="aether-renderer-features-") as tmp:
        work = Path(tmp)
        (work / "cfg_if.rs").write_text(CFG_IF)
        macro_lib = work / "libcfg_if.rlib"
        result = run("rustc", "--edition=2024", "--crate-type=rlib", "--crate-name=cfg_if", str(work / "cfg_if.rs"), "-o", str(macro_lib))
        assert result.returncode == 0, result.stderr
        renderer = (NATIVE / "src/aether_renderer.rs").as_posix()
        fixture = work / "fixture.rs"
        fixture.write_text(CONTRACTS + f'\n#[path = "{renderer}"] mod actual;\n' + MAIN)
        for name, features, expected in CASES:
            executable = work / f"{name}.exe"
            command = ["rustc", "--edition=2024", "-Awarnings", str(fixture), "--extern", f"cfg_if={macro_lib}", "-o", str(executable)]
            enabled = set(features)
            pending = list(features)
            while pending:
                feature = pending.pop()
                for edge in manifest["features"].get(feature, []):
                    if edge in manifest["features"] and edge not in enabled:
                        enabled.add(edge)
                        pending.append(edge)
            for feature in sorted(enabled):
                command += ["--cfg", f'feature="{feature}"']
            result = run(*command)
            assert result.returncode == 0, f"{name}: {result.stderr}"
            if args.compile_only:
                print(f"COMPILE PASS {name}; execution not requested")
            else:
                result = run(str(executable), expected)
                assert result.returncode == 0, f"{name}: {result.stderr}"
                print(f"PASS {name} -> {expected}")
        result = run("rustc", "--edition=2024", str(fixture), "--extern", f"cfg_if={macro_lib}", "-o", str(work / "no-renderer.exe"))
        assert result.returncode != 0 and "At least one renderer feature must be enabled" in result.stderr
        print("PASS no renderer -> explicit diagnostic")
    print("PASS optional tracing and target-only network dependency edges")


if __name__ == "__main__":
    main()
