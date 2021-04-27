use std::collections::HashSet;
use curve25519_dalek::{
    ristretto::CompressedRistretto,
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    program_error::ProgramError,
};

pub struct WorldState {
    pub is_initialized: bool, // 1 byte
    pub initializer: Pubkey, // 32 bytes
    pub supply: u64, // 8 bytes
    pub agg_pk: AggPubkey, // 32 bytes
}

impl Sealed for WorldState {}
impl IsInitialized for WorldState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for WorldState {
    const LEN: usize = 73;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, WorldState::LEN];
        let (
            is_initialized,
            initializer,
            supply,
            agg_pk,
        ) = array_refs![src, 1, 32, 8, 32];

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(WorldState {
            is_initialized,
            initializer: Pubkey::new_from_array(*initializer),
            supply: u64::from_le_bytes(*supply),
            agg_pk: AggPubkey::new_from_array(*agg_pk),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, WorldState::LEN];
        let (
            is_initialized_dst,
            initializer_dst,
            supply_dst,
            agg_pk_dst,
        ) = mut_array_refs![dst, 1, 32, 8, 32];

        let WorldState {
            is_initialized,
            initializer,
            supply,
            agg_pk,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_dst.copy_from_slice(initializer.as_ref());
        *supply_dst = supply.to_le_bytes();
        agg_pk_dst.copy_from_slice(agg_pk.as_ref());
    }
}


pub struct AggPubkey(CompressedRistretto);
impl AggPubkey {
    pub fn new(point: CompressedRistretto) -> Self {
        Self(point)
    }

    pub fn new_from_array(bytes: [u8; 32]) -> Self {
        Self::new(CompressedRistretto(bytes))
    }
}

impl AsRef<[u8]> for AggPubkey {
    fn as_ref(&self) -> &[u8] {
        let AggPubkey(point) = self;
        point.as_bytes()
    }
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
