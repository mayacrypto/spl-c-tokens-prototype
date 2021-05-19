
use borsh::{BorshSerialize, BorshDeserialize};

use crate::{
    error::CTokenError,
    proof::{Pedersen, PedersenComm, PedersenBase, ProofKnowledge, 
        BorshScalar, BorshRangeProof,
    },
};

use curve25519_dalek::{
    scalar::Scalar,
    ristretto::RistrettoPoint,
    traits::Identity,
};
use bulletproofs::{
    PedersenGens,
    BulletproofGens,
};
use merlin::Transcript;
use sha3::Sha3_512;

// Bit length for the Bulletproof range proof. 
// 
// Proof verification scales linearly with the bit length. If 32 bits of token granularity suffices
// for applications, then this will decrease the cost of verification by half.
// 
const RANGE_BIT_LENGTH: usize = 64;

const TX_VEC_SIZE: usize = 2;

/// Intuitively, we can think of an SPL confidential token account as a regular SPL token account
/// but where the token amount is wrapped inside commitments. We can intuitively think of
/// commitments as an encryption of the token amount. In this prototype code, we specifically use
/// Pedersen commitments, which consists of a single 32-byte compressed Ristretto point.
///
/// The use of Pedersen commitments is standard for settings where confidentiality, but not
/// anonymity (hiding the TX graph), is of concern. For settings where we want both confidentiality
/// and anonymity, we would use El Gamal, which consists of two 32-byte compressed Ristretto
/// points.
/// 
/// Since token amounts are wrapped inside commitments, complications do arise in how we want to
/// manage these accounts regarding issues like rent. For the prototype code, we put these issues
/// aside and focus on transaction verification since we are primarily interested in the cost of
/// these verifications.
///
/// A transaction consists of a vector of input commitments and output commitments (for the
/// prototype code, we use an array of fixed size 2 for simplicity). An input commitment is a
/// commitment held by one of the existing accounts. An output commitment is a commitment that is
/// newly created by the transaction.
///
/// For transaction verification, we must verify the following:
/// 1. Are the input commitments valid?
///
///     Given that input commitments are just output commitments that were produced in previous
///     transactions, we can forgo input verification given that output commitments are verified
///     correctly.
///
/// 2. Are the output commitments valid?
///
///     Output verification consist of verifying that the commitments are valid Pedersen
///     commitments that wrap values in the range [0, 2^64]. This is done by range proofs in
///     Bulletproofs.
///
/// 3. Are the input and output commitments consistent?
///
///     Consistency verification consist of verifying that the sum of the values inside the input
///     commitments are equal to the sum of the values inside the output commitments. This is done
///     by a custom proof-of-knowledge verification algorithm in proof.rs.
///
/// The proof-of-knowledge verification is essentially the cost of doing one Ed25519 signature
/// verification. The bulk of the verification will be verifying the range proofs, which should
/// require around 64*2 elliptic curve multiplication on Ristretto points (though each of these
/// multiplications are largely independent and hence parallelizable operation).
///
/// Notes on one-time usage of accounts:
///     - To prevent issues like front-running, an account is one-time-use per transaction. For
///       example, suppose that an account with address [pk] holds a commitment [comm]. When the user
///       spends tokens from this account, even if there are tokens left over from the transaction,
///       the user must create a new account [pk'] with the new commitment [comm'] and purge the original
///       account. There are other ways to prevent issues like front-running, but for the purpose
///       of the prototype code, we make accounts one-time-use per transactin.
///
///     - One optimization (from MimbleWimble) that can be made is to replace [pk] with the
///       commitment [comm]. Public keys and Pedersen commitments are all 32-bytes, and Pedersen
///       commitments are also randomly generated (and hence, collisions are highly unlikely). This
///       is quite natural when combined with the point above that accounts are one-time-use since
///       commitments constantly change with transactions (and hence, addresses constantly change). 
///       for the prototype code, this optimization is not made to prevent possible confusion.
///


/// Trait for any transaction data requiring direct cryptographic verification using on-chain code.
pub trait CryptoVerRequired {
    fn verify_crypto(&self) -> Result<(), CTokenError>;
}

