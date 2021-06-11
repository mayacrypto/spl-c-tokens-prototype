use borsh::{BorshDeserialize, BorshSerialize};


#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PedersenComm(pub(crate) [u8; 32]);

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProofKnowledge {
    pub nonce: [u8; 32],
    pub scalar: [u8; 32],
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RangeProof;
