// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Host-side projection of semantic UI nodes onto original spatial design
//! archetypes. Topology is described only by geometric properties.

/// Spatial layout used to place semantic UI nodes in 10D coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpatialManifoldTopology {
    PlanarDesktopGrid,
    CylindricalGlassArc,
    IsometricWireframeCityGrid,
    OrbitalVolumetricAmphitheatre10d,
}

/// Project a node index onto a 10D coordinate for the given topology.
pub fn project_semantic_node(
    topology: SpatialManifoldTopology,
    index: usize,
    count: usize,
) -> [f32; 10] {
    let count = count.max(1);
    let t = if count <= 1 {
        0.0
    } else {
        index as f32 / (count.saturating_sub(1) as f32)
    };
    let angle = (index as f32 / count as f32) * core::f32::consts::TAU;
    match topology {
        SpatialManifoldTopology::PlanarDesktopGrid => {
            let cols = 4usize;
            let x = (index % cols) as f32 * 0.55 - 0.82;
            let y = (index / cols) as f32 * -0.42 + 0.55;
            [x, y, 0.0, t, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]
        }
        SpatialManifoldTopology::CylindricalGlassArc => {
            let yaw = -1.15 + t * 2.3;
            [
                yaw.sin() * 1.35,
                0.08 * ((index % 3) as f32 - 1.0),
                -yaw.cos() * 1.35 + 0.4,
                yaw,
                0.22,
                0.85,
                1.0,
                0.12,
                0.0,
                1.0,
            ]
        }
        SpatialManifoldTopology::IsometricWireframeCityGrid => {
            let col = (index % 4) as f32;
            let row = (index / 4) as f32;
            let height = 0.35 + (index % 5) as f32 * 0.18;
            [
                (col - row) * 0.42,
                height,
                (col + row) * 0.24,
                t,
                height,
                0.0,
                1.0,
                0.0,
                1.0,
                1.0,
            ]
        }
        SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d => [
            angle.cos() * 1.6,
            0.15 * (index as f32 * 0.7).sin(),
            angle.sin() * 1.6,
            angle,
            0.55,
            0.8,
            1.0,
            0.25,
            t,
            1.0,
        ],
    }
}

/// Project a whole node list, preserving identity slots 0..n.
pub fn project_semantic_scene_with_cartridge(
    topology: SpatialManifoldTopology,
    count: usize,
) -> [[f32; 10]; 64] {
    let mut out = [[0.0; 10]; 64];
    let n = count.min(64);
    for index in 0..n {
        out[index] = project_semantic_node(topology, index, n.max(1));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbital_nodes_span_a_full_ring() {
        let nodes = project_semantic_scene_with_cartridge(
            SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d,
            8,
        );
        let xs: Vec<f32> = (0..8).map(|i| nodes[i][0]).collect();
        assert!(xs.iter().any(|x| *x < -0.5));
        assert!(xs.iter().any(|x| *x > 0.5));
        assert!(nodes[0].iter().all(|a| a.is_finite()));
    }
}
