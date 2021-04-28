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

/// Mint data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mint {
    /// Mint authority.
    pub mint_authority: Pubkey, // 32 bytes
    /// Total supply of tokens.
    pub supply: u64, // 8 bytes
    /// Is `true` if this structure has been initialized
    pub is_initialized: bool, // 1 byte
}
impl Sealed for Mint {}
impl IsInitialized for Mint {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for Mint {
    const LEN: usize = 41;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Mint::LEN];
        let (
            mint_authority, 
            supply, 
            is_initialized
        ) = array_refs![src, 32, 8, 1];

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Mint {
            mint_authority: Pubkey::new_from_array(*mint_authority),
            supply: u64::from_le_bytes(*supply),
            is_initialized,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 41];
        let (
            mint_authority_dst,
            supply_dst,
            is_initialized_dst,
        ) = mut_array_refs![dst, 32, 8, 1];
        let &Mint {
            ref mint_authority,
            supply,
            is_initialized,
        } = self;
        mint_authority_dst.copy_from_slice(mint_authority.as_ref());
        *supply_dst = supply.to_le_bytes();
        is_initialized_dst[0] = is_initialized as u8;
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
