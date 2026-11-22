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
}
