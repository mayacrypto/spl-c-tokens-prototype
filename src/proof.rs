#![allow(non_snake_case)]

use curve25519_dalek::{
    ristretto::RistrettoPoint,
    ristretto::CompressedRistretto,
    scalar::Scalar,
    constants::RISTRETTO_BASEPOINT_POINT,
    constants::RISTRETTO_BASEPOINT_COMPRESSED,
};
use sha3::Sha3_512;
use borsh::{BorshSerialize, BorshDeserialize};
// use bulletproofs::RangeProof;
use std::io::{Write, Error};
use std::io;
use arrayref::array_ref;
use std::ops::Deref;


#[derive(BorshDeserialize, BorshSerialize)]
pub struct ProofKnowledge {
    /// Nonce component
    pub nonce: BorshRistretto,
    /// Scalar component
    pub scalar: BorshScalar,
}


/// Struct that holds algorithms related to Pedersen commitments as static functions
/// 
/// This struct is purely for code organization and can be removed as the crypto API evolves
pub struct Pedersen;

/// Base points that are used to generate Pedersen commitments.
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

/// The actual Pedersen commitment
#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct PedersenComm {
    /// Ristretto point representing the commitment
    comm: BorshRistretto,
}
impl PedersenComm {
    pub fn new(comm: BorshRistretto) -> Self {
        Self{ comm }
    }
    pub fn getComm(&self) -> BorshRistretto {
        self.comm
    }
}
impl Pedersen {

    // Ideally, there should be a PedersenComm constructor that samples a random opening and
    // produces a Pedersen commitment. Putting the constructor in the tests for now since it is a
    // randomized function. Ultimately, we should package the crypto component into a separate
    // library that allows for randomized functions.

    /// Given a commitment along with the corresponding base points, opening, and the committed
    /// value, verifies the validity of the commitment.
    pub fn verify_commitment (
        comm: &PedersenComm, // commitment to be verified
        base: &PedersenBase, // base points for the commitment
        open: &Scalar,       // opening for the commitment
        val: &Scalar,        // committed value
    ) -> bool {
        let PedersenBase{ G, H } = base;
        *comm.getComm() == (open * G + val * H).compress()
    }
}

/// Type wrapper of Scalar: to implement the Borsh Serialize/Deserialize traits
/// using the New Type Pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
                    "Scalar deserialize error"
            ))
        };
        *buf = &buf[32..];
        Ok(BorshScalar(scalar.unwrap()))
    }
}

/// Type wrapper for CompressedRistretto: to implement the Borsh
/// Serialize/Deserialize traits using the New Type Pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct BorshRangeProof;

// /// Type wrapper for RangeProof: to implement the Borsh Serialize/Deserialize traits using
// /// the New Type Pattern.
// #[derive(Clone, Debug)]
// pub struct BorshRangeProof(RangeProof);
// impl BorshRangeProof {
//     pub fn new(range_proof: RangeProof) -> Self {
//         Self(range_proof)
//     }
// }
// impl Deref for BorshRangeProof {
//     type Target = RangeProof;
// 
//     fn deref(&self) -> &RangeProof {
//         let Self(range_proof) = self;
//         range_proof
//     }
// }
// impl BorshSerialize for BorshRangeProof {
//     fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
//         let Self(range_proof) = self;
//         let range_proof_bytes = range_proof.to_bytes();
//         writer.write(&range_proof_bytes)?;
//         Ok(())
//     }
// }
// impl BorshDeserialize for BorshRangeProof {
//     fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
//         let range_proof = RangeProof::from_bytes(buf)
//             .or(Err(std::io::ErrorKind::InvalidData))?;
//         Ok( BorshRangeProof(range_proof) )
//     }
// }

// We need ElGamal for anonymity, but not necessarily for confidentiality.
//
// pub struct ElGamal;
// pub struct ElGamalBase {
//     /// Base for the committed value
//     pub G: RistrettoPoint,
//     /// Base for the blinding factor
//     pub H: RistrettoPoint,
// }
// impl Default for ElGamalBase {
//     fn default() -> Self {
//         ElGamalBase {
//             G: RISTRETTO_BASEPOINT_POINT,
//             H: RistrettoPoint::hash_from_bytes::<Sha3_512>(
//                 RISTRETTO_BASEPOINT_COMPRESSED.as_bytes(),
//             ),
//         }
//     }
// }
// pub struct ElGamalComm {
//     /// Randomness component
//     pub R: CompressedRistretto,
//     /// Payload component
//     pub C: CompressedRistretto,
// }
// impl ElGamal {
//     pub fn commit(base: ElGamalBase, val: Scalar) -> (ElGamalComm, Scalar) {
//         let ElGamalBase{ G, H } = base;
//         let mut rng = OsRng;
//         let open = Scalar::random(&mut rng);
// 
//         let R = open*G;
//         let C = R + val*H;
//         let comm = ElGamalComm{ 
//             R: R.compress(), 
//             C: C.compress() 
//         };
//         (comm, open)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    fn commit_pedersen(base: PedersenBase, val: Scalar, open: Option<Scalar>) 
        -> (PedersenComm, Scalar) {
        let PedersenBase{ G, H } = base;

        // If a commitment opening is not specified, sample a random opening
        let open = open.or_else(|| {
            let mut rng = OsRng;
            Some(Scalar::random(&mut rng))
        }).unwrap();

        // Generate the commitment using the opening
        let C = open * G + val * H;

        // Wrap the commitment component into PedersenComm
        let comm = PedersenComm { comm: BorshRistretto(C.compress()) };

        // Return the commitment and the corresponding opening
        (comm, open)
    }

    // TODO: Write tests here

}
