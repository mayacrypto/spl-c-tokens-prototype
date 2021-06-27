#![allow(dead_code, non_snake_case)]

use borsh::{BorshDeserialize, BorshSerialize};

use arrayref::array_ref;
use std::io;
use std::io::{Error, Write};
use std::ops::Deref;


use curve25519_dalek::{
    ristretto::CompressedRistretto, scalar::Scalar, ristretto::RistrettoPoint,
};

// #[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
// pub struct PedersenComm(pub(crate) [u8; 32]);

pub struct CommitmentBase {
    /// Base for the committed value
    pub G: RistrettoPoint,
    /// Base for the blinding factor
    pub H: RistrettoPoint,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PedersenComm {
    /// Ristretto point representing the commitment
    pub comm: BorshRistretto,
}

impl PedersenComm {
    /// Given a commitment along with the corresponding base points, opening,
    /// and the committed value, verifies the validity of the commitment.
    pub fn verify_commitment(
        comm: &PedersenComm,
        base: &CommitmentBase,
        open: &Scalar,
        val: &Scalar,
    ) -> bool {
        let CommitmentBase { G, H } = base;
        *comm.comm == (open * G + val * H).compress()
    }
}

// //  a struct for the El Gamal commitment (encryption)
//  Commitment = ((open * G), (val + open * H))
#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ElGamalComm {
    pub comm_h: BorshRistretto,
    pub comm_g: BorshRistretto, 
 }
 
impl ElGamalComm {
    pub fn verify_commitment(
        comm: &ElGamalComm,
        base: &CommitmentBase,
        open: &Scalar,
        val: &RistrettoPoint,
    ) -> bool {
        let CommitmentBase { G, H } = base;
        (*comm.comm_g == (open * G).compress()) & (*comm.comm_h == (val + open * H).compress())
    }
}


#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProofKnowledge {
    pub nonce: [u8; 32],
    pub scalar: [u8; 32],
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RangeProof;

/// Type wrapper of Scalar: to implement the Borsh Serialize/Deserialize traits
/// using the New Type Pattern.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BorshScalar(Scalar);
impl BorshScalar {
    pub fn new(scalar: Scalar) -> Self {
        Self(scalar)
    }
}
impl Deref for BorshScalar {
    type Target = Scalar;

    fn deref(&self) -> &Scalar {
        let Self(scalar) = self;
        scalar
    }
}
impl BorshSerialize for BorshScalar {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let Self(scalar) = self;
        let scalar_bytes = scalar.to_bytes();
        writer.write(&scalar_bytes)?;
        Ok(())
    }
}
impl BorshDeserialize for BorshScalar {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        let scalar = Scalar::from_canonical_bytes(*array_ref![buf, 0, 32]);
        if scalar.is_none() {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Scalar deserialize error",
            ));
        };
        *buf = &buf[32..];
        Ok(BorshScalar(scalar.unwrap()))
    }
}

/// Type wrapper for CompressedRistretto: to implement the Borsh
/// Serialize/Deserialize traits using the New Type Pattern.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BorshRistretto(CompressedRistretto);
impl BorshRistretto {
    pub fn new(ristretto: CompressedRistretto) -> Self {
        Self(ristretto)
    }
}
impl Deref for BorshRistretto {
    type Target = CompressedRistretto;

    fn deref(&self) -> &CompressedRistretto {
        let Self(ristretto) = self;
        ristretto
    }
}
impl BorshSerialize for BorshRistretto {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let Self(ristretto) = self;
        let ristretto_bytes = ristretto.to_bytes();
        writer.write(&ristretto_bytes)?;
        Ok(())
    }
}
impl BorshDeserialize for BorshRistretto {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        let ristretto = CompressedRistretto(*array_ref![buf, 0, 32]);
        *buf = &buf[32..];
        Ok(BorshRistretto(ristretto))
    }
}
