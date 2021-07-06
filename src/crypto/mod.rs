
#![allow(dead_code, non_snake_case)]

extern crate rand_core;

use borsh::{BorshDeserialize, BorshSerialize};

use arrayref::array_ref;
use std::io;
use std::io::{Error, Write};
use std::ops::Deref;
use sha3::Sha3_512;

use curve25519_dalek::{
    ristretto::CompressedRistretto, scalar::Scalar, ristretto::RistrettoPoint, constants::RISTRETTO_BASEPOINT_POINT, constants::RISTRETTO_BASEPOINT_COMPRESSED,
};

// #[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
// pub struct PedersenComm(pub(crate) [u8; 32]);

pub struct CommitmentBase {
    /// Base for the committed value
    pub G: RistrettoPoint,
    /// Base for the blinding factor
    pub H: RistrettoPoint,
}

// this creates a default assignmnent of CommitmentBase base points
impl Default for CommitmentBase {
    fn default() -> Self {
        CommitmentBase {
            G: RISTRETTO_BASEPOINT_POINT,
            H: RistrettoPoint::hash_from_bytes::<Sha3_512>(
                RISTRETTO_BASEPOINT_COMPRESSED.as_bytes(),
            ),
        }
    }
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

// // TODO: implement a struct for the El Gamal commitment (encryption)
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


    // a more detailed version of commitment verification that shows 
    // which of the two commitment components wouldnt verify for a non-verifying opening
    pub fn verify_commitment_detailed(
        comm: &ElGamalComm,
        base: &CommitmentBase,
        open: &Scalar,
        val: &RistrettoPoint,
    ) -> (bool,bool) {
        let CommitmentBase { G, H } = base;
        ((*comm.comm_g == (open * G).compress()),(*comm.comm_h == (val + open * H).compress()))
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



#[cfg(test)]
mod tests_for_ElGamal {
    use super::*;
    use rand_core::OsRng;

    // this test considers the default commitment base, a random val 
    // and a random scalar and first commits to val
    //then it checks whether verify_commitment_detailed() works on it or not
    // the test must verify without any error

    #[test]
    fn elgamal_open_ver_shd_work()-> Result<(), String> {
        let base: CommitmentBase = CommitmentBase::default();    
        let mut prng = OsRng;

        let val: RistrettoPoint = RistrettoPoint::random(&mut prng);
        let open: Scalar = Scalar::random(&mut prng);
        let CommitmentBase { G, H } = base;

        let comm = ElGamalComm {
            comm_g: BorshRistretto((open * G).compress()),
            comm_h: BorshRistretto((val + open * H).compress()),
        };

        let (bool_g, bool_h) = ElGamalComm::verify_commitment_detailed(&comm, &base, &open, &val);
        if (bool_g & bool_h) {
            Ok(())
        } else if (!bool_g & !bool_h){
            Err(String::from("Oops, both open * G and val + open * H are incorrectly computed"))
        } else if (bool_g == false){
            Err(String::from("Oops, open * G is incorrectly computed"))
        } else {
            Err(String::from("Oops, val + open * H is incorrectly computed"))
        }
    }

    // this test considers the default commitment base, a random val and a random scalar and first incorrectly 
    //commits to val by interchaning comm_h and comm_g values
    //then it checks whether verify_commitment_detailed() works on it or not
    // the test shouldnt verify and should output the error "As expected, open * G and val + open * H are incorrectly computed"


    #[test]
    fn elgamal_open_ver_shdnt_work()-> Result<(), String> {
        let base: CommitmentBase = CommitmentBase::default();    
        let mut prng = OsRng;

        let val: RistrettoPoint = RistrettoPoint::random(&mut prng);
        let open: Scalar = Scalar::random(&mut prng);
        let CommitmentBase { G, H } = base;

        let comm = ElGamalComm {
            comm_h: BorshRistretto((open * G).compress()),
            comm_g: BorshRistretto((val + open * H).compress()),
        };

        let (bool_g, bool_h) = ElGamalComm::verify_commitment_detailed(&comm, &base, &open, &val);
        if (bool_g & bool_h) {
            Ok(())
        } else if (!bool_g & !bool_h){
            Err(String::from("As expected, open * G and val + open * H are incorrectly computed"))
        }
        else if (bool_g == false){
            Err(String::from("As expected, open * G is incorrectly computed"))
        } else {
            Err(String::from("As expected, val + open * H is incorrectly computed"))
        } 
    }

}


