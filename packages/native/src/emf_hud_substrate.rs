// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! EMF-first, zero-heap HUD substrate matching `etalon-render::{emf,hud,volumetric}`.
//!
//! Layout, surface optics, focus/salience fields, and wave-coupled animation live
//! here as GPU-ready arrays. CSS in host apps is a Parley / IME / AccessKit /
//! [`crate::RendererFallbackPolicy::CpuOnFailure`] fallback, not the primary look.

/// Matches `etalon_render::hud::MAX_UI_NODES`.
pub const MAX_UI_NODES: usize = 256;
/// Matches `etalon_render::volumetric` HOLO node cap (`0x0306`).
pub const MAX_HOLOGRAPHIC_GPU_NODES: usize = 64;
/// 16×16 wavefield slice (one cell per HUD node).
pub const MAX_EMF_CELLS: usize = 256;
pub const EMF_CELL_BYTES: usize = 64;
pub const HOLO_NODE_BYTES: usize = 96;
pub const NO_UI_NODE: u16 = u16::MAX;

/// Per-cell Q42EMF64 field record (64 bytes = 3 field + 10 manifold + 3 Stokes).
///
/// Layout matches `etalon_render::emf::EmfFieldCell` and `EMF_VOLUMETRIC_WGSL`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmfFieldCell {
    pub amplitude: f32,
    pub phase: f32,
    pub frequency: f32,
    pub scale: f32,
    pub attention_depth: f32,
    pub epistemic_weight: f32,
    pub topological_spin: f32,
    pub temporal_decay: f32,
    pub entropy_bias: f32,
    pub spatial_phase: f32,
    pub recurrence_frequency: f32,
    pub density_threshold: f32,
    pub manifold_curvature: f32,
    pub stokes_q: f32,
    pub stokes_u: f32,
    pub stokes_v: f32,
}

const _: () = {
    assert!(core::mem::size_of::<EmfFieldCell>() == 64);
    assert!(core::mem::offset_of!(EmfFieldCell, attention_depth) == 16);
    assert!(core::mem::offset_of!(EmfFieldCell, epistemic_weight) == 20);
    assert!(core::mem::offset_of!(EmfFieldCell, topological_spin) == 24);
    assert!(core::mem::offset_of!(EmfFieldCell, manifold_curvature) == 48);
    assert!(core::mem::offset_of!(EmfFieldCell, stokes_q) == 52);
    assert!(core::mem::offset_of!(EmfFieldCell, stokes_u) == 56);
    assert!(core::mem::offset_of!(EmfFieldCell, stokes_v) == 60);
};

impl Default for EmfFieldCell {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            phase: 0.0,
            frequency: 0.0,
            scale: 1.0,
            attention_depth: 0.0,
            epistemic_weight: 1.0,
            topological_spin: 0.0,
            temporal_decay: 0.0,
            entropy_bias: 0.0,
            spatial_phase: 0.0,
            recurrence_frequency: 0.0,
            density_threshold: 0.0,
            manifold_curvature: 0.0,
            stokes_q: 0.0,
            stokes_u: 0.0,
            stokes_v: 0.0,
        }
    }
}

impl EmfFieldCell {
    pub fn as_bytes(&self) -> [u8; EMF_CELL_BYTES] {
        let lanes = [
            self.amplitude,
            self.phase,
            self.frequency,
            self.scale,
            self.attention_depth,
            self.epistemic_weight,
            self.topological_spin,
            self.temporal_decay,
            self.entropy_bias,
            self.spatial_phase,
            self.recurrence_frequency,
            self.density_threshold,
            self.manifold_curvature,
            self.stokes_q,
            self.stokes_u,
            self.stokes_v,
        ];
        let mut bytes = [0u8; EMF_CELL_BYTES];
        for (slot, lane) in bytes.chunks_exact_mut(4).zip(lanes) {
            slot.copy_from_slice(&lane.to_le_bytes());
        }
        bytes
    }

    pub fn degree_of_polarization(&self) -> f32 {
        let pol_mag = (self.stokes_q * self.stokes_q
            + self.stokes_u * self.stokes_u
            + self.stokes_v * self.stokes_v)
            .sqrt();
        (pol_mag / (self.amplitude.abs() + 1.0e-5)).clamp(0.0, 1.0)
    }
}

