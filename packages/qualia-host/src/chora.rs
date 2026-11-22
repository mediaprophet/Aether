// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Parent-only read bridge. Transport correlation is separate from canonical
//! authority, which is revalidated by NativeGraphBridge on every request.
use crate::NativeGraphBridge;
use aether_chora_bridge_protocol::{
    CallerContext, Control, ControlEnvelope, ExecutionProfile, MAX_DEADLINE_SECS, MAX_PENDING,
    MAX_REPLY_BYTES, Nonce, ReplyError, RouteStamp,
};
use qualia_core_db::{NQuin, vibe_host::native_authority::graph::GraphError};
use serde::Serialize;
use std::{
    io::Write,
    time::{Duration, Instant},
};

struct Route {
    stamp: RouteStamp,
    caller: CallerContext,
    profile: ExecutionProfile,
    bridge: NativeGraphBridge,
    scratch: Vec<NQuin>,
}
struct Pending {
    stamp: RouteStamp,
    id: Nonce,
    deadline: Instant,
}

/// An application must explicitly register the actual engine-derived caller,
/// exact route and a signed canonical bridge. There is no auto-registration.
/// Engine messages must arrive over the owned pipe after PeerGuard validation.
/// Keep this owner off the render callback: graph reads try-lock, but JSON work
/// and scratch allocation are cold host work. No worker receives credentials.
#[derive(Default)]
pub struct ChoraGraphRoutes {
    routes: Vec<Route>,
    pending: Vec<Pending>,
    closed: bool,
}
impl ChoraGraphRoutes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        stamp: RouteStamp,
        caller: CallerContext,
        profile: ExecutionProfile,
        bridge: NativeGraphBridge,
    ) -> Result<(), ReplyError> {
        if self.closed {
            return Err(ReplyError::Cancelled);
        }
        if self.routes.len() >= MAX_PENDING {
            return Err(ReplyError::Busy);
        }
        // One private session belongs to one configured worker profile.
        if self
            .routes
            .iter()
            .any(|r| r.stamp.session == stamp.session && r.profile != profile)
        {
            return Err(ReplyError::InvalidContext);
        }
        // A new document/view binding requires explicit invalidation first.
        if self
            .routes
            .iter()
            .any(|r| r.stamp.session == stamp.session && r.stamp.view == stamp.view)
        {
            return Err(ReplyError::InvalidContext);
        }
        let validation = ControlEnvelope {
            stamp,
            sequence: 1,
            message: Control::PageRequest {
                id: [0; 16],
                caller: caller.clone(),
                channel: "qualia.graph.snapshot".into(),
                payload: "{}".into(),
            },
        };
        validation
            .validate()
            .map_err(|_| ReplyError::InvalidContext)?;
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(bridge.snapshot_capacity())
            .map_err(|_| ReplyError::Unavailable)?;
        scratch.resize(bridge.snapshot_capacity(), NQuin::default());
        self.routes
            .try_reserve(1)
            .map_err(|_| ReplyError::Unavailable)?;
        self.routes.push(Route {
            stamp,
            caller,
            profile,
            bridge,
            scratch,
        });
        Ok(())
    }

    /// Retain at most 64 real requests. Only the fixed no-argument read service
    /// is admitted; scripts, mutation channels and page-selected contexts deny.
    pub fn accept(&mut self, envelope: &ControlEnvelope, now: Instant) -> Result<(), ReplyError> {
        if self.closed {
            return Err(ReplyError::Cancelled);
        }
        envelope
            .validate()
            .map_err(|_| ReplyError::InvalidContext)?;
        if envelope.sequence == 0 {
            return Err(ReplyError::InvalidContext);
        }
        let Control::PageRequest {
            id,
            caller,
            channel,
            payload,
        } = &envelope.message
        else {
            return Err(ReplyError::Denied);
        };
        if channel != "qualia.graph.snapshot" || payload != "{}" {
            return Err(ReplyError::Denied);
        }
        if !self
            .routes
            .iter()
            .any(|r| r.stamp == envelope.stamp && r.caller == *caller)
        {
            return Err(ReplyError::Denied);
        }
        if self
            .pending
            .iter()
            .any(|p| p.stamp == envelope.stamp && p.id == *id)
        {
            return Err(ReplyError::InvalidContext);
        }
        if self.pending.len() >= MAX_PENDING {
            return Err(ReplyError::Busy);
        }
        let deadline = now
            .checked_add(Duration::from_secs(MAX_DEADLINE_SECS))
            .ok_or(ReplyError::Unavailable)?;
        self.pending
            .try_reserve(1)
            .map_err(|_| ReplyError::Unavailable)?;
        self.pending.push(Pending {
            stamp: envelope.stamp,
            id: *id,
            deadline,
        });
        Ok(())
    }

    /// Consume the retained request once, revalidate canonical authority and
    /// serialize a fresh complete snapshot. All six lanes are decimal strings,
    /// preserving u64 values when consumed by traditional web JavaScript.
    pub fn finish(
        &mut self,
        stamp: RouteStamp,
        id: Nonce,
        now: Instant,
    ) -> Result<String, ReplyError> {
        if self.closed {
            return Err(ReplyError::Cancelled);
        }
        let index = self
            .pending
            .iter()
            .position(|p| p.stamp == stamp && p.id == id)
            .ok_or(ReplyError::Cancelled)?;
        let pending = self.pending.swap_remove(index);
        if now >= pending.deadline {
            return Err(ReplyError::TimedOut);
        }
        let route = self
            .routes
            .iter_mut()
            .find(|r| r.stamp == stamp)
            .ok_or(ReplyError::Cancelled)?;
        let mut output = BoundedOutput(Vec::new());
        output
            .0
            .try_reserve_exact(MAX_REPLY_BYTES)
            .map_err(|_| ReplyError::Unavailable)?;
        let read = match route.bridge.read_snapshot(&mut route.scratch) {
            Ok(read) => read,
            Err(error) => {
                route.scratch.fill(NQuin::default());
                return Err(map_graph(error));
            }
        };
        let result = serde_json::to_writer(
            &mut output,
            &Snapshot {
                context: read.context.to_string(),
                revision: read.revision.to_string(),
                quins: Rows(&route.scratch[..read.written]),
            },
        );
        route.scratch.fill(NQuin::default());
        if result.is_err() {
            output.0.fill(0);
            return Err(ReplyError::TooLarge);
        }
        // Real wall-clock expiry also applies if serialization consumed the lease.
        if Instant::now() >= pending.deadline {
            output.0.fill(0);
            return Err(ReplyError::TimedOut);
        }
        String::from_utf8(output.0).map_err(|_| ReplyError::Unavailable)
    }

    pub fn cancel(&mut self, stamp: RouteStamp, id: Nonce) {
        self.pending.retain(|p| p.stamp != stamp || p.id != id);
    }
    /// Cancel and discard a session/view's old document routes before navigation,
    /// resize or actual worker document invalidation. A new exact route must be
    /// registered explicitly; the replacement stamp alone grants nothing.
    pub fn invalidate(&mut self, stamp: RouteStamp) {
        let same_view = |s: RouteStamp| s.session == stamp.session && s.view == stamp.view;
        self.pending.retain(|p| !same_view(p.stamp));
        self.routes.retain_mut(|r| {
            if same_view(r.stamp) {
                r.scratch.fill(NQuin::default());
                false
            } else {
                true
            }
        });
    }
    /// A worker lifecycle acknowledgement may already match a newly registered
    /// owner. Preserve that exact route and remove only previous owners.
    #[cfg(feature = "chora-client")]
    pub(crate) fn advance_view(&mut self, stamp: RouteStamp) {
        let obsolete =
            |s: RouteStamp| s.session == stamp.session && s.view == stamp.view && s != stamp;
        self.pending.retain(|p| !obsolete(p.stamp));
        self.routes.retain_mut(|r| {
            if obsolete(r.stamp) {
                r.scratch.fill(NQuin::default());
                false
            } else {
                true
            }
        });
    }
    /// Returns expired exact requests for the parent to send timeout replies.
    pub fn expire(&mut self, now: Instant) -> Vec<(RouteStamp, Nonce)> {
        let mut expired = Vec::new();
        self.pending.retain(|p| {
            if now >= p.deadline {
                expired.push((p.stamp, p.id));
                false
            } else {
                true
            }
        });
        expired
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.pending.clear();
        for route in &mut self.routes {
            route.scratch.fill(NQuin::default());
        }
        self.routes.clear();
    }
}
impl Drop for ChoraGraphRoutes {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(feature = "chora-client")]
impl ChoraGraphRoutes {
    /// Pump one dedicated owned worker on a host scheduling thread. Registration
    /// stays an explicit application action. At most 64 events are processed per
    /// call. Busy authority reads get explicit Busy replies; a full transport
    /// queue closes the worker and clears all parent routes so no request is lost
    /// silently. Shutdown/reaping remains the client's background responsibility.
    pub fn pump_client(
        &mut self,
        client: &aether_chora_client::ChoraClient,
        now: Instant,
    ) -> Result<usize, aether_chora_bridge_protocol::ProtocolError> {
        use aether_chora_bridge_protocol::ProtocolError;
        let current = match client.stamp() {
            Ok(stamp) => stamp,
            Err(ProtocolError::Busy) => return Err(ProtocolError::Busy),
            Err(error) => {
                self.close();
                return Err(error);
            }
        };
        // This API owns routes for one process. The current client supports only
        // the explicit trusted public profile, which must match registration.
        if self.routes.iter().any(|r| {
            r.stamp.session != current.session
                || r.stamp.view != current.view
                || r.profile != ExecutionProfile::TrustedEmbedding
        }) {
            self.close();
            client.shutdown();
            return Err(ProtocolError::WrongSession);
        }
        self.advance_view(current);
        let mut processed = 0;
        while processed < MAX_PENDING {
            let Some(envelope) = client.take_event() else {
                break;
            };
            processed += 1;
            match &envelope.message {
                Control::PageRequest { id, .. } => {
                    let result = self
                        .accept(&envelope, now)
                        .and_then(|()| self.finish(envelope.stamp, *id, now));
                    if let Err(error) = client.reply(envelope.stamp, *id, result) {
                        if matches!(error, ProtocolError::StaleFrame) {
                            self.cancel(envelope.stamp, *id);
                            continue;
                        }
                        self.close();
                        client.shutdown();
                        return Err(error);
                    }
                }
                Control::Cancel { id, .. } => self.cancel(envelope.stamp, *id),
                Control::Invalidate { .. } => self.advance_view(envelope.stamp),
                Control::Ready { .. } => {}
                Control::Stopped | Control::Failed { .. } => {
                    self.close();
                    return Err(ProtocolError::Closed);
                }
                _ => {
                    self.close();
                    client.shutdown();
                    return Err(ProtocolError::WrongDirection);
                }
            }
        }
        for (stamp, id) in self.expire(now) {
            if let Err(error) = client.reply(stamp, id, Err(ReplyError::TimedOut)) {
                if matches!(error, ProtocolError::StaleFrame) {
                    continue;
                }
                self.close();
                client.shutdown();
                return Err(error);
            }
        }
        Ok(processed)
    }
}

