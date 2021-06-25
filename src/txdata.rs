use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    crypto::{PedersenComm, ProofKnowledge, RangeProof},
    error::CTokenError,
};

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
    pub range_proof: RangeProof,
    /// Proof of knowledge to validate transaction
    pub proof_knowledge: ProofKnowledge,
}
impl CryptoVerRequired for MintData {
    /// Verifies all required crypto components. Currently, the actual verification is commented
    /// out to calculate BPF instruction counts without the crypto components.
    fn verify_crypto(&self) -> Result<(), CTokenError> {
        // let Self {
        //     amount,
        //     out_comm,
        //     range_proof,
        //     proof_knowledge,
        // } = self;

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
        // let ProofKnowledge { nonce, scalar } = proof_knowledge;

        // let c = Scalar::hash_from_bytes::<Sha3_512>(&nonce.to_bytes()); // get corresponding scalar
        // let nonce = nonce.decompress().unwrap(); // decompress nonce component
        // let PedersenBase { G, .. } = PedersenBase::default(); // get corresponding base
        // let amount_ristretto = Scalar::from(*amount) * G; // encode amount into a Ristretto point
        // let out_comm_ristretto = out_comm.getComm().decompress().unwrap();

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
// pub fn sample_mint_client_for_test(amount: u64) -> MintData {
//     // Generate commitment
//     let pedersen_comm = PedersenComm([0; 32]);
//     let out_comm = pedersen_comm;
// 
//     // Generate range proof for the commitment
//     let range_proof = RangeProof;
// 
//     // Generate proof of knowledge for the produced commitments
//     let proof_knowledge = ProofKnowledge {
//         nonce: [0; 32],
//         scalar: [0; 32],
//     };
// 
//     // Return mint data
//     MintData {
//         amount,
//         out_comm,
//         range_proof,
//         proof_knowledge,
//     }
// }

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
    pub range_proofs: (RangeProof, RangeProof),
    /// Proof of knowledge to validate transaction
    pub proofs_knowledge: (ProofKnowledge, ProofKnowledge),
}
impl CryptoVerRequired for TransferData {
    fn verify_crypto(&self) -> Result<(), CTokenError> {
        // let Self {
        //     in_comms,
        //     out_comms,
        //     range_proofs,
        //     proofs_knowledge,
        // } = self;

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

        // // Setup for proof-of-knowledge verification
        // let (proof_knowledge_sender, proof_knowledge_receiver) = proofs_knowledge;
        // let ProofKnowledge {
        //     nonce: sender_nonce,
        //     scalar: sender_scalar,
        // } = proof_knowledge_sender;
        // let ProofKnowledge {
        //     nonce: receiver_nonce,
        //     scalar: receiver_scalar,
        // } = proof_knowledge_receiver;

        // let sender_c = Scalar::hash_from_bytes::<Sha3_512>(&sender_nonce.to_bytes());
        // let receiver_c = Scalar::hash_from_bytes::<Sha3_512>(&receiver_nonce.to_bytes());
        // let sender_nonce = sender_nonce.decompress().unwrap(); // decompress nonce component
        // let receiver_nonce = receiver_nonce.decompress().unwrap(); // unwrapping error for now

        // let PedersenBase { G, .. } = PedersenBase::default(); // get corresponding base

        // let extract_comm = |x: PedersenComm| x.getComm().decompress().unwrap();
        // let aggregate = (extract_comm(out_comms.0) + extract_comm(in_comms.0)) * sender_c;
        // let aggregate =
        //     aggregate + (extract_comm(out_comms.1) - extract_comm(in_comms.1)) * receiver_c;

        // Check algebraic relation for proof-of-knowledge
        // if (**sender_scalar + **receiver_scalar) * G != aggregate + sender_nonce + receiver_nonce {
        //     return Err(CTokenError::InvalidProof);
        // }
        Ok(())
    }
}

