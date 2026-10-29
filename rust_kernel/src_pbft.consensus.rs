use sha3::{Sha3_256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbftVote {
    pub node_id: String,
    pub vote: bool,
    pub signature: String,
}

pub struct PbftConsensus {
    pub nodes: Vec<String>,
    pub quorum_threshold: usize,
}

impl PbftConsensus {
    pub fn new() -> Self {
        let nodes = vec![
            "R0_0001".to_string(), "R0_0068".to_string(),
            "R1_0089".to_string(), "R2_0017".to_string(),
            "R5_0061".to_string()
        ];
        let quorum_threshold = (nodes.len() * 2 / 3) + 1;
        Self { nodes, quorum_threshold }
    }

    pub fn simulate_voting_round(&self, block_hash: &str) -> (bool, Vec<PbftVote>) {
        let mut votes = Vec::new();
        let mut positive_votes = 0;

        for node in &self.nodes {
            let mut hasher = Sha3_256::new();
            hasher.update(format!("{}:{}", node, block_hash).as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            let vote = hash.chars().next().unwrap_or('0') > '7';
            if vote { positive_votes += 1; }

            votes.push(PbftVote {
                node_id: node.clone(),
                vote,
                signature: hash,
            });
        }

        (positive_votes >= self.quorum_threshold, votes)
    }
}
