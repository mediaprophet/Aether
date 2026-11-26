// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Validated QualiaDB phase-space payloads for a host-owned viewport.

use qualia_core_db::container_10d::{
    TypedPayloadError, TypedPayloadResolver,
    holographic_phase_section::{HolographicPhaseError, decode_holographic_phase_section_into},
};
pub use qualia_core_db::{
    container_10d::{
        EulerianWavefieldRecord, FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE,
        HOLOGRAPHIC_MAX_RAYMARCH_STEPS, HOLOGRAPHIC_PHASE_HEADER_SIZE, HolographicPhaseHeader,
    },
    q42::m64_matter::M64HoloRecord,
};

const _: () = assert!(FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE == 0x0306);
const _: () = assert!(HOLOGRAPHIC_PHASE_HEADER_SIZE == 64);
const _: () = assert!(std::mem::size_of::<HolographicPhaseHeader>() == 64);
const _: () = assert!(std::mem::align_of::<HolographicPhaseHeader>() == 64);
const _: () = assert!(HOLOGRAPHIC_MAX_RAYMARCH_STEPS == 32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Holographic10dViewportState {
    HeaderVerified,
    MissingHoloSection,
    AmbiguousHoloSection,
    RaymarchBudgetExceeded { requested: u32, max: u32 },
    InvalidContainer(TypedPayloadError),
    InvalidPayload(HolographicPhaseError),
}

/// A verified phase-space header with caller-owned decoded GPU staging records.
pub struct Holographic10dViewportSpec<'a> {
    header: HolographicPhaseHeader,
    segments: &'a [M64HoloRecord],
    wavefields: &'a [EulerianWavefieldRecord],
}

impl<'a> Holographic10dViewportSpec<'a> {
    /// Bind the sole canonical HOLO field in a validated MID2 resolver.
    pub fn from_10d(
        view: &TypedPayloadResolver<'_, '_>,
        segments: &'a mut [M64HoloRecord],
        wavefields: &'a mut [EulerianWavefieldRecord],
    ) -> Result<Self, Holographic10dViewportState> {
        let mut fields = view.payloads_of_kind(FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE);
        let field = fields
            .next()
            .ok_or(Holographic10dViewportState::MissingHoloSection)?
            .map_err(Holographic10dViewportState::InvalidContainer)?;
        if fields.next().is_some() {
            return Err(Holographic10dViewportState::AmbiguousHoloSection);
        }
        Self::from_payload(field.record.kind, field.bytes, segments, wavefields)
    }

    pub fn from_payload(
        kind: u16,
        bytes: &[u8],
        segments: &'a mut [M64HoloRecord],
        wavefields: &'a mut [EulerianWavefieldRecord],
    ) -> Result<Self, Holographic10dViewportState> {
        use Holographic10dViewportState::*;
        if kind != FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE {
            return Err(MissingHoloSection);
        }
        let (header, segment_count, wavefield_count) = decode_holographic_phase_section_into(
            bytes, segments, wavefields,
        )
        .map_err(|error| match error {
            HolographicPhaseError::RaymarchStepsExceeded { got, max } if got > max => {
                RaymarchBudgetExceeded {
                    requested: u32::from(got),
                    max: u32::from(max),
                }
            }
            _ => InvalidPayload(error),
        })?;
        if !(0..3).all(|axis| {
            let min = header.proxy_aabb_min[axis];
            let max = header.proxy_aabb_max[axis];
            min.is_finite() && max.is_finite() && min <= max
        }) {
            return Err(InvalidPayload(HolographicPhaseError::InvalidBounds));
        }
        Ok(Self {
            header,
            segments: &segments[..segment_count],
            wavefields: &wavefields[..wavefield_count],
        })
    }

    pub fn state(&self) -> Holographic10dViewportState {
        Holographic10dViewportState::HeaderVerified
    }
    pub fn header(&self) -> &HolographicPhaseHeader {
        &self.header
    }
    pub fn segments(&self) -> &[M64HoloRecord] {
        self.segments
    }
    pub fn wavefields(&self) -> &[EulerianWavefieldRecord] {
        self.wavefields
    }

    pub fn raymarch_steps(&self, requested: u32) -> Result<u32, Holographic10dViewportState> {
        let max = u32::from(self.header.max_raymarch_steps);
        if requested > max {
            return Err(Holographic10dViewportState::RaymarchBudgetExceeded { requested, max });
        }
        Ok(requested)
    }