/// Data required for a Mint instruction
///
/// There are no input commitments, but only output commitments. Verification consist of:
/// - Range proof verification for each of the output commitments 
/// - Proof of knowledge verification that the sum of the output commitments indeed contain the
///   specified amount create
///
#[derive(BorshSerialize, BorshDeserialize)]
pub struct MintData {
    /// Amount of newly minted tokens
    pub amount: u64,
    /// Commitments produced
    pub out_comms: Vec<(PedersenComm, BorshRangeProof)>,
    /// Proof of knowledge to validate transaction
    pub proof_knowledge: ProofKnowledge,
}
impl CryptoVerRequired for MintData {
    fn verify_crypto(&self) -> Result<(), CTokenError> {
        let Self { amount, out_comms, proof_knowledge } = self;

        // Verify range proofs
        for (comm, range_proof) in out_comms {
            range_proof.verify_single(
                &BulletproofGens::new(RANGE_BIT_LENGTH, 1),
                &PedersenGens::default(),
                &mut Transcript::new(b""),
                &comm.getComm(),
                1,
            )?;
        }

        // Setup for proof-of-knowledge verification
        let ProofKnowledge { nonce, scalar } = proof_knowledge;
        
        let c = Scalar::hash_from_bytes::<Sha3_512>(&nonce.to_bytes()); // get corresponding scalar
        let nonce = nonce.decompress().unwrap(); // decompress nonce component
        let PedersenBase{ G, .. } = PedersenBase::default(); // get corresponding base
        let amount = Scalar::from(*amount)*G; // encode amount into a Ristretto point

        let mut aggregate = RistrettoPoint::identity(); // sum up Ristretto components for each commitment
        for (comm, _) in out_comms {
            aggregate = aggregate + comm.getComm().decompress().unwrap();
        }

        // Check algebraic relation for proof-of-knowledge
        if **scalar * G == c * (aggregate - amount) + nonce {
            return Err(CTokenError::InvalidProof);
        }
        Ok(())
    }
}

/// Data required for a Transfer instruction
///
/// Verification consist of:
/// - Range proof verification for each of the output commitments 
/// - Proof of knowledge verification that the sum of the input and output commitments contain 0
///
#[derive(BorshSerialize, BorshDeserialize)]
pub struct TransferData {
    /// Commitments spent
    pub in_comms: Vec<PedersenComm>,
    /// Commitments produced
    pub out_comms: Vec<(PedersenComm, BorshRangeProof)>,
    /// Proof of knowledge to validate transaction
    pub proof_knowledge: ProofKnowledge,
}
impl CryptoVerRequired for TransferData {
    fn verify_crypto(&self) -> Result<(), CTokenError> {
        let Self { in_comms, out_comms, proof_knowledge } = self;
        
        // Verify range proofs
        for (comm, range_proof) in out_comms {
            range_proof.verify_single(
                &BulletproofGens::new(RANGE_BIT_LENGTH, 1),
                &PedersenGens::default(),
                &mut Transcript::new(b""),
                &comm.getComm(),
                1,
            )?;
        }

        // Setup for proof-of-knowledge verification
        let ProofKnowledge { nonce, scalar } = proof_knowledge;

        let c = Scalar::hash_from_bytes::<Sha3_512>(&nonce.to_bytes()); // get corresponding scalar
        let nonce = nonce.decompress().unwrap(); // decompress nonce component
        let PedersenBase{ G, .. } = PedersenBase::default(); // get corresponding base

        let mut aggregate = RistrettoPoint::identity(); // sum up Ristretto components for each commitment
        for comm in in_comms {
            aggregate = aggregate - comm.getComm().decompress().unwrap();
        }
        for (comm, _) in out_comms {
            aggregate = aggregate + comm.getComm().decompress().unwrap();
        }

        // Check algebraic relation for proof-of-knowledge
        if **scalar * G == c * aggregate + nonce {
            return Err(CTokenError::InvalidProof);
        }
        Ok(())
    }
}

/// Data required for a CloseAccount instruction
///
/// Verification consist of:
/// - Checking that the provided commitment and opening match
///
#[derive(BorshSerialize, BorshDeserialize)]
pub struct CloseAccountData {
    /// Claimed number of tokens
    pub amount: u64,
    /// Commitment
    pub comm: PedersenComm,
    /// Commitment opening
    pub open: BorshScalar,
}
impl CryptoVerRequired for CloseAccountData {
    fn verify_crypto(&self) -> Result<(), CTokenError> {
        let Self { amount, comm, open } = self;

        // Verify commitment and opening
        if Pedersen::verify_commitment(
            comm,
            &PedersenBase::default(),
            &Scalar::from(*amount),
            open,
        ) {
            Ok(())
        } else {
            Err(CTokenError::OpeningInvalid)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    // TODO: Write tests here

}
