// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Headless native owner for an already-adopted project. No fixture credentials.
use aether_qualia_host::{
    ExecutionLimits, native_graph_runtime,
    qualia_cooperative_core::authoring::DocumentRevision,
    qualia_core_db::{
        NQuin,
        vibe_host::{
            native_authority::{
                NativeAuthority, SessionHandle, SignedAdmission, TrustedProject,
                graph::SignedGraphGrant,
            },
            vibe_sentinel::WebizenSentinelGate,
        },
    },
    ui_channel,
    vibe::extension_registry::CallExecutionContext,
    vibescript_abi::{EffectSet, ExecutionMode, ResourceEnvelope, q_hash_str},
};
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

const HELP: &str = "Native authority-owned project graph startup\n\n\
Usage: PROGRAM JOURNAL_DIR PROJECT_ID ISSUER PUBLIC_KEY_HEX ADMISSION_JSON DOCUMENT_JSON SUBJECT GRAPH_GRANT_JSON SCRIPT_VIBE\n\n\
Use an existing host-owned journal directory and independently adopted issuer key.\n\
The signed admission, document revision and signed graph grant must belong to that\n\
project. The example does not enroll subjects, create grants or adopt trust anchors\n\
automatically. It prints a one-use session challenge; use the admitted controller's\n\
existing native signing tool to sign the digest under qualia.native-authoring-session.v1,\n\
then enter the signature as 128 hexadecimal characters. Never enter a private key.\n\n\
Only the authority-owned project graph is accessible. Extensions have no implicit\n\
Sentinel permits. Use native_graph for headless execution or native_graph_window\n\
from the reviewed local integration profile for the native application.\n";
fn bounded_file(path: &str, maximum: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let file = File::open(path)?;
    if file.metadata()?.len() > maximum as u64 {
        return Err("input exceeds byte limit".into());
    }
    let mut data = Vec::new();
    file.take(maximum as u64 + 1).read_to_end(&mut data)?;
    if data.len() > maximum {
        return Err("input grew beyond byte limit".into());
    }
    Ok(data)
}
fn hex<const N: usize>(text: &str) -> Result<[u8; N], Box<dyn Error>> {
    let bytes = text.trim().as_bytes();
    if bytes.len() != N * 2 {
        return Err("incorrect hexadecimal length".into());
    }
    let nibble = |b: u8| -> Result<u8, Box<dyn Error>> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'A'..=b'F' => Ok(b - b'A' + 10),
            _ => Err("invalid hexadecimal character".into()),
        }
    };
    let mut output = [0; N];
    for (out, pair) in output.iter_mut().zip(bytes.chunks_exact(2)) {
        *out = nibble(pair[0])? * 16 + nibble(pair[1])?;
    }
    Ok(output)
}
/// Shared explicit startup for the headless and actual native window examples.
pub struct AuthenticatedInputs {
    pub authority: Arc<NativeAuthority>,
    pub session: SessionHandle,
    pub grant: SignedGraphGrant,
    pub source: String,
    pub subject: String,
}
struct Disconnect<'a> {
    authority: &'a NativeAuthority,
    session: &'a SessionHandle,
}
impl Drop for Disconnect<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.authority.disconnect(self.session) {
            eprintln!("Session disconnect requires reconciliation: {error}");
        }
    }
}
pub fn authenticate_inputs() -> Result<Option<AuthenticatedInputs>, Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() || args.as_slice() == ["--help"] {
        print!("{HELP}");
        return Ok(None);
    }
    if args.len() != 9 {
        return Err(HELP.into());
    }
    let admission: SignedAdmission = serde_json::from_slice(&bounded_file(&args[4], 64 * 1024)?)?;
    let document: DocumentRevision = serde_json::from_slice(&bounded_file(&args[5], 64 * 1024)?)?;
    let grant: SignedGraphGrant = serde_json::from_slice(&bounded_file(&args[7], 64 * 1024)?)?;
    let source = String::from_utf8(bounded_file(&args[8], 64 * 1024)?)?;
    let authority = Arc::new(NativeAuthority::open(
        Path::new(&args[0]),
        TrustedProject {
            project_id: args[1].clone(),
            issuer_identifier: args[2].clone(),
            public_key: hex(&args[3])?,
        },
        admission,
        document,
        1024 * 1024,
        256,
    )?);
    let challenge = authority.begin_session(&args[6], 300)?;
    print!("Session challenge digest: ");
    for byte in challenge.message_digest() {
        print!("{byte:02x}");
    }
    println!("\nSign using the admitted controller and qualia.native-authoring-session.v1.");
    print!("Signature (128 hex characters): ");
    io::stdout().flush()?;
    // A bounded line, not a secret: the private key remains with its native owner.
    let mut line = Vec::new();
    let mut input = io::stdin().lock();
    loop {
        let mut byte = [0; 1];
        if input.read(&mut byte)? == 0 || byte[0] == b'\n' {
            break;
        }
        if line.len() >= 130 {
            return Err("signature line exceeds bound".into());
        }
        line.push(byte[0]);
    }
    let signature = hex::<64>(std::str::from_utf8(&line)?)?;
    let session = authority.complete_session(challenge, &signature)?;
    Ok(Some(AuthenticatedInputs {
        authority,
        session,
        grant,
        source,
        subject: args[6].clone(),
    }))
}
fn main() -> Result<(), Box<dyn Error>> {
    let Some(AuthenticatedInputs {
        authority,
        session,
        grant,
        source,
        subject,
    }) = authenticate_inputs()?
    else {
        return Ok(());
    };
    // Covers construction, preparation, execution and delivery failures alike.
    let _disconnect = Disconnect {
        authority: &authority,
        session: &session,
    };
    let mut extension_audit = [NQuin::default(); 2];
    let gate = WebizenSentinelGate::new(
        &[],
        &mut extension_audit,
        1,
        EffectSet::PURE_COMPUTE,
        ResourceEnvelope::hot_zero_heap(100_000, 4096),
    );
    let cancelled = Arc::new(AtomicBool::new(false));
    let (sender, mut receiver) = ui_channel(1, || {})?;
    let mut runtime = native_graph_runtime(
        authority.clone(),
        &session,
        &grant,
        gate,
        ExecutionLimits::default(),
        sender,
        cancelled.clone(),
    )?;
    let program = runtime.prepare(&source)?;
    let outcome = runtime.execute(
        program,
        CallExecutionContext {
            actor_did_hash: q_hash_str(&subject),
            context_world_hash: grant.grant.context,
            lamport_timestamp: 0,
            mode: ExecutionMode::GovernedApplication,
        },
    );
    if outcome.is_ok() {
        receiver.drain(1, |delivery| {
            println!("Canonical result: {:?}", delivery.value)
        });
    }
    let (host, _) = runtime.into_services();
    if let Some(receipt) = host.last_receipt() {
        println!(
            "Last acknowledged native graph commit: context {}, revision {}",
            receipt.context, receipt.revision
        );
    }
    cancelled.store(true, Ordering::Release);
    if let Err(error) = outcome {
        eprintln!(
            "Execution failed; earlier acknowledged graph effects may remain in the native journal."
        );
        return Err(error.into());
    }
    Ok(())
}
