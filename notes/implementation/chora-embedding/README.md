# Chora generic embedding transport — C2

This patch adds a bounded `navigator.chora.request(channel, payload)` producer,
constellation-owned request/reply/cancellation routing, and the public
`WebViewDelegate::notify_embedder_message` request type. It changes the isolated
Chora snapshot at the original checkpoint. The resumed session verified and applied
the complete series to the writable sibling checkout; see
[current local evidence](../resume-chora.md). The resumed root WebGPU graph
compiles the shared embedding interfaces, while full engine and runtime checks
remain distinct.

## Caller and lifecycle contract

- The initial profile admits fully active, unsandboxed top-level HTTP(S)
  documents. Frames, workers, inherited documents, opaque origins, `blob:`,
  `data:` and internal protocols receive no host bridge authority.
- The DOM producer supplies its actual immutable security origin; the
  constellation verifies the active pipeline and compares that origin with the
  engine's final pipeline URL. Page bodies cannot select caller identity.
  Opaque origins retain their distinct engine identities and are rejected.
- The engine assigns an instance identifier, WebView/frame pipeline identity and
  navigation epoch. The host maps the engine instance and private-browsing state
  to its own profile and authorizes each channel/operation. These fields grant
  no host capability on their own.
- Current pipeline or exact URL changes rotate the epoch, including same-document
  push/replace/fragment navigation and history restoration. Navigation start,
  pipeline close and shutdown invalidate pending requests.
- Channel names are bounded to 128 ASCII bytes, payloads to 16 KiB and replies to
  64 KiB. There are at most 64 pending requests per engine and 64 per producer.
  The 30-second deadline uses the constellation's timed channel selection, so an
  idle page does not need to send another request for expiration.
- Replies are one-shot. Missing handlers and dropped host request handles deny;
  explicit `cancel()` rejects. Expired, consumed and stale IDs cannot deliver a
  reply. Host cancellation notifications run on the thread spinning Chora.
- Weak cancellation tokens do not retain WebViews. A retained host request can
  observe `is_cancelled()` after its view closes and remains cancelled after
  engine destruction. While the engine lives, cancellation notifications and
  tokens are processed through its event loop. The constellation independently
  rejects stale replies. Hosts stop work they have scheduled when cancelled.
- Script callback teardown becomes an `Unavailable` rejection instead of an
  unwrap panic. This API does not add an AbortSignal/page cancellation overload.
  No keystore provider or ecosystem operation is connected by this patch.

## Patch and provenance

`chora-c2-embedding.patch` contains 15 files and preserves MPL notices.
`manifest.json` records available Git revision metadata, exact original file hashes,
C2-only candidate hashes, and the combined snapshot hashes. LF-normalized hashes
are provided alongside raw hashes to make Windows newline handling explicit.
Original source baselines are stored outside this repository.

Apply to a writable checkout matching the recorded baseline, in this order:
C1 Etalon, C2 embedding, C3 TLS, then the C API ownership patch. C2 excludes C1's
GPU type change in the shared script enum and C3's TLS exports/startup edits.
A scratch checkout made from the original bytes passed `git apply --check
--whitespace=error-all`, application, and comparison with every C2 candidate.
See `patch-apply-check.log`.

## Original checkpoint verification

| Check | Result |
| --- | --- |
| All 12 modified Rust files parse and pass rustfmt check | Passed |
| Actual standalone lifecycle ledger unit tests | 5 passed |
| Actual enabled checked-in WebIDL parse, ChoraMessaging generation and concrete type mapping | Passed |
| Generated Request CurrentRealm/Promise and Navigator getter JSContext signatures | Passed |
| Actual modified shared/embedder crate check | Blocked before compilation by missing offline Stylo revision |
| External consumer selecting patched Chora by explicit path and repeated patches | Blocked before compilation by the same missing Stylo revision |
| Actual protocol origin/opaque/size unit tests | Added; not executed |
| Browser default-denial, body-spoof, size and subframe fixtures | Added; not executed |

The originally missing revision was `a05fadffd43bc173b2d6e1e2caf44a36b45354af`
from `https://github.com/servo/stylo`. It has now been retrieved exactly. The
original logs preserve the earlier dependency failures, not a current missing-source
claim; the external consumer and browser protocol tests still need execution.
No substitute caller-identity types were used to claim protocol compilation.

`check_webidl.py --checkout PATH` runs the actual parser/configuration and selected
binding generation. `check_consumer.py --checkout PATH --cargo-home CACHE` checks
`consumer.rs` from an independent temporary workspace. Dependency workspace
patches are repeated in that consumer instead of assuming they are inherited.

Full engine Rust compilation, executable allowed/denied/deferred host responses,
idle timeout, navigation/BFCache and shutdown browser tests, cross-instance reply
probes, malicious IPC/process isolation, and mixed-service host authorization
remain release gates. The source wiring and isolated ledger tests do not prove
runtime browser readiness or completion of the keystore findings.
