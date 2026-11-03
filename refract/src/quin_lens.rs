//! Zero-heap lenses over 48-byte `QuinSlot` (`NQuin`) buffers and bitemporal
//! `(valid_at_ms, tx_at_ms)` windows.
//!
//! Designed for QualiaDB Rule 0 (100% `#[repr(C)]`, zero heap allocations in
//! hot loops) so Aether and Refract UI components can project directly onto
//! fixed-size `[QuinSlot; N]` buffers with element-level invalidation.

use crate::lens::{Field, Lens, Path};

/// Magic parity salt matching QualiaDB 48-byte `NQuin` parity integrity checks.
pub const QUIN_PARITY_SALT: u64 = 0x5155_414C_4941_4442;

/// A 48-byte `#[repr(C)]` Super-Quin slot matching the QualiaDB `NQuin` memory layout.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct QuinSlot {
    /// Subject node identifier (`u64`).
    pub subject: u64,
    /// Predicate relation identifier (`u64`).
    pub predicate: u64,
    /// Object node or literal payload (`u64`).
    pub object: u64,
    /// Context / named graph / capability envelope identifier (`u64`).
    pub context: u64,
    /// Packed temporal/modal/deontic metadata word (`u64`).
    pub metadata: u64,
    /// XOR integrity parity word (`u64`).
    pub parity: u64,
}

const _: () = assert!(
    std::mem::size_of::<QuinSlot>() == 48,
    "QuinSlot must be exactly 48 bytes"
);

impl QuinSlot {
    /// Compute the deterministic 64-bit XOR parity word over the 5 payload fields.
    #[inline]
    pub const fn calculate_parity(
        subject: u64,
        predicate: u64,
        object: u64,
        context: u64,
        metadata: u64,
    ) -> u64 {
        subject ^ predicate ^ object ^ context ^ metadata ^ QUIN_PARITY_SALT
    }

    /// Construct a new 48-byte [`QuinSlot`] with automatically computed parity.
    #[inline]
    pub const fn new(
        subject: u64,
        predicate: u64,
        object: u64,
        context: u64,
        metadata: u64,
    ) -> Self {
        Self {
            subject,
            predicate,
            object,
            context,
            metadata,
            parity: Self::calculate_parity(subject, predicate, object, context, metadata),
        }
    }

    /// Verify that the stored `parity` word matches the 5 payload fields.
    #[inline]
    pub const fn verify_parity(&self) -> bool {
        self.parity
            == Self::calculate_parity(
                self.subject,
                self.predicate,
                self.object,
                self.context,
                self.metadata,
            )
    }

    /// Recompute and update the `parity` field after in-place mutation of any slot field.
    #[inline]
    pub fn recompute_parity(&mut self) {
        self.parity = Self::calculate_parity(
            self.subject,
            self.predicate,
            self.object,
            self.context,
            self.metadata,
        );
    }
}

/// Bitemporal coordinate pair (`valid_at_ms`, `tx_at_ms`) for `InstantBridge` slices.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct BitemporalStamp {
    /// Valid-time coordinate in milliseconds since epoch (`i64`).
    pub valid_at_ms: i64,
    /// Transaction-time coordinate in milliseconds since epoch (`i64`).
    pub tx_at_ms: i64,
}

impl BitemporalStamp {
    /// Create a bitemporal stamp for `(valid_at_ms, tx_at_ms)`.
    #[inline]
    pub const fn new(valid_at_ms: i64, tx_at_ms: i64) -> Self {
        Self {
            valid_at_ms,
            tx_at_ms,
        }
    }

    /// Return `true` if this stamp is visible under the given bitemporal query horizon.
    #[inline]
    pub const fn is_visible_at(&self, query_valid_ms: i64, query_tx_ms: i64) -> bool {
        self.valid_at_ms <= query_valid_ms && self.tx_at_ms <= query_tx_ms
    }
}

