#![allow(non_snake_case)]

use curve25519_dalek::{
    ristretto::RistrettoPoint,
    scalar::Scalar,
};
use rand_core::OsRng;



// TODO: Consider using a commitment trait to unify syntax for ElGamal and Pedersen

pub struct ElGamal;
pub struct ElGamalBase {
    /// Base for the committed value
    pub G: RistrettoPoint,
    /// Base for the blinding factor
    pub H: RistrettoPoint,
}
pub struct ElGamalComm {
    /// Randomness
    pub R: RistrettoPoint,
    /// Payload
    pub C: RistrettoPoint,
}
impl ElGamal {
    pub fn commit(base: ElGamalBase, val: Scalar) -> (ElGamalComm, Scalar) {
        let ElGamalBase{ G, H } = base;
        let mut rng = OsRng;
        let open = Scalar::random(&mut rng);

        let R = open*G;
        let C = R + val*H;
        let comm = ElGamalComm{ R, C };
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
pub struct PedersenComm {
    /// Commitment
    pub C: RistrettoPoint,
}
impl Pedersen {
    pub fn commit(base: PedersenBase, val: Scalar) -> (PedersenComm, Scalar) {
        let PedersenBase{ G, H } = base;
        let mut rng = OsRng;
        let open = Scalar::random(&mut rng);
        let comm = PedersenComm {
            C: open * G + val * H,
        };
        (comm, open)
    }
}