    /// Convert decoded HOLO segments and wavefields into 96-byte std430 GPU nodes.
    /// At most 64 SDF primitives are staged per pass.
    pub fn holographic_nodes_gpu(&self) -> ([HolographicNodeGpu; MAX_HOLOGRAPHIC_GPU_NODES], u32) {
        stage_holographic_nodes_gpu(self.segments, self.wavefields)
    }

    /// Build twelve outward-facing triangles for the proxy bounding lens.
    pub fn proxy_lens(&self) -> [[[f32; 3]; 3]; 12] {
        let [x, y, z] = self.header.proxy_aabb_min;
        let [u, v, w] = self.header.proxy_aabb_max;
        let corners = [
            [x, y, z],
            [u, y, z],
            [u, v, z],
            [x, v, z],
            [x, y, w],
            [u, y, w],
            [u, v, w],
            [x, v, w],
        ];
        [
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [3, 7, 6],
            [3, 6, 2],
            [0, 4, 7],
            [0, 7, 3],
            [1, 2, 6],
            [1, 6, 5],
        ]
        .map(|face| face.map(|index| corners[index]))
    }
}

/// Maximum SDF primitives uploaded to `@group(1) @binding(0)`.
pub const MAX_HOLOGRAPHIC_GPU_NODES: usize = 64;