fn map_graph(error: GraphError) -> ReplyError {
    match error {
        GraphError::Busy => ReplyError::Busy,
        GraphError::Capacity => ReplyError::TooLarge,
        GraphError::Cancelled => ReplyError::Cancelled,
        GraphError::Journal(_) | GraphError::Clock | GraphError::Poisoned => {
            ReplyError::Unavailable
        }
        _ => ReplyError::Denied,
    }
}
struct BoundedOutput(Vec<u8>);
impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_REPLY_BYTES.saturating_sub(self.0.len()) {
            return Err(std::io::ErrorKind::WriteZero.into());
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[derive(Serialize)]
struct Snapshot<'a> {
    context: String,
    revision: String,
    quins: Rows<'a>,
}
struct Rows<'a>(&'a [NQuin]);
impl Serialize for Rows<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for q in self.0 {
            seq.serialize_element(&[
                q.subject.to_string(),
                q.predicate.to_string(),
                q.object.to_string(),
                q.context.to_string(),
                q.metadata.to_string(),
                q.parity.to_string(),
            ])?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_output_stops_at_byte_limit() {
        let row = NQuin {
            subject: u64::MAX,
            predicate: u64::MAX,
            object: u64::MAX,
            context: u64::MAX,
            metadata: u64::MAX,
            parity: u64::MAX,
        };
        let rows = vec![row; 4096];
        let mut output = BoundedOutput(Vec::new());
        output.0.try_reserve_exact(MAX_REPLY_BYTES).unwrap();
        let capacity = output.0.capacity();
        assert!(
            serde_json::to_writer(
                &mut output,
                &Snapshot {
                    context: u64::MAX.to_string(),
                    revision: u64::MAX.to_string(),
                    quins: Rows(&rows)
                }
            )
            .is_err()
        );
        assert!(output.0.len() <= MAX_REPLY_BYTES);
        assert_eq!(output.0.capacity(), capacity);
    }
}
