#![allow(non_snake_case)]

use curve25519_dalek::{
    ristretto::RistrettoPoint,
    ristretto::CompressedRistretto,
    scalar::Scalar,
    traits::Identity,
    constants::RISTRETTO_BASEPOINT_POINT,
    constants::RISTRETTO_BASEPOINT_COMPRESSED,
};
use sha3::Sha3_512;
use bulletproofs::{
    PedersenGens,
    BulletproofGens,
    RangeProof,
    ProofError,
};
use merlin::Transcript;
use rand_core::OsRng;
use crate::error::CTokenError;

const RANGE_BIT_LENGTH: usize = 64;

pub struct MintTransaction {
    /// Supply of newly minted tokens
    pub supply: u64,
    /// Commitments produced
    pub out_comms: OutComms,
    /// Proof of knowledge to validate transaction
    pub proof_knowledge: ProofKnowledge,
}
impl MintTransaction {
    pub fn verify_proofs(&self) -> Result<(), CTokenError> {
        let Self { supply, out_comms, proof_knowledge } = self;
        out_comms.verify_range_proofs()?;

        let OutComms { comms, .. } = out_comms;
        let ProofKnowledge { nonce, scalar } = proof_knowledge;
        // ignoring decompression error for now
        let nonce = nonce.decompress().unwrap();
        let mut aggregate = RistrettoPoint::identity();
        comms.iter().for_each(| PedersenComm{ C } | {
            aggregate = aggregate + C.decompress().unwrap();
        });
        let PedersenBase{ G, .. } = PedersenBase::default();

        let c = Scalar::hash_from_bytes::<Sha3_512>(b"");
        let supply = Scalar::from(*supply)*G;
        if scalar*G == c*(aggregate - supply) + nonce {
            return Err(CTokenError::InvalidProof);
        }

        Ok(())
    }
}


pub struct TransferTransaction {
    /// Commitments spent
    pub in_comms: InComms,
    /// Commitments produced
    pub out_comms: OutComms,
    /// Proof of knowledge to validate transaction
    pub proof_knowledge: ProofKnowledge,
}
impl TransferTransaction {
    pub fn verify_proofs(&self) -> bool {
        true
    }
}

pub struct InComms {
    /// List of commitments spent
    pub comms: Vec<PedersenComm>,
}

pub struct OutComms {
    /// List of new commitments produced
    pub comms: Vec<PedersenComm>,
    /// List of range proofs
    pub range_proofs: Vec<RangeProof>,
}
impl OutComms {
    pub fn verify_range_proofs(&self) -> Result<(), ProofError> {
        let Self { comms, range_proofs } = self;
        for (comm, proof) in comms.iter().zip(range_proofs.iter()) {
            proof.verify_single(
                &BulletproofGens::new(RANGE_BIT_LENGTH, 1),
                &PedersenGens::default(),
                &mut Transcript::new(b""),
                &comm.C,
                1,
            )?;
        }
        Ok(())
    }
}


pub struct ProofKnowledge {
    /// Nonce component
    pub nonce: CompressedRistretto,
    /// Scalar component
    pub scalar: Scalar,
}

// TODO: Consider using a commitment trait to unify syntax for ElGamal and Pedersen
pub struct ElGamal;
pub struct ElGamalBase {
    /// Base for the committed value
    pub G: RistrettoPoint,
    /// Base for the blinding factor
    pub H: RistrettoPoint,
}
impl Default for ElGamalBase {
    fn default() -> Self {
        ElGamalBase {
            G: RISTRETTO_BASEPOINT_POINT,
            H: RistrettoPoint::hash_from_bytes::<Sha3_512>(
                RISTRETTO_BASEPOINT_COMPRESSED.as_bytes(),
            ),
        }
    }
}
pub struct ElGamalComm {
    /// Randomness component
    pub R: CompressedRistretto,
    /// Payload component
    pub C: CompressedRistretto,
}
impl ElGamal {
    pub fn commit(base: ElGamalBase, val: Scalar) -> (ElGamalComm, Scalar) {
        let ElGamalBase{ G, H } = base;
        let mut rng = OsRng;
        let open = Scalar::random(&mut rng);

        let R = open*G;
        let C = R + val*H;
        let comm = ElGamalComm{ 
            R: R.compress(), 
            C: C.compress() 
        };
        (comm, open)
    }
}

pub struct Pedersen;
pub struct PedersenBase {
    /// Base for the committed value
    pub G: RistrettoPoint,
    /// Base for the blinding factor
    pub H: RistrettoPoint,
}
impl Default for PedersenBase {
    fn default() -> Self {
        PedersenBase {
            G: RISTRETTO_BASEPOINT_POINT,
            H: RistrettoPoint::hash_from_bytes::<Sha3_512>(
                RISTRETTO_BASEPOINT_COMPRESSED.as_bytes(),
            ),
        }
    }
}
pub struct PedersenComm {
    /// Commitment
    pub C: CompressedRistretto,
}
impl Pedersen {
    pub fn commit(base: PedersenBase, val: Scalar) -> (PedersenComm, Scalar) {
        let PedersenBase{ G, H } = base;
        let mut rng = OsRng;
        let open = Scalar::random(&mut rng);
        let C = open * G + val * H;
        let comm = PedersenComm {
            C: C.compress(),
        };
        (comm, open)
    }
}