/// 96-byte std430 GPU record matching `etalon_render::volumetric::HolographicNodeGpu`.
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
const _: () = assert!(core::mem::align_of::<HolographicNodeGpu>() == 16);

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
    pub fn as_bytes(&self) -> [u8; 96] {
        let mut bytes = [0u8; 96];
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

/// Stage up to 64 SDF primitives from HOLO segments and Eulerian wavefields.
pub fn stage_holographic_nodes_gpu(
    segments: &[M64HoloRecord],
    wavefields: &[EulerianWavefieldRecord],
) -> ([HolographicNodeGpu; MAX_HOLOGRAPHIC_GPU_NODES], u32) {
    let mut nodes = [HolographicNodeGpu::default(); MAX_HOLOGRAPHIC_GPU_NODES];
    let mut count = 0usize;
    for record in segments.iter().take(MAX_HOLOGRAPHIC_GPU_NODES) {
        nodes[count] = node_from_m64(record);
        count += 1;
    }
    for wave in wavefields
        .iter()
        .take(MAX_HOLOGRAPHIC_GPU_NODES.saturating_sub(count))
    {
        nodes[count] = node_from_wavefield(wave, count as u8);
        count += 1;
    }
    (nodes, count as u32)
}

fn node_from_m64(record: &M64HoloRecord) -> HolographicNodeGpu {
    let x = f16_to_f32(record.local_origin[0]);
    let y = f16_to_f32(record.local_origin[1]);
    let z = f16_to_f32(record.local_origin[2]);
    let min_scale = f16_to_f32(record.scale_bounds[0]).abs().max(0.05);
    let max_scale = f16_to_f32(record.scale_bounds[1]).abs().max(min_scale);
    let scale = (min_scale + max_scale) * 0.5;
    let primitive = f32::from(record.metric_variety % 5);
    let omega = record.natural_freq_omega.map(f16_to_f32);
    let damping = f16_to_f32(record.damping_decay);
    let spin = f32::from(record.topological_spin) / 32768.0;
    let rgb = [
        f32::from(record.spectral_spd_base[0]) / 255.0,
        f32::from(record.spectral_spd_base[1]) / 255.0,
        f32::from(record.spectral_spd_base[2]) / 255.0,
    ];
    HolographicNodeGpu {
        position_scale: [x, y, z, scale],
        half_extents_primitive: [scale * 0.5, scale * 0.5, scale * 0.5, primitive],
        frequency_amplitude: [omega[0], omega[1], omega[2], damping],
        manifold_tensor: [
            1.0,
            spin,
            damping,
            f32::from(record.semantic_node_idx) / 255.0,
        ],
        connection_endpoint: [
            0.0,
            0.0,
            0.0,
            f32::from(record.semantic_node_idx),
        ],
        emissive_color: [rgb[0], rgb[1], rgb[2], 1.0],
    }
}

fn node_from_wavefield(wave: &EulerianWavefieldRecord, slot: u8) -> HolographicNodeGpu {
    HolographicNodeGpu {
        position_scale: [
            wave.stream_velocity_m_s[0] * 0.01,
            wave.bed_elevation_m * 0.01,
            wave.stream_velocity_m_s[2] * 0.01,
            wave.water_depth_m.max(0.2) * 0.05,
        ],
        half_extents_primitive: [0.4, 0.15, 0.4, 3.0],
        frequency_amplitude: [
            f16_to_f32(wave.mode_frequencies_f16[0]),
            f16_to_f32(wave.mode_amplitudes_f16[0]),
            wave.vorticity_curl_rad_s,
            0.2,
        ],
        manifold_tensor: [0.8, wave.vorticity_curl_rad_s, 0.1, 0.4],
        connection_endpoint: [0.0, 0.0, 0.0, f32::from(slot)],
        emissive_color: [0.25, 0.55, 0.95, 0.85],
    }
}

fn f16_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits >> 15);
    let exp = u32::from((bits >> 10) & 0x1f);
    let frac = u32::from(bits & 0x3ff);
    let ieee = if exp == 0 {
        if frac == 0 {
            sign << 31
        } else {
            let mut e = 127u32 - 15;
            let mut m = frac;
            while m & 0x400 == 0 {
                m <<= 1;
                e -= 1;
            }
            (sign << 31) | (e << 23) | ((m & 0x3ff) << 13)
        }
    } else if exp == 31 {
        (sign << 31) | 0x7f80_0000 | (frac << 13)
    } else {
        (sign << 31) | ((exp + (127 - 15)) << 23) | (frac << 13)
    };
    f32::from_bits(ieee)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qualia_core_db::container_10d::holographic_phase_section::encode_holographic_phase_section;

    fn payload() -> [u8; 64] {
        let mut bytes = [0; 64];
        let header = HolographicPhaseHeader {
            magic: *b"HOLO",
            version: 1,
            flags: 0,
            holo_segment_count: 0,
            wavefield_count: 0,
            max_harmonic_order: 4,
            max_raymarch_steps: 32,
            primary_constituent_lut_idx: 0,
            proxy_lens_kind: 0,
            proxy_aabb_min: [-1.0, -2.0, -3.0],
            proxy_aabb_max: [1.0, 2.0, 3.0],
            sdf_smooth_union_radius: 0.0,
            reference_sigma: 0.5,
            nquin_subject_hash: 1,
            payload_crc32c: 0,
            header_crc32c: 0,
        };
        assert_eq!(
            encode_holographic_phase_section(header, &[], &[], &mut bytes).unwrap(),
            64
        );
        bytes
    }

    #[test]
    fn verifies_canonical_header_and_builds_wound_proxy_faces() {
        let bytes = payload();
        let view = Holographic10dViewportSpec::from_payload(
            FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE,
            &bytes,
            &mut [],
            &mut [],
        )
        .unwrap();
        assert_eq!(view.state(), Holographic10dViewportState::HeaderVerified);
        assert_eq!(view.raymarch_steps(32), Ok(32));
        assert_eq!(
            view.raymarch_steps(33),
            Err(Holographic10dViewportState::RaymarchBudgetExceeded {
                requested: 33,
                max: 32
            })
        );
        let faces = view.proxy_lens();
        assert_eq!(faces.len(), 12);
        for [a, b, c] in faces {
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let normal = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            assert!(
                normal
                    .into_iter()
                    .zip(a)
                    .map(|(n, coordinate)| n * coordinate)
                    .sum::<f32>()
                    > 0.0
            );
        }
    }

    #[test]
    fn missing_typed_field_and_damaged_header_are_rejected() {
        let mut bytes = payload();
        assert!(matches!(
            Holographic10dViewportSpec::from_payload(0, &bytes, &mut [], &mut []),
            Err(Holographic10dViewportState::MissingHoloSection)
        ));
        bytes[13] ^= 1;
        assert!(matches!(
            Holographic10dViewportSpec::from_payload(
                FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE,
                &bytes,
                &mut [],
                &mut []
            ),
            Err(Holographic10dViewportState::InvalidPayload(
                HolographicPhaseError::ChecksumMismatch { .. }
            ))
        ));
    }

    #[test]
    fn stages_m64_segments_into_96_byte_gpu_nodes() {
        let segments = [M64HoloRecord::default(); 2];
        let (nodes, count) = stage_holographic_nodes_gpu(&segments, &[]);
        assert_eq!(count, 2);
        assert_eq!(core::mem::size_of_val(&nodes[0]), 96);
        assert!(nodes[0].position_scale[3].is_finite());
        assert_eq!(nodes[0].as_bytes().len(), 96);
    }
}
