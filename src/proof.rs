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
use borsh::{BorshSerialize, BorshDeserialize};
use bulletproofs::{
    PedersenGens,
    BulletproofGens,
    RangeProof,
    ProofError,
};
use merlin::Transcript;
use rand_core::OsRng;
use crate::error::CTokenError;
use std::io::{Write, Error};
use std::io;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

const RANGE_BIT_LENGTH: usize = 64;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct MintData {
    /// Amount of newly minted tokens
    pub amount: u64,
    /// Commitments produced
    pub out_comms: OutComms,
    /// Proof of knowledge to validate transaction
    pub proof_knowledge: ProofKnowledge,
}
impl MintData {
    pub fn verify_proofs(&self) -> Result<(), CTokenError> {
        let Self { amount, out_comms, proof_knowledge } = self;
        out_comms.verify_range_proofs()?;

        let OutComms { comms_proofs } = out_comms;
        let ProofKnowledge { nonce, scalar } = proof_knowledge;
        
        let c = Scalar::hash_from_bytes::<Sha3_512>(&nonce.to_bytes());
        // ignoring decompression error for now
        let nonce = nonce.decompress().unwrap();
        let mut aggregate = RistrettoPoint::identity();
        comms_proofs.iter().for_each(| PedersenCommRange{ comm, .. } | {
            aggregate = aggregate + comm.C.decompress().unwrap();
        });
        let PedersenBase{ G, .. } = PedersenBase::default();

        let amount = Scalar::from(*amount)*G;
        if scalar*G == c*(aggregate - amount) + nonce {
            return Err(CTokenError::InvalidProof);
        }
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TransferData {
    /// Commitments spent
    pub in_comms: InComms,
    /// Commitments produced
    pub out_comms: OutComms,
    /// Proof of knowledge to validate transaction
    pub proof_knowledge: ProofKnowledge,
}
impl TransferData {
    pub fn verify_proofs(&self) -> Result<(), CTokenError> {
        let Self { in_comms, out_comms, proof_knowledge } = self;
        out_comms.verify_range_proofs()?;

        let InComms { comms: in_comms } = in_comms;
        let OutComms { comms_proofs } = out_comms;
        let ProofKnowledge { nonce, scalar } = proof_knowledge;

        let c = Scalar::hash_from_bytes::<Sha3_512>(&nonce.to_bytes());
        // ignoring decompression error for now
        let nonce = nonce.decompress().unwrap();
        let mut aggregate = RistrettoPoint::identity();
        in_comms.iter().for_each(| PedersenComm{ C } | {
            aggregate = aggregate - C.decompress().unwrap();
        });
        comms_proofs.iter().for_each(| PedersenCommRange{ comm, .. } | {
            aggregate = aggregate + comm.C.decompress().unwrap();
        });
        let PedersenBase{ G, .. } = PedersenBase::default();

        if scalar*G == c*aggregate + nonce {
            return Err(CTokenError::InvalidProof);
        }

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct InComms {
    /// List of commitments spent
    pub comms: Vec<PedersenComm>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct OutComms {
    /// List of produced commitments with attached range proofs
    pub comms_proofs: Vec<PedersenCommRange>,
}
impl OutComms {
    pub fn verify_range_proofs(&self) -> Result<(), ProofError> {
        let Self { comms_proofs } = self;
        for PedersenCommRange{ comm, range_proof } in comms_proofs {
            range_proof.verify_single(
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
impl BorshSerialize for ProofKnowledge {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let Self { nonce, scalar } = self;
        let nonce_bytes = nonce.as_bytes();
        let scalar_bytes = scalar.as_bytes();

        writer.write(nonce_bytes)?;
        writer.write(scalar_bytes)?;
        Ok(())
    }
}
impl BorshDeserialize for ProofKnowledge {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        let buf = array_ref![buf, 0, 64];
        let ( nonce, scalar ) = array_refs![buf, 32, 32];
        let scalar = Scalar::from_canonical_bytes(*scalar)
            .ok_or(std::io::ErrorKind::InvalidData)?;
        Ok( ProofKnowledge {
            nonce: CompressedRistretto(*nonce),
            scalar,
        })
    }
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
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
impl BorshSerialize for PedersenComm {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let Self { C } = self;
        writer.write_all(C.as_bytes())?;
        Ok(())
    }
}
impl BorshDeserialize for PedersenComm {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        let C = array_ref![buf, 0, 32];
        Ok( PedersenComm {
            C: CompressedRistretto(*C),
        })
    }
}

pub struct PedersenCommRange {
    /// Pedersen commitment
    pub comm: PedersenComm,
    pub range_proof: RangeProof,
}
impl BorshSerialize for PedersenCommRange {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let Self { comm, range_proof } = self;
        let comm_bytes = comm.try_to_vec()?;
        let range_proof_bytes = range_proof.to_bytes();

        writer.write(&comm_bytes)?;
        writer.write(&range_proof_bytes)?;

        Ok(())
    }
}
impl BorshDeserialize for PedersenCommRange {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        let buf = array_ref![buf, 0, 736];
        let ( comm, range_proof ) = array_refs![buf, 32, 704];
        let range_proof = RangeProof::from_bytes(range_proof)
            .or(Err(std::io::ErrorKind::InvalidData))?;
        Ok( PedersenCommRange {
            comm: PedersenComm{ C: CompressedRistretto(*comm) },
            range_proof,
        })
    }
}
