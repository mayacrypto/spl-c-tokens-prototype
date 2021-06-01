use borsh::{BorshSerialize, BorshDeserialize};

use crate::{
    error::CTokenError,
    proof::{
        BorshRangeProof, BorshRistretto, BorshScalar, PedersenComm, ProofKnowledge,
        PedersenBase, commit_pedersen,
    }
};
use sha3::Sha3_512;

use curve25519_dalek::{
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::Identity,
};
use rand_core::OsRng; // Only for generating commitments and proof of knowledge

// // use bulletproofs::{
// //     PedersenGens,
// //     BulletproofGens,
// // };
// // use merlin::Transcript;
// 
// // Bit length for the Bulletproof range proof. 
// // 
// // Proof verification scales linearly with the bit length. If 32 bits of token granularity suffices
// // for applications, then this will decrease the cost of verification by half.
// // 
// const RANGE_BIT_LENGTH: usize = 64;
// 
// const TX_VEC_SIZE: usize = 2;

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
/// multiplications are largely independent and hence parallelizable operations).
///
/// Notes on one-time usage of accounts:
///     - To prevent issues like front-running, an account is one-time-use per transaction. For
///     example, suppose that an account with address [pk] holds a commitment [comm]. When the user
///     spends tokens from this account, even if there are tokens left over from the transaction,
///     the user must create a new account [pk'] with the new commitment [comm'] and purge the
///     original account. There are other ways to prevent issues like front-running, but for the
///     purpose of the prototype code, we make accounts one-time-use per transaction.
///
///     - An optimization (from MimbleWimble) that can be made is to replace [pk] with the
///     commitment [comm]. Public keys and Pedersen commitments are all 32-bytes, and Pedersen
///     commitments are also randomly generated (and hence, collisions are highly unlikely). This
///     is quite natural when combined with the point above that accounts are one-time-use since
///     commitments constantly change with transactions (and hence, addresses constantly change).
///     for the prototype code, this optimization is not made to prevent possible confusion.
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
#[derive(BorshSerialize, BorshDeserialize, Copy, Clone)]
pub struct MintData {
    /// Amount of newly minted tokens
    pub amount: u64,
    /// Commitments produced
    pub out_comm: PedersenComm,
    /// Range proof for the produced commitment
    pub range_proof: BorshRangeProof,
    /// Proof of knowledge to validate transaction
    pub proof_knowledge: ProofKnowledge,
}
impl CryptoVerRequired for MintData {
    fn verify_crypto(&self) -> Result<(), CTokenError> {
        let Self { amount, out_comm, range_proof, proof_knowledge } = self;

        // Skipping range proof verification for now
        // 
        // // Verify range proofs
        // for (comm, range_proof) in out_comms {
        //     range_proof.verify_single(
        //         &BulletproofGens::new(RANGE_BIT_LENGTH, 1),
        //         &PedersenGens::default(),
        //         &mut Transcript::new(b""),
        //         &comm.getComm(),
        //         1,
        //     )?;
        // }

        // Setup for proof-of-knowledge verification
        let ProofKnowledge { nonce, scalar } = proof_knowledge;

        let c = Scalar::hash_from_bytes::<Sha3_512>(&nonce.to_bytes()); // get corresponding scalar
        let nonce = nonce.decompress().unwrap(); // decompress nonce component
        let PedersenBase{ G, .. } = PedersenBase::default(); // get corresponding base
        let amount_ristretto = Scalar::from(*amount)*G; // encode amount into a Ristretto point
        let out_comm_ristretto = out_comm.getComm().decompress().unwrap();

        // Check algebraic relation for proof-of-knowledge
        // if **scalar * G != c * (out_comm_ristretto - amount_ristretto) + nonce {
        //     return Err(CTokenError::InvalidProof);
        // }
        Ok(())
    }
}

