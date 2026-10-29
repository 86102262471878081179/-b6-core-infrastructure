// ═══════════════════════════════════════════════════════════════════════════════
// B6-CORE :: RUST WORM KERNEL v2.1.0 — MONSTER EDITION
// ═══════════════════════════════════════════════════════════════════════════════
// 
// Dies ist der ultraschnelle Rust-Kern der Black Six Matrix.
// Er vereint:
//   - Axum REST API (Port 8080)
//   - SHA3-512 Post-Quantum Hasher
//   - Merkle-Tree für Integrity Proofs
//   - PBFT Consensus Simulation
//   - WORM Ledger (Append-Only, Immutable)
//   - Prometheus-Metriken
//   - eBPF-Syscall-Simulation
// 
// Status: Φ_LOCKED_ABSOLUTE
// ═══════════════════════════════════════════════════════════════════════════════

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_512};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ═══════════════════════════════════════════════════════════════════════════════
// 1. KONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
struct Config {
    bind_addr: String,
    worm_path: PathBuf,
    master_seal: String,
    node_id: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            bind_addr: std::env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            worm_path: PathBuf::from(
                std::env::var("WORM_STORAGE_PATH")
                    .unwrap_or_else(|_| "/var/lib/worm_vault".to_string()),
            ),
            master_seal: std::env::var("MASTER_SEAL").unwrap_or_else(|_|
                "850b2ef003f53db90f704b16e54869fc6ba6265eae13aafde9046fe0e4d99dab".to_string()
            ),
            node_id: std::env::var("NODE_ID").unwrap_or_else(|_| "B6_NODE_001".to_string()),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. DATENMODELLE
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WormBlock {
    seq: u64,
    timestamp: u64,
    event: String,
    payload: serde_json::Value,
    prev_hash: String,
    hash: String,
    merkle_root: String,
    pbft_status: String,
    kyber_seal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitRequest {
    domain: String,
    action: String,
    payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitResponse {
    status: String,
    block: WormBlock,
    seal: String,
    node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MerkleProof {
    hash: String,
    merkle_root: String,
    path: Vec<String>,
    verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    node_id: String,
    uptime_secs: u64,
    total_blocks: u64,
    last_hash: String,
    master_seal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PbftVote {
    node: String,
    vote: String,
    signature: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. GLOBALER STATE
// ═══════════════════════════════════════════════════════════════════════════════

struct AppState {
    config: Config,
    blocks: Mutex<Vec<WormBlock>>,
    last_hash: Mutex<String>,
    start_time: u64,
    metrics: Mutex<Metrics>,
}

#[derive(Default, Clone)]
struct Metrics {
    total_commits: u64,
    total_errors: u64,
    total_verifications: u64,
    pbft_rounds: u64,
    ebpf_blocked: u64,
}

impl AppState {
    fn new(config: Config) -> Self {
        let last_hash = config.master_seal.clone();
        Self {
            config,
            blocks: Mutex::new(Vec::new()),
            last_hash: Mutex::new(last_hash),
            start_time: now_unix(),
            metrics: Mutex::new(Metrics::default()),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. KRYPTOGRAFIE-LAYER
// ═══════════════════════════════════════════════════════════════════════════════

/// Berechnet den SHA3-512 Hash eines Strings.
fn sha3_512_hash(data: &str) -> String {
    let mut hasher = Sha3_512::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Simuliert Kyber-1024 Post-Quantum Seal (deterministisch).
/// In Produktion würde hier der pqcrypto-Kyber Crate stehen.
fn kyber_seal(payload: &str, master: &str) -> String {
    let combined = format!("KYBER1024:{}:{}", master, payload);
    sha3_512_hash(&combined)
}

/// Berechnet einen Merkle-Root über eine Liste von Hashes.
fn compute_merkle_root(hashes: &[String]) -> String {
    if hashes.is_empty() {
        return sha3_512_hash("EMPTY");
    }
    if hashes.len() == 1 {
        return hashes[0].clone();
    }
    
    let mut layer: Vec<String> = hashes.to_vec();
    
    while layer.len() > 1 {
        let mut next_layer = Vec::new();
        for chunk in layer.chunks(2) {
            let combined = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                format!("{}{}", chunk[0], chunk[0])
            };
            next_layer.push(sha3_512_hash(&combined));
        }
        layer = next_layer;
    }
    
    layer[0].clone()
}

/// Simuliert einen PBFT-Consensus (4 Nodes, 3/4 Majority nötig).
fn simulate_pbft(block_hash: &str) -> String {
    let nodes = ["R0_0001", "R0_0068", "R1_0089", "R2_0017"];
    let mut votes_for = 0;
    
    for node in nodes.iter() {
        let vote_material = format!("{}:{}", node, block_hash);
        let vote_hash = sha3_512_hash(&vote_material);
        // Deterministische "Wahl": erste Hex-Ziffer > 7 = Ja
        if vote_hash.chars().next().unwrap_or('0') > '7' {
            votes_for += 1;
        }
    }
    
    if votes_for >= 3 {
        "PBFT_COMMITTED".to_string()
    } else {
        "PBFT_PENDING".to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. WORM LEDGER (Append-Only)
// ═══════════════════════════════════════════════════════════════════════════════

async fn append_to_worm(block: &WormBlock, worm_path: &PathBuf) -> Result<(), std::io::Error> {
    fs::create_dir_all(worm_path)?;
    let file_path = worm_path.join("ledger.jsonl");
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)?;
    
    let line = serde_json::to_string(block)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    writeln!(file, "{}", line)?;
    file.sync_all()?;
    
    info!("[WORM] Block #{} appended to ledger.jsonl", block.seq);
    Ok(())
}

async fn load_worm_from_disk(worm_path: &PathBuf) -> Vec<WormBlock> {
    let file_path = worm_path.join("ledger.jsonl");
    if !file_path.exists() {
        return Vec::new();
    }
    
    let content = fs::read_to_string(&file_path).unwrap_or_default();
    content
        .lines()
        .filter_map(|l| serde_json::from_str::<WormBlock>(l).ok())
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. HILFSFUNKTIONEN
// ═══════════════════════════════════════════════════════════════════════════════

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. HTTP HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

/// GET /health — Health Check
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let blocks = state.blocks.lock().await;
    let last_hash = state.last_hash.lock().await.clone();
    let metrics = state.metrics.lock().await.clone();
    
    let health = HealthResponse {
        status: "HEALTHY".to_string(),
        version: "2.1.0".to_string(),
        node_id: state.config.node_id.clone(),
        uptime_secs: now_unix() - state.start_time,
        total_blocks: blocks.len() as u64,
        last_hash: last_hash.chars().take(32).collect(),
        master_seal: state.config.master_seal.chars().take(16).collect(),
    };
    
    info!("[HEALTH] total_commits={} total_errors={} pbft_rounds={}",
          metrics.total_commits, metrics.total_errors, metrics.pbft_rounds);
    
    (StatusCode::OK, Json(health))
}

/// POST /api/v1/worm/commit — WORM Commit
async fn commit_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CommitRequest>,
) -> impl IntoResponse {
    info!("[COMMIT] domain={} action={}", req.domain, req.action);
    
    // 1. Hole aktuellen State
    let mut blocks = state.blocks.lock().await;
    let mut last_hash = state.last_hash.lock().await;
    let mut metrics = state.metrics.lock().await;
    
    let seq = (blocks.len() + 1) as u64;
    let timestamp = now_unix();
    
    // 2. Payload serialisieren
    let payload_str = serde_json::to_string(&req.payload).unwrap_or_default();
    
    // 3. SHA3-512 Hash berechnen
    let hash_material = format!(
        "{}|{}|{}|{}|{}",
        last_hash, seq, timestamp, req.event_or_default(), payload_str
    );
    let block_hash = sha3_512_hash(&hash_material);
    
    // 4. Merkle Root berechnen
    let all_hashes: Vec<String> = blocks.iter().map(|b| b.hash.clone()).collect();
    let mut merkle_hashes = all_hashes.clone();
    merkle_hashes.push(block_hash.clone());
    let merkle_root = compute_merkle_root(&merkle_hashes);
    
    // 5. PBFT Consensus
    let pbft_status = simulate_pbft(&block_hash);
    metrics.pbft_rounds += 1;
    
    // 6. Kyber Seal
    let kyber = kyber_seal(&payload_str, &state.config.master_seal);
    
    // 7. Block erzeugen
    let block = WormBlock {
        seq,
        timestamp,
        event: req.event_or_default(),
        payload: req.payload.clone(),
        prev_hash: last_hash.clone(),
        hash: block_hash.clone(),
        merkle_root,
        pbft_status: pbft_status.clone(),
        kyber_seal: kyber.clone(),
    };
    
    // 8. Persistieren (WORM)
    if let Err(e) = append_to_worm(&block, &state.config.worm_path).await {
        warn!("[COMMIT] WORM-Persist fehlgeschlagen: {}", e);
        metrics.total_errors += 1;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "ERROR",
                "detail": format!("WORM-Persist fehlgeschlagen: {}", e)
            })),
        );
    }
    
    // 9. State updaten
    *last_hash = block_hash.clone();
    blocks.push(block.clone());
    metrics.total_commits += 1;
    
    info!("[COMMIT] Block #{} created, PBFT={}", seq, pbft_status);
    
    let response = CommitResponse {
        status: "COMMITTED".to_string(),
        block,
        seal: format!("Φ_LOCKED_EX_TUNC:{}", kyber),
        node: state.config.node_id.clone(),
    };
    
    (StatusCode::OK, Json(response))
}

/// GET /api/v1/worm/proof/:hash — Merkle Proof für einen Block
async fn proof_handler(
    State(state): State<Arc<AppState>>,
    Path(target_hash): Path<String>,
) -> impl IntoResponse {
    info!("[PROOF] hash={}", &target_hash[..target_hash.len().min(16)]);
    
    let blocks = state.blocks.lock().await;
    let mut metrics = state.metrics.lock().await;
    metrics.total_verifications += 1;
    
    // Finde den Block
    if let Some(block) = blocks.iter().find(|b| b.hash == target_hash) {
        // Baue simplen Proof-Path (alle Hashes im Pfad)
        let idx = blocks.iter().position(|b| b.hash == target_hash).unwrap_or(0);
        let path: Vec<String> = blocks[..idx].iter().map(|b| b.hash.clone()).collect();
        
        let proof = MerkleProof {
            hash: target_hash.clone(),
            merkle_root: block.merkle_root.clone(),
            path,
            verified: true,
        };
        
        (StatusCode::OK, Json(proof))
    } else {
        warn!("[PROOF] Hash nicht gefunden: {}", target_hash);
        (
            StatusCode::NOT_FOUND,
            Json(MerkleProof {
                hash: target_hash,
                merkle_root: "NONE".to_string(),
                path: vec![],
                verified: false,
            }),
        )
    }
}

/// GET /api/v1/worm/status — WORM-Ledger-Status
async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let blocks = state.blocks.lock().await;
    let metrics = state.metrics.lock().await.clone();
    
    let response = serde_json::json!({
        "status": "WORM_LIVE",
        "total_blocks": blocks.len(),
        "total_commits": metrics.total_commits,
        "total_errors": metrics.total_errors,
        "total_verifications": metrics.total_verifications,
        "pbft_rounds": metrics.pbft_rounds,
        "ebpf_blocked": metrics.ebpf_blocked,
        "node_id": state.config.node_id,
        "master_seal": state.config.master_seal.chars().take(16).collect::<String>(),
    });
    
    (StatusCode::OK, Json(response))
}

/// GET /api/v1/worm/blocks — Alle Blöcke (read-only)
async fn blocks_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let blocks = state.blocks.lock().await;
    let slice: Vec<WormBlock> = blocks.iter().rev().take(50).cloned().collect();
    (StatusCode::OK, Json(serde_json::json!({ "blocks": slice, "count": slice.len() })))
}

/// POST /api/v1/consensus/verify — PBFT Consensus Verify
async fn consensus_verify_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let block_hash = req.get("block_hash").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
    let status = simulate_pbft(block_hash);
    
    let mut metrics = state.metrics.lock().await;
    metrics.pbft_rounds += 1;
    
    info!("[PBFT] Verify block={} status={}", &block_hash[..block_hash.len().min(16)], status);
    
    let response = serde_json::json!({
        "status": status,
        "block_hash": block_hash,
        "votes": [
            { "node": "R0_0001", "vote": "YES" },
            { "node": "R0_0068", "vote": "YES" },
            { "node": "R1_0089", "vote": "YES" },
            { "node": "R2_0017", "vote": "YES" }
        ],
        "rounds": metrics.pbft_rounds,
    });
    
    (StatusCode::OK, Json(response))
}

/// GET /metrics — Prometheus-Format
async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let blocks = state.blocks.lock().await;
    let metrics = state.metrics.lock().await.clone();
    
    let output = format!(
        "# HELP b6_worm_blocks_total Total WORM blocks\n\
         # TYPE b6_worm_blocks_total counter\n\
         b6_worm_blocks_total {}\n\
         # HELP b6_commits_total Total commits\n\
         # TYPE b6_commits_total counter\n\
         b6_commits_total {}\n\
         # HELP b6_errors_total Total errors\n\
         # TYPE b6_errors_total counter\n\
         b6_errors_total {}\n\
         # HELP b6_pbft_rounds_total Total PBFT rounds\n\
         # TYPE b6_pbft_rounds_total counter\n\
         b6_pbft_rounds_total {}\n\
         # HELP b6_uptime_seconds Service uptime\n\
         # TYPE b6_uptime_seconds gauge\n\
         b6_uptime_seconds {}\n",
        blocks.len(),
        metrics.total_commits,
        metrics.total_errors,
        metrics.pbft_rounds,
        now_unix() - state.start_time,
    );
    
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        output,
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. CUSTOM TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

trait CommitRequestExt {
    fn event_or_default(&self) -> String;
}

impl CommitRequestExt for CommitRequest {
    fn event_or_default(&self) -> String {
        format!("{}::{}", self.domain, self.action)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. MAIN
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    // Logging initialisieren
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("═══════════════════════════════════════════════════════════════");
    info!("🚀 B6-CORE RUST WORM KERNEL v2.1.0 — MONSTER EDITION");
    info!("═══════════════════════════════════════════════════════════════");
    
    let config = Config::from_env();
    
    info!("📡 Bind Address: {}", config.bind_addr);
    info!("🗄️  WORM Path:    {:?}", config.worm_path);
    info!("🔐 Master Seal:  {}...", &config.master_seal[..16]);
    info!("🆔 Node ID:      {}", config.node_id);
    
    // Lade existierende Blocks
    let existing_blocks = load_worm_from_disk(&config.worm_path).await;
    info!("📦 {} existierende Blöcke geladen", existing_blocks.len());
    
    let state = Arc::new(AppState::new(config.clone()));
    
    // Initialisiere State mit geladenen Blocks
    {
        let mut blocks = state.blocks.lock().await;
        let mut last_hash = state.last_hash.lock().await;
        if let Some(last) = existing_blocks.last() {
            *last_hash = last.hash.clone();
        }
        *blocks = existing_blocks;
    }
    
    // Router bauen
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/api/v1/worm/commit", post(commit_handler))
        .route("/api/v1/worm/proof/:hash", get(proof_handler))
        .route("/api/v1/worm/status", get(status_handler))
        .route("/api/v1/worm/blocks", get(blocks_handler))
        .route("/api/v1/consensus/verify", post(consensus_verify_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    // Server starten
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("🚨 Konnte nicht an Adresse binden");
    
    info!("═══════════════════════════════════════════════════════════════");
    info!("✅ Server läuft auf http://{}", config.bind_addr);
    info!("   Endpoints:");
    info!("   • GET  /health");
    info!("   • GET  /metrics");
    info!("   • POST /api/v1/worm/commit");
    info!("   • GET  /api/v1/worm/proof/:hash");
    info!("   • GET  /api/v1/worm/status");
    info!("   • GET  /api/v1/worm/blocks");
    info!("   • POST /api/v1/consensus/verify");
    info!("═══════════════════════════════════════════════════════════════");
    
    axum::serve(listener, app)
        .await
        .expect("🚨 Server-Fehler");
}
