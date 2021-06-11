use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    crypto::{PedersenComm, ProofKnowledge, RangeProof},
    error::CTokenError,
};


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
pub fn sample_mint_client_for_test(amount: u64) -> MintData {
    // Generate commitment
    let pedersen_comm = PedersenComm([0; 32]);
    let out_comm = pedersen_comm;

    // Generate range proof for the commitment
    let range_proof = RangeProof;

    // Generate proof of knowledge for the produced commitments
    let proof_knowledge = ProofKnowledge {
        nonce: [0; 32],
        scalar: [0; 32],
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

