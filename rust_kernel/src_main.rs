use b6_core::{
    worm_ledger::WORMLedger,
    ggo_beweis::{QuaderTensor4096, generate_gl1000_proof},
    bila_core::{BilaReflexCore, BilaReflexError},
    pbft_consensus::PbftConsensus,
};

fn main() {
    println!("================================================================================");
    println!("☀️ BLACK SIX MATRIX — RUST CORE ENGINE (EDITION 2021/2024)");
    println!("================================================================================");

    let tensor = QuaderTensor4096::initialize(b"ODIN_ULTRA_ROOT_SEED", 278);
    let digest = tensor.compute_digest();
    println!("[+] Quader 4096 Tensor Initialisiert.");
    println!("    Digest:  {}...", &digest[..16]);
    println!("    Entropie: {:.4}", tensor.entropy_score);

    let cert = generate_gl1000_proof("DSGVO_Art22_Audit", &tensor);
    println!("[+] GGO-Beweiszertifikat Erzeugt:");
    println!("    ID:      {}", cert.trace_id);
    println!("    PBFT:    {}", cert.pbft_consensus_reached);
    println!("    Status:  {}", cert.compliance_status);
    println!("    Hash:    {}...", &cert.proof_hash[..16]);

    let mut reflex_core = BilaReflexCore::new();
    let cmd = reflex_core.decode_bitstream(0b0001_0000_1111_0000);
    match reflex_core.execute_reflex(cmd) {
        Ok(msg) => println!("[+] BILA Core Test 1: {}", msg),
        Err(e) => println!("[-] BILA Core Test 1 Fehler: {}", e),
    }

    let cmd_fail = reflex_core.decode_bitstream(0b1111_0000_0000_0000);
    match reflex_core.execute_reflex(cmd_fail) {
        Ok(msg) => println!("[+] BILA Core Test 2: {}", msg),
        Err(BilaReflexError::EmergencyCutout) => println!("[+] BILA Core Test 2: Erwarteter Emergency Cutout ausgelöst."),
        Err(e) => println!("[-] BILA Core Test 2 Fehler: {}", e),
    }

    let mut ledger = WORMLedger::new();
    let block = ledger.append_block(cert.proof_hash.as_bytes());
    println!("[+] WORM Ledger Block #{} angehängt.", block.index);
    println!("    Block Hash:  {}...", &block.block_hash[..16]);

    let pbft = PbftConsensus::new();
    let (consensus, votes) = pbft.simulate_voting_round(&block.block_hash);
    println!("[+] PBFT Konsens Check: {}", consensus);
    for vote in votes {
        println!("    Node {}: {} (Sig: {}...)", vote.node_id, if vote.vote { "YES" } else { "NO" }, &vote.signature[..8]);
    }

    let is_valid = ledger.verify_integrity();
    println!("[+] WORM Integrity: {}", if is_valid { "VERIFIED (Tamper-Proof)" } else { "CORRUPTED" });
    println!("================================================================================");
}
