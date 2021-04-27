use std::collections::HashSet;

use solana_program::pubkey::Pubkey;

#[derive(Default)]
pub struct WorldState {
    supply: u64,
    input: HashSet<Commitment>,
    output: HashSet<Commitment>,
}

#[derive(Default)]
pub struct MintTX {
    supply: u64,
    output: HashSet<Commitment>,
}

#[derive(Default)]
pub struct TransferTX {
    input: HashSet<CommInput>,
    output: HashSet<CommOutput>,
    sig: Sig,
}

pub struct CommInput;
pub struct CommOutput {
    comm: Commitment,
    range_proof: RangeProof,
    proof_knowledge: ProofKnowledge,
}
impl CommOutput {
    pub fn verify(comm: CommOutput) -> bool {
        let CommOutput {
            comm,
            range_proof,
            proof_knowledge
        } = comm;

        let b_rp = RangeProof::verify(&comm, &range_proof);
        let b_pk = ProofKnowledge::verify(&comm, &proof_knowledge);
        b_rp && b_pk
    }
}

pub struct Commitment([u8; 32]);
impl From<Commitment> for Pubkey {
    fn from(comm: Commitment) -> Self {
        let Commitment(bytes) = comm;
        Pubkey::new(&bytes)
    }
}

pub struct RangeProof;
impl RangeProof {
    pub fn verify(comm: &Commitment, range_proof: &RangeProof) -> bool {
        true
    }
}

pub struct ProofKnowledge;
impl ProofKnowledge {
    pub fn verify(comm: &Commitment, proof_knowledge: &ProofKnowledge) -> bool {
        true
    }
}

#[derive(Default)]
pub struct Sig;