/// Initializes a mint transaction.
/// 
/// This function should only be used for testing purposes. A real mint client
/// should have constant runtime.
///
pub fn sample_mint_client_for_test(amount: u64) -> MintData {
    // Generate commitment
    let pedersen_comm = PedersenComm::new(
        BorshRistretto::new(CompressedRistretto([0; 32]))
    );
    let out_comm = pedersen_comm;

    // Generate range proof for the commitment
    let range_proof = BorshRangeProof;

    // Generate proof of knowledge for the produced commitments
    let proof_knowledge = ProofKnowledge { 
        nonce: BorshRistretto::new(CompressedRistretto([0; 32])),
        scalar: BorshScalar::new(Scalar::default()),
    };

    // Return mint data
    MintData {
        amount,
        out_comm,
        range_proof,
        proof_knowledge,
    }
}

/// Data required for a Transfer instruction
///
/// Verification consist of:
/// - Range proof verification for each of the output commitments 
/// - Proof of knowledge verification that the sum of the input and output commitments contain 0
///
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TransferData {
    /// Sender and receiver source commitments
    pub in_comms: (PedersenComm, PedersenComm),
    /// Sender and receiver destination commitments
    pub out_comms: (PedersenComm, PedersenComm),
    /// Range proofs for the destination commitments
    pub range_proofs: (BorshRangeProof, BorshRangeProof),
    /// Proof of knowledge to validate transaction
    pub proofs_knowledge: (ProofKnowledge, ProofKnowledge),
}
impl CryptoVerRequired for TransferData {
    fn verify_crypto(&self) -> Result<(), CTokenError> {
        let Self { in_comms, out_comms, range_proofs, proofs_knowledge } = self;
        
        // Skipping range proof verification for now
        //
        // // Verify range proofs
        // for (comm, range_proof) in out_comms {
        //     range_proof.verify_single(
        //         &BulletproofGens::new(RANGE_BIT_LENGTH, 1),
        //         &PedersenGens::default(),
        //         &mut Transcript::new(b""),
        //         &comm.getComm(),
        //         1,
        //     )?;
        // }

        // Setup for proof-of-knowledge verification
        let (proof_knowledge_sender, proof_knowledge_receiver) = proofs_knowledge;
        let ProofKnowledge{ nonce: sender_nonce, scalar: sender_scalar } = proof_knowledge_sender;
        let ProofKnowledge{ nonce: receiver_nonce, scalar: receiver_scalar } = proof_knowledge_receiver;

        let sender_c = Scalar::hash_from_bytes::<Sha3_512>(&sender_nonce.to_bytes());
        let receiver_c = Scalar::hash_from_bytes::<Sha3_512>(&receiver_nonce.to_bytes());
        let sender_nonce = sender_nonce.decompress().unwrap(); // decompress nonce component
        let receiver_nonce = receiver_nonce.decompress().unwrap(); // unwrapping error for now

        let PedersenBase{ G, .. } = PedersenBase::default(); // get corresponding base

        let extract_comm = |x: PedersenComm| x.getComm().decompress().unwrap();
        let aggregate = (extract_comm(out_comms.0) + extract_comm(in_comms.0)) * sender_c;
        let aggregate = aggregate + (extract_comm(out_comms.1) - extract_comm(in_comms.1)) * receiver_c;

        // Check algebraic relation for proof-of-knowledge
        // if (**sender_scalar + **receiver_scalar) * G != aggregate + sender_nonce + receiver_nonce {
        //     return Err(CTokenError::InvalidProof);
        // }
        Ok(())
    }
}

/// Initializes a transaction.
///
/// A transation is initiated first by the sender who provides the receiver with information 
/// regarding its commiment and other values (see below). The receiver then combines the 
/// information provided by the sender with its own infomation and generates a transaction
/// data to be submitted to the blockchain.
/// 

/// Struct that models the information that the sender sends to the receiver of the token.
#[derive(Debug)]
pub struct SenderMessageToReceiver {
    /// The number of tokens that the sender wishes to send
    pub transfer_amount: u64,
    /// The current commitment associated with the sender's account
    pub sender_source_comm: PedersenComm,
    /// The commitment that will be associated withthe sender's account after the transaction
    pub sender_dest_comm: PedersenComm,
    /// The range proof to prove that the sender's new destination commitment is valid
    pub sender_dest_range_proof: BorshRangeProof,
    /// A temporary commitment to be provided to the receiver as specified in MimbleWimble
    pub interim_comm: PedersenComm,
    /// The opening for the temporary commitment
    pub interim_open: BorshScalar,
    /// Proof of knowledge validating the source and destination commitments
    pub proof_knowledge_sender: ProofKnowledge,
}