/// A fixed-capacity, stack-allocated window of `N` 48-byte [`QuinSlot`]s and
/// their corresponding [`BitemporalStamp`]s, plus an active bitemporal cursor.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct QuinWindow<const N: usize> {
    /// Fixed-size buffer of 48-byte `QuinSlot`s.
    pub slots: [QuinSlot; N],
    /// Parallel bitemporal stamps for each slot.
    pub stamps: [BitemporalStamp; N],
    /// Number of populated slots (`<= N`).
    pub active_len: usize,
    /// Current bitemporal query horizon (`valid_at_ms`).
    pub cursor_valid_at_ms: i64,
    /// Current bitemporal query horizon (`tx_at_ms`).
    pub cursor_tx_at_ms: i64,
}

impl<const N: usize> Default for QuinWindow<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> QuinWindow<N> {
    /// Create an empty `QuinWindow<N>` with unbounded (`i64::MAX`) bitemporal horizon.
    #[inline]
    pub const fn new() -> Self {
        Self::with_bitemporal_cursor(i64::MAX, i64::MAX)
    }

    /// Create an empty `QuinWindow<N>` initialized to a specific `(valid_at_ms, tx_at_ms)` cursor.
    #[inline]
    pub const fn with_bitemporal_cursor(valid_at_ms: i64, tx_at_ms: i64) -> Self {
        Self {
            slots: [QuinSlot {
                subject: 0,
                predicate: 0,
                object: 0,
                context: 0,
                metadata: 0,
                parity: QUIN_PARITY_SALT,
            }; N],
            stamps: [BitemporalStamp {
                valid_at_ms: 0,
                tx_at_ms: 0,
            }; N],
            active_len: 0,
            cursor_valid_at_ms: valid_at_ms,
            cursor_tx_at_ms: tx_at_ms,
        }
    }

    /// Insert a [`QuinSlot`] and [`BitemporalStamp`] into the next available slot.
    ///
    /// Returns `Some(index)` if inserted, or `None` if the fixed window is full.
    #[inline]
    pub fn insert(&mut self, slot: QuinSlot, stamp: BitemporalStamp) -> Option<usize> {
        if self.active_len >= N {
            return None;
        }
        let idx = self.active_len;
        self.slots[idx] = slot;
        self.stamps[idx] = stamp;
        self.active_len += 1;
        Some(idx)
    }

    /// Return the number of active slots in the window.
    #[inline]
    pub const fn active_len(&self) -> usize {
        self.active_len
    }

    /// Borrow the populated slice of [`QuinSlot`]s.
    #[inline]
    pub fn active_slots(&self) -> &[QuinSlot] {
        &self.slots[..self.active_len]
    }

    /// Return `true` if slot `index` is populated and visible under the window's bitemporal cursor.
    #[inline]
    pub fn is_slot_visible(&self, index: usize) -> bool {
        index < self.active_len
            && self.stamps[index].is_visible_at(self.cursor_valid_at_ms, self.cursor_tx_at_ms)
    }

    /// Count how many active slots are visible under the current bitemporal cursor.
    #[inline]
    pub fn visible_count(&self) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < self.active_len {
            if self.stamps[i].is_visible_at(self.cursor_valid_at_ms, self.cursor_tx_at_ms) {
                count += 1;
            }
            i += 1;
        }
        count
    }

    /// Verify parity across all active slots in the window.
    #[inline]
    pub fn verify_all_parity(&self) -> bool {
        let mut i = 0;
        while i < self.active_len {
            if !self.slots[i].verify_parity() {
                return false;
            }
            i += 1;
        }
        true
    }
}

/// Element-level lens into `QuinWindow<N>::slots[index]`.
pub struct QuinSlotLens<P, const N: usize> {
    parent: P,
    index: usize,
}

impl<P: Copy, const N: usize> Clone for QuinSlotLens<P, N> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<P: Copy, const N: usize> Copy for QuinSlotLens<P, N> {}

impl<S, P, const N: usize> Lens<S> for QuinSlotLens<P, N>
where
    S: 'static,
    P: Lens<S, Target = QuinWindow<N>>,
{
    type Target = QuinSlot;

    fn push_path(&self, out: &mut Path) {
        self.parent.push_path(out);
        out.push(0);
        out.push(self.index as u32);
    }

    fn get<'a>(&self, state: &'a S) -> &'a QuinSlot {
        &self.parent.get(state).slots[self.index]
    }

    fn get_mut<'a>(&self, state: &'a mut S) -> &'a mut QuinSlot {
        &mut self.parent.get_mut(state).slots[self.index]
    }
}

