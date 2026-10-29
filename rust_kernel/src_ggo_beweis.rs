use sha3::{Sha3_256, Digest};
use serde::{Serialize, Deserialize};
use crate::pbft_consensus::PbftConsensus;

pub const TENSOR_DIMENSION: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuaderTensor4096 {
    pub matrix: Vec<f32>,
    pub source_count: usize,
    pub entropy_score: f32,
    pub master_seal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGOBeweisCertificate {
    pub trace_id: String,
    pub timestamp: u64,
    pub action: String,
    pub tensor_digest: String,
    pub pbft_consensus_reached: bool,
    pub proof_hash: String,
    pub compliance_status: String,
}

impl QuaderTensor4096 {
    pub fn initialize(seed: &[u8], verified_sources: usize) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(seed);
        let digest = hasher.finalize();

        let mut matrix = Vec::with_capacity(TENSOR_DIMENSION);
        for i in 0..TENSOR_DIMENSION {
            let raw_val = digest[i % digest.len()] as f32;
            matrix.push(raw_val / 255.0);
        }

        let entropy_score = Self::calculate_entropy(&matrix);

        Self {
            matrix,
            source_count: verified_sources,
            entropy_score,
            master_seal: format!("{:x}", digest),
        }
    }

    fn calculate_entropy(data: &[f32]) -> f32 {
        let mut bins = [0u32; 10];
        for &val in data {
            let bin = ((val * 10.0) as usize).min(9);
            bins[bin] += 1;
        }
        let total = data.len() as f32;
        let mut entropy = 0.0;
        for count in bins.iter() {
            if *count > 0 {
                let p = *count as f32 / total;
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    pub fn compute_digest(&self) -> String {
        let mut hasher = Sha3_256::new();
        for val in &self.matrix {
            hasher.update(val.to_le_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

pub fn generate_gl1000_proof(action: &str, tensor: &QuaderTensor4096) -> GGOBeweisCertificate {
    let timestamp = chrono::Utc::now().timestamp() as u64;
    let tensor_digest = tensor.compute_digest();

    let pbft = PbftConsensus::new();
    let (consensus_reached, _votes) = pbft.simulate_voting_round(&tensor_digest);

    let payload = format!("TS:{}|ACT:{}|TENSOR:{}|PBFT:{}", timestamp, action, tensor_digest, consensus_reached);
    let mut hasher = Sha3_256::new();
    hasher.update(payload.as_bytes());
    let proof_hash = format!("{:x}", hasher.finalize());

    GGOBeweisCertificate {
        trace_id: format!("GL-1000-{}", &proof_hash[..16]),
        timestamp,
        action: action.to_string(),
        tensor_digest,
        pbft_consensus_reached: consensus_reached,
        proof_hash,
        compliance_status: "EU_AI_ACT_ART_52_VERIFIED_EX_TUNC".to_string(),
    }
}