/// This is a function that generates a sender's message to be sent to the receiver
///
/// This function is only for testing purposes and to demonstrate the logic of the
/// protocol. A real transfer client should have constant runtime.
///
pub fn sample_transfer_sender_client_for_test(
        sender_source_comm: PedersenComm, 
        sender_source_open: BorshScalar,
        sender_source_amount: u64,
        transfer_amount: u64
    ) -> SenderMessageToReceiver {
    // Generate sender destination commitment
    let (sender_dest_comm, sender_dest_open) = commit_pedersen(sender_source_amount - transfer_amount);

    // Generate interim commitment
    let (interim_comm, interim_open) = commit_pedersen(transfer_amount);

    // Generate range proofs for the destination commitment
    let sender_dest_range_proof = BorshRangeProof;

    // Generate proof of knowledge for the produced commitments
    let nonce_scalar = Scalar::random(&mut OsRng);
    let nonce = RistrettoPoint::default() * nonce_scalar;
    
    let c_scalar = Scalar::hash_from_bytes::<Sha3_512>(nonce.compress().as_bytes());
    let proof_knowledge_scalar = 
        (*sender_source_open - *sender_dest_open - *interim_open) * c_scalar + nonce_scalar;

    let proof_knowledge_sender = ProofKnowledge{ 
        nonce: BorshRistretto::new(nonce.compress()), 
        scalar: BorshScalar::new(proof_knowledge_scalar),
    };

    // Return sender message
    SenderMessageToReceiver {
        transfer_amount,
        sender_source_comm,
        sender_dest_comm,
        sender_dest_range_proof,
        interim_comm,
        interim_open,
        proof_knowledge_sender,
    }
}

pub fn sample_transfer_receiver_client_for_test(
    sender_message: SenderMessageToReceiver,
    receiver_source_comm: PedersenComm,
    receiver_source_open: BorshScalar,
    ) -> TransferData {
    
    let SenderMessageToReceiver {
        transfer_amount,
        sender_source_comm,
        sender_dest_comm,
        sender_dest_range_proof,
        interim_comm,
        interim_open,
        proof_knowledge_sender,
    } = sender_message;
    
    // Verify validity of sender message (interim_comm used here)
    // - check range proof for sender_dest_comm
    // - check proof of knowledge_sender

    // Generate receiver destination commitment
    let (receiver_dest_comm, receiver_dest_open) = commit_pedersen(transfer_amount);

    // Generate range proof for the destination commitment
    let receiver_dest_range_proof = BorshRangeProof;

    // Generate proof of knowledge for the produced commitments
    let nonce_scalar = Scalar::random(&mut OsRng);
    let nonce = RistrettoPoint::default() * nonce_scalar;
    
    let c_scalar = Scalar::hash_from_bytes::<Sha3_512>(nonce.compress().as_bytes());
    let proof_knowledge_scalar = 
        (*receiver_source_open + *interim_open - *receiver_dest_open) * c_scalar + nonce_scalar;

    let proof_knowledge_receiver = ProofKnowledge{ 
        nonce: BorshRistretto::new(nonce.compress()), 
        scalar: BorshScalar::new(proof_knowledge_scalar),
    };

    TransferData {
        in_comms: (sender_source_comm, receiver_source_comm),
        out_comms: (sender_dest_comm, receiver_dest_comm),
        range_proofs: (sender_dest_range_proof, receiver_dest_range_proof),
        proofs_knowledge: (proof_knowledge_sender, proof_knowledge_receiver),
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
    // /// Commitment
    // pub comm: PedersenComm,
    // /// Commitment opening
    // pub open: BorshScalar,
}
impl CryptoVerRequired for CloseAccountData {
    fn verify_crypto(&self) -> Result<(), CTokenError> {
        // let Self { amount, comm, open } = self;

        // // Verify commitment and opening
        // if Pedersen::verify_commitment(
        //     comm,
        //     &PedersenBase::default(),
        //     &Scalar::from(*amount),
        //     open,
        // ) {
        //     Ok(())
        // } else {
        //     Err(CTokenError::OpeningInvalid)
        // }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    // TODO: Write tests here

}