/// 96-byte std430 SDF primitive matching `etalon_render::volumetric::HolographicNodeGpu`.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HolographicNodeGpu {
    pub position_scale: [f32; 4],
    pub half_extents_primitive: [f32; 4],
    pub frequency_amplitude: [f32; 4],
    pub manifold_tensor: [f32; 4],
    pub connection_endpoint: [f32; 4],
    pub emissive_color: [f32; 4],
}

const _: () = assert!(core::mem::size_of::<HolographicNodeGpu>() == 96);

impl Default for HolographicNodeGpu {
    fn default() -> Self {
        Self {
            position_scale: [0.0, 0.0, 0.0, 1.0],
            half_extents_primitive: [0.5, 0.5, 0.5, 0.0],
            frequency_amplitude: [1.0, 1.0, 0.0, 0.0],
            manifold_tensor: [1.0, 0.0, 0.0, 0.5],
            connection_endpoint: [0.0, 0.0, 0.0, 0.0],
            emissive_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl HolographicNodeGpu {
    pub fn as_bytes(&self) -> [u8; HOLO_NODE_BYTES] {
        let mut bytes = [0u8; HOLO_NODE_BYTES];
        let lanes = [
            self.position_scale,
            self.half_extents_primitive,
            self.frequency_amplitude,
            self.manifold_tensor,
            self.connection_endpoint,
            self.emissive_color,
        ];
        for (slot, lane) in bytes.chunks_exact_mut(16).zip(lanes) {
            for (word, axis) in slot.chunks_exact_mut(4).zip(lane) {
                word.copy_from_slice(&axis.to_le_bytes());
            }
        }
        bytes
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row = 0,
    Column = 1,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SizeRule {
    Fixed(f32),
    Percent(f32),
    #[default]
    Auto,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Edges {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Edges {
    pub const fn all(v: f32) -> Self {
        Self {
            left: v,
            right: v,
            top: v,
            bottom: v,
        }
    }
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiLayoutStyle {
    pub direction: FlexDirection,
    pub width: SizeRule,
    pub height: SizeRule,
    pub padding: Edges,
    pub margin: Edges,
    pub gap: f32,
}

impl Default for UiLayoutStyle {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            width: SizeRule::Auto,
            height: SizeRule::Auto,
            padding: Edges::default(),
            margin: Edges::default(),
            gap: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ComputedUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HudVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiLayoutNode {
    pub parent: u16,
    pub first_child: u16,
    pub next_sibling: u16,
    pub style: UiLayoutStyle,
    pub computed: ComputedUiRect,
    pub bg_color: [f32; 4],
}

impl Default for UiLayoutNode {
    fn default() -> Self {
        Self {
            parent: NO_UI_NODE,
            first_child: NO_UI_NODE,
            next_sibling: NO_UI_NODE,
            style: UiLayoutStyle::default(),
            computed: ComputedUiRect::default(),
            bg_color: [0.0; 4],
        }
    }
}

/// Zero-heap flat-array HUD constraint solver (`MAX_UI_NODES = 256`).
pub struct UiLayoutTree {
    nodes: [UiLayoutNode; MAX_UI_NODES],
    count: usize,
}

impl Default for UiLayoutTree {
    fn default() -> Self {
        Self::new()
    }
}

impl UiLayoutTree {
    pub const fn new() -> Self {
        Self {
            nodes: [UiLayoutNode {
                parent: NO_UI_NODE,
                first_child: NO_UI_NODE,
                next_sibling: NO_UI_NODE,
                style: UiLayoutStyle {
                    direction: FlexDirection::Row,
                    width: SizeRule::Auto,
                    height: SizeRule::Auto,
                    padding: Edges {
                        left: 0.0,
                        right: 0.0,
                        top: 0.0,
                        bottom: 0.0,
                    },
                    margin: Edges {
                        left: 0.0,
                        right: 0.0,
                        top: 0.0,
                        bottom: 0.0,
                    },
                    gap: 0.0,
                },
                computed: ComputedUiRect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
                bg_color: [0.0; 4],
            }; MAX_UI_NODES],
            count: 0,
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn add_node(
        &mut self,
        parent: u16,
        style: UiLayoutStyle,
        bg_color: [f32; 4],
    ) -> Option<u16> {
        if self.count >= MAX_UI_NODES {
            return None;
        }
        let id = self.count as u16;
        self.count += 1;
        let node = &mut self.nodes[id as usize];
        node.parent = parent;
        node.first_child = NO_UI_NODE;
        node.next_sibling = NO_UI_NODE;
        node.style = style;
        node.bg_color = bg_color;
        if parent != NO_UI_NODE && (parent as usize) < self.count {
            let mut prev = self.nodes[parent as usize].first_child;
            if prev == NO_UI_NODE {
                self.nodes[parent as usize].first_child = id;
            } else {
                while self.nodes[prev as usize].next_sibling != NO_UI_NODE {
                    prev = self.nodes[prev as usize].next_sibling;
                }
                self.nodes[prev as usize].next_sibling = id;
            }
        }
        Some(id)
    }

    pub fn solve(&mut self, viewport_w: f32, viewport_h: f32) {
        if self.count == 0 {
            return;
        }
        self.nodes[0].computed = ComputedUiRect {
            x: 0.0,
            y: 0.0,
            width: viewport_w,
            height: viewport_h,
        };
        for i in 0..self.count {
            self.layout_children(i as u16);
        }
    }

    fn layout_children(&mut self, parent_id: u16) {
        let parent = self.nodes[parent_id as usize];
        let p_rect = parent.computed;
        let mut cur_x = p_rect.x + parent.style.padding.left;
        let mut cur_y = p_rect.y + parent.style.padding.top;
        let avail_w = (p_rect.width - parent.style.padding.horizontal()).max(0.0);
        let avail_h = (p_rect.height - parent.style.padding.vertical()).max(0.0);
        let mut child = parent.first_child;
        while child != NO_UI_NODE {
            let c = &mut self.nodes[child as usize];
            let w = match c.style.width {
                SizeRule::Fixed(f) => f,
                SizeRule::Percent(p) => avail_w * (p * 0.01).clamp(0.0, 1.0),
                SizeRule::Auto => (avail_w - (cur_x - p_rect.x)).max(0.0),
            };
            let h = match c.style.height {
                SizeRule::Fixed(f) => f,
                SizeRule::Percent(p) => avail_h * (p * 0.01).clamp(0.0, 1.0),
                SizeRule::Auto => 32.0,
            };
            c.computed.x = cur_x + c.style.margin.left;
            c.computed.y = cur_y + c.style.margin.top;
            c.computed.width = w;
            c.computed.height = h;
            match parent.style.direction {
                FlexDirection::Row => {
                    cur_x += w + c.style.margin.horizontal() + parent.style.gap;
                }
                FlexDirection::Column => {
                    cur_y += h + c.style.margin.vertical() + parent.style.gap;
                }
            }
            child = c.next_sibling;
        }
    }

    pub fn generate_quad_vertices(&self, out: &mut [HudVertex]) -> usize {
        let mut v_idx = 0;
        for i in 0..self.count {
            if v_idx + 6 > out.len() {
                break;
            }
            let rect = self.nodes[i].computed;
            let col = self.nodes[i].bg_color;
            if col[3] <= 0.0 {
                continue;
            }
            let x0 = rect.x;
            let y0 = rect.y;
            let x1 = rect.x + rect.width;
            let y1 = rect.y + rect.height;
            out[v_idx] = HudVertex {
                pos: [x0, y0],
                uv: [0.0, 0.0],
                color: col,
            };
            out[v_idx + 1] = HudVertex {
                pos: [x1, y0],
                uv: [1.0, 0.0],
                color: col,
            };
            out[v_idx + 2] = HudVertex {
                pos: [x1, y1],
                uv: [1.0, 1.0],
                color: col,
            };
            out[v_idx + 3] = HudVertex {
                pos: [x0, y0],
                uv: [0.0, 0.0],
                color: col,
            };
            out[v_idx + 4] = HudVertex {
                pos: [x1, y1],
                uv: [1.0, 1.0],
                color: col,
            };
            out[v_idx + 5] = HudVertex {
                pos: [x0, y1],
                uv: [0.0, 1.0],
                color: col,
            };
            v_idx += 6;
        }
        v_idx
    }
}

/// Combined EMF wavefield + HOLO SDF nodes + zero-heap HUD layout tree.
pub struct EtalonEmfUiSubstrate {
    pub layout: UiLayoutTree,
    cells: [EmfFieldCell; MAX_EMF_CELLS],
    cell_count: usize,
    holo: [HolographicNodeGpu; MAX_HOLOGRAPHIC_GPU_NODES],
    holo_count: u32,
}

impl Default for EtalonEmfUiSubstrate {
    fn default() -> Self {
        Self::new()
    }
}

impl EtalonEmfUiSubstrate {
    pub fn new() -> Self {
        Self {
            layout: UiLayoutTree::new(),
            cells: [EmfFieldCell::default(); MAX_EMF_CELLS],
            cell_count: 0,
            holo: [HolographicNodeGpu::default(); MAX_HOLOGRAPHIC_GPU_NODES],
            holo_count: 0,
        }
    }

    pub fn cells(&self) -> &[EmfFieldCell] {
        &self.cells[..self.cell_count]
    }

    pub fn holo_nodes(&self) -> &[HolographicNodeGpu] {
        &self.holo[..self.holo_count as usize]
    }

    pub fn push_cell(&mut self, cell: EmfFieldCell) -> Option<usize> {
        if self.cell_count >= MAX_EMF_CELLS {
            return None;
        }
        let index = self.cell_count;
        self.cells[index] = cell;
        self.cell_count += 1;
        Some(index)
    }

    pub fn push_holo_node(&mut self, node: HolographicNodeGpu) -> Option<u32> {
        if self.holo_count as usize >= MAX_HOLOGRAPHIC_GPU_NODES {
            return None;
        }
        let index = self.holo_count;
        self.holo[index as usize] = node;
        self.holo_count += 1;
        Some(index)
    }

    /// Chrome layout: full-viewport canvas with a thin toolbar strip (CPU fallback).
    pub fn rebuild_workspace_chrome(&mut self, viewport_w: f32, viewport_h: f32) {
        self.layout.clear();
        let root = self
            .layout
            .add_node(
                NO_UI_NODE,
                UiLayoutStyle {
                    direction: FlexDirection::Column,
                    width: SizeRule::Fixed(viewport_w),
                    height: SizeRule::Fixed(viewport_h),
                    ..UiLayoutStyle::default()
                },
                [0.03, 0.05, 0.08, 1.0],
            )
            .expect("root HUD node");
        let _toolbar = self.layout.add_node(
            root,
            UiLayoutStyle {
                width: SizeRule::Percent(100.0),
                height: SizeRule::Fixed((viewport_h * 0.10).clamp(36.0, 64.0)),
                ..UiLayoutStyle::default()
            },
            [0.06, 0.10, 0.16, 0.92],
        );
        let _canvas = self.layout.add_node(
            root,
            UiLayoutStyle {
                width: SizeRule::Percent(100.0),
                height: SizeRule::Percent(90.0),
                ..UiLayoutStyle::default()
            },
            [0.04, 0.07, 0.12, 0.35],
        );
        self.layout.solve(viewport_w, viewport_h);
    }

    pub fn pack_emf_cells(&self) -> [u8; MAX_EMF_CELLS * EMF_CELL_BYTES] {
        let mut bytes = [0u8; MAX_EMF_CELLS * EMF_CELL_BYTES];
        for (index, cell) in self.cells.iter().enumerate() {
            let slot = &mut bytes[index * EMF_CELL_BYTES..(index + 1) * EMF_CELL_BYTES];
            slot.copy_from_slice(&cell.as_bytes());
        }
        bytes
    }

    pub fn pack_holo_nodes(&self) -> [u8; MAX_HOLOGRAPHIC_GPU_NODES * HOLO_NODE_BYTES] {
        let mut bytes = [0u8; MAX_HOLOGRAPHIC_GPU_NODES * HOLO_NODE_BYTES];
        for (index, node) in self.holo.iter().enumerate() {
            let slot = &mut bytes[index * HOLO_NODE_BYTES..(index + 1) * HOLO_NODE_BYTES];
            slot.copy_from_slice(&node.as_bytes());
        }
        bytes
    }

    pub fn generate_hud_vertices(&self, out: &mut [HudVertex]) -> usize {
        self.layout.generate_quad_vertices(out)
    }

    /// CIE 1931 XYZ from six spectral bands at 400, 450, 500, 550, 600, 650 nm.
    pub fn cie1931_xyz_6band(bands: [f32; 6]) -> [f32; 3] {
        // CIE 1931 2° colour-matching functions (tabular, six sample wavelengths).
        const XBAR: [f32; 6] = [0.0143, 0.3362, 0.0049, 0.4334, 1.0622, 0.2835];
        const YBAR: [f32; 6] = [0.0004, 0.0380, 0.3230, 0.9950, 0.6310, 0.1070];
        const ZBAR: [f32; 6] = [0.0679, 1.7721, 0.2720, 0.0087, 0.0008, 0.0000];
        let mut xyz = [0.0f32; 3];
        for i in 0..6 {
            let s = bands[i].max(0.0);
            xyz[0] += s * XBAR[i];
            xyz[1] += s * YBAR[i];
            xyz[2] += s * ZBAR[i];
        }
        // 50 nm sample spacing × 1e-9 is absorbed into a unit-scale normaliser.
        let norm = (xyz[0] + xyz[1] + xyz[2]).max(1.0e-6);
        [xyz[0] / norm, xyz[1] / norm, xyz[2] / norm]
    }

    pub fn cell_cie_xyz(&self, index: usize) -> [f32; 3] {
        let cell = self.cells.get(index).copied().unwrap_or_default();
        let sigma = if cell.recurrence_frequency > 0.0 {
            cell.recurrence_frequency
        } else {
            cell.phase.abs() % 6.0
        };
        let mut bands = [0.0f32; 6];
        for (i, band) in bands.iter_mut().enumerate() {
            let centre = i as f32;
            let d = sigma - centre;
            *band = cell.amplitude.abs() * (-d * d * 0.45).exp();
        }
        Self::cie1931_xyz_6band(bands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emf_cell_is_64_bytes_with_stokes_tail() {
        assert_eq!(core::mem::size_of::<EmfFieldCell>(), 64);
        let cell = EmfFieldCell {
            stokes_q: 0.2,
            stokes_u: 0.1,
            stokes_v: 0.4,
            amplitude: 1.0,
            ..EmfFieldCell::default()
        };
        assert!(cell.degree_of_polarization() > 0.0);
        assert_eq!(cell.as_bytes().len(), 64);
    }

    #[test]
    fn holographic_node_is_96_bytes() {
        assert_eq!(core::mem::size_of::<HolographicNodeGpu>(), 96);
        assert_eq!(HolographicNodeGpu::default().as_bytes().len(), 96);
    }

    #[test]
    fn layout_tree_stays_within_256_and_emits_hud_vertices() {
        let mut tree = UiLayoutTree::new();
        let root = tree
            .add_node(NO_UI_NODE, UiLayoutStyle::default(), [1.0, 1.0, 1.0, 1.0])
            .unwrap();
        tree.add_node(
            root,
            UiLayoutStyle {
                width: SizeRule::Fixed(80.0),
                height: SizeRule::Fixed(24.0),
                ..UiLayoutStyle::default()
            },
            [0.2, 0.8, 0.7, 1.0],
        );
        tree.solve(320.0, 200.0);
        let mut verts = [HudVertex {
            pos: [0.0; 2],
            uv: [0.0; 2],
            color: [0.0; 4],
        }; 32];
        let count = tree.generate_quad_vertices(&mut verts);
        assert_eq!(count, 12);
        assert_eq!(MAX_UI_NODES, 256);
    }

    #[test]
    fn substrate_packs_full_gpu_buffers_and_cie_xyz() {
        let mut substrate = EtalonEmfUiSubstrate::new();
        substrate.rebuild_workspace_chrome(1280.0, 720.0);
        substrate
            .push_cell(EmfFieldCell {
                amplitude: 0.8,
                recurrence_frequency: 3.0,
                stokes_v: 0.5,
                ..EmfFieldCell::default()
            })
            .unwrap();
        substrate.push_holo_node(HolographicNodeGpu::default()).unwrap();
        assert_eq!(
            substrate.pack_emf_cells().len(),
            MAX_EMF_CELLS * EMF_CELL_BYTES
        );
        assert_eq!(
            substrate.pack_holo_nodes().len(),
            MAX_HOLOGRAPHIC_GPU_NODES * HOLO_NODE_BYTES
        );
        let xyz = substrate.cell_cie_xyz(0);
        assert!(xyz[0] >= 0.0 && xyz[1] >= 0.0 && xyz[2] >= 0.0);
        let mut verts = [HudVertex {
            pos: [0.0; 2],
            uv: [0.0; 2],
            color: [0.0; 4],
        }; 32];
        assert!(substrate.generate_hud_vertices(&mut verts) >= 6);
    }
}
