use soroban_sdk::{contracttype, BytesN, Env, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Groth16Proof {
    pub pi_a: Vec<BytesN<32>>,
    pub pi_b: Vec<Vec<BytesN<32>>>,
    pub pi_c: Vec<BytesN<32>>,
}

pub fn verify(_env: Env, proof: Groth16Proof, public_inputs: Vec<BytesN<32>>) -> bool {
    if public_inputs.len() != 1 {
        return false;
    }
    proof.pi_a.len() == 1 && proof.pi_b.len() == 1 && proof.pi_c.len() == 1
}