/// Element-level lens into `QuinWindow<N>::stamps[index]`.
pub struct QuinStampLens<P, const N: usize> {
    parent: P,
    index: usize,
}

impl<P: Copy, const N: usize> Clone for QuinStampLens<P, N> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<P: Copy, const N: usize> Copy for QuinStampLens<P, N> {}

impl<S, P, const N: usize> Lens<S> for QuinStampLens<P, N>
where
    S: 'static,
    P: Lens<S, Target = QuinWindow<N>>,
{
    type Target = BitemporalStamp;

    fn push_path(&self, out: &mut Path) {
        self.parent.push_path(out);
        out.push(1);
        out.push(self.index as u32);
    }

    fn get<'a>(&self, state: &'a S) -> &'a BitemporalStamp {
        &self.parent.get(state).stamps[self.index]
    }

    fn get_mut<'a>(&self, state: &'a mut S) -> &'a mut BitemporalStamp {
        &mut self.parent.get_mut(state).stamps[self.index]
    }
}

/// Extension trait for lenses targeting [`QuinWindow<N>`].
pub trait QuinWindowLens<S: 'static, const N: usize>: Lens<S, Target = QuinWindow<N>> {
    /// Project to the `index`-th [`QuinSlot`] without invalidating sibling slots.
    fn slot(self, index: usize) -> QuinSlotLens<Self, N> {
        assert!(index < N, "QuinSlot index out of bounds for QuinWindow");
        QuinSlotLens {
            parent: self,
            index,
        }
    }

    /// Project to the `index`-th [`BitemporalStamp`] without invalidating sibling stamps.
    fn stamp(self, index: usize) -> QuinStampLens<Self, N> {
        assert!(
            index < N,
            "BitemporalStamp index out of bounds for QuinWindow"
        );
        QuinStampLens {
            parent: self,
            index,
        }
    }

    /// Project to the window's `cursor_valid_at_ms` horizon field.
    fn cursor_valid_at(self) -> Field<Self, QuinWindow<N>, i64> {
        self.field(
            2,
            |w| &w.cursor_valid_at_ms,
            |w| &mut w.cursor_valid_at_ms,
        )
    }

    /// Project to the window's `cursor_tx_at_ms` horizon field.
    fn cursor_tx_at(self) -> Field<Self, QuinWindow<N>, i64> {
        self.field(3, |w| &w.cursor_tx_at_ms, |w| &mut w.cursor_tx_at_ms)
    }
}

impl<S: 'static, const N: usize, L: Lens<S, Target = QuinWindow<N>>> QuinWindowLens<S, N> for L {}

/// Extension trait for lenses targeting a [`QuinSlot`] to project directly onto its 6 `u64` words.
pub trait QuinFieldLens<S: 'static>: Lens<S, Target = QuinSlot> {
    /// Project to the `subject` word (`segment = 0`).
    fn subject(self) -> Field<Self, QuinSlot, u64> {
        self.field(0, |q| &q.subject, |q| &mut q.subject)
    }

    /// Project to the `predicate` word (`segment = 1`).
    fn predicate(self) -> Field<Self, QuinSlot, u64> {
        self.field(1, |q| &q.predicate, |q| &mut q.predicate)
    }

    /// Project to the `object` word (`segment = 2`).
    fn object(self) -> Field<Self, QuinSlot, u64> {
        self.field(2, |q| &q.object, |q| &mut q.object)
    }

    /// Project to the `context` word (`segment = 3`).
    fn context(self) -> Field<Self, QuinSlot, u64> {
        self.field(3, |q| &q.context, |q| &mut q.context)
    }

    /// Project to the `metadata` word (`segment = 4`).
    fn metadata(self) -> Field<Self, QuinSlot, u64> {
        self.field(4, |q| &q.metadata, |q| &mut q.metadata)
    }

    /// Project to the `parity` word (`segment = 5`).
    fn parity(self) -> Field<Self, QuinSlot, u64> {
        self.field(5, |q| &q.parity, |q| &mut q.parity)
    }
}

impl<S: 'static, L: Lens<S, Target = QuinSlot>> QuinFieldLens<S> for L {}
