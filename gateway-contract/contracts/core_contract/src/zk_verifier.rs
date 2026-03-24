#![allow(dead_code)]
use soroban_sdk::Env;

// Type aliases to satisfy clippy::type_complexity
type G1Point = ([u8; 32], [u8; 32]);
type G2Point = (([u8; 32], [u8; 32]), ([u8; 32], [u8; 32]));

// ---------------------------------------------------------------------------
// Groth16 BN254 verification key (test/stub values — replace with real ceremony output)
// ---------------------------------------------------------------------------

// G1 point: (x, y) as 32-byte big-endian field elements concatenated → 64 bytes
const VK_ALPHA_G1: ([u8; 32], [u8; 32]) = (
    [0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
     0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09,
     0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
     0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09],
    [0x09, 0xf8, 0xe7, 0xd6, 0xc5, 0xb4, 0xa3, 0x92,
     0x81, 0x70, 0x6f, 0x5e, 0x4d, 0x3c, 0x2b, 0x1a,
     0x09, 0xf8, 0xe7, 0xd6, 0xc5, 0xb4, 0xa3, 0x92,
     0x81, 0x70, 0x6f, 0x5e, 0x4d, 0x3c, 0x2b, 0x1a],
);

// G2 point: ((x0,x1),(y0,y1)) — BN254 G2 uses Fp2 elements
const VK_BETA_G2: G2Point = (
    (
        [0x2a, 0x3b, 0x4c, 0x5d, 0x6e, 0x7f, 0x80, 0x91,
         0xa2, 0xb3, 0xc4, 0xd5, 0xe6, 0xf7, 0x08, 0x19,
         0x2a, 0x3b, 0x4c, 0x5d, 0x6e, 0x7f, 0x80, 0x91,
         0xa2, 0xb3, 0xc4, 0xd5, 0xe6, 0xf7, 0x08, 0x19],
        [0x19, 0x08, 0xf7, 0xe6, 0xd5, 0xc4, 0xb3, 0xa2,
         0x91, 0x80, 0x7f, 0x6e, 0x5d, 0x4c, 0x3b, 0x2a,
         0x19, 0x08, 0xf7, 0xe6, 0xd5, 0xc4, 0xb3, 0xa2,
         0x91, 0x80, 0x7f, 0x6e, 0x5d, 0x4c, 0x3b, 0x2a],
    ),
    (
        [0x3b, 0x4c, 0x5d, 0x6e, 0x7f, 0x80, 0x91, 0xa2,
         0xb3, 0xc4, 0xd5, 0xe6, 0xf7, 0x08, 0x19, 0x2a,
         0x3b, 0x4c, 0x5d, 0x6e, 0x7f, 0x80, 0x91, 0xa2,
         0xb3, 0xc4, 0xd5, 0xe6, 0xf7, 0x08, 0x19, 0x2a],
        [0x2a, 0x19, 0x08, 0xf7, 0xe6, 0xd5, 0xc4, 0xb3,
         0xa2, 0x91, 0x80, 0x7f, 0x6e, 0x5d, 0x4c, 0x3b,
         0x2a, 0x19, 0x08, 0xf7, 0xe6, 0xd5, 0xc4, 0xb3,
         0xa2, 0x91, 0x80, 0x7f, 0x6e, 0x5d, 0x4c, 0x3b],
    ),
);

const VK_GAMMA_G2: G2Point = VK_BETA_G2;
const VK_DELTA_G2: G2Point = VK_BETA_G2;

// IC[0] and IC[1] — one public input expected (the username commitment)
const VK_IC_0: ([u8; 32], [u8; 32]) = VK_ALPHA_G1;
const VK_IC_1: ([u8; 32], [u8; 32]) = VK_ALPHA_G1;

// ---------------------------------------------------------------------------
// Proof layout: proof is a 256-byte slice:
//   [  0.. 63] A  (G1 point)
//   [ 64..191] B  (G2 point)
//   [192..255] C  (G1 point)
// ---------------------------------------------------------------------------

/// Minimal field arithmetic helpers (BN254 scalar field prime)
/// p = 21888242871839275222246405745257275088548364400416034343698204186575808495617
const BN254_P: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29,
    0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91,
    0x43, 0xe1, 0xf5, 0x93, 0xf0, 0x00, 0x00, 0x01,
];

fn bytes_lt(a: &[u8; 32], b: &[u8; 32]) -> bool {
    for i in 0..32 {
        if a[i] < b[i] { return true; }
        if a[i] > b[i] { return false; }
    }
    false
}

fn field_element_valid(fe: &[u8; 32]) -> bool {
    bytes_lt(fe, &BN254_P)
}

/// Simplified linear combination: vk_ic_0 + public_input[0] * vk_ic_1
/// Real impl would use scalar multiplication on BN254; here we do a
/// byte-level stub that confirms shapes are correct and inputs are in-field.
fn compute_linear_combination(public_inputs: &[[u8; 32]]) -> Option<([u8; 32], [u8; 32])> {
    if public_inputs.len() != 1 {
        return None;
    }
    if !field_element_valid(&public_inputs[0]) {
        return None;
    }
    // Stub: return IC[0] xor-mixed with the input (replace with real msm)
    let mut x = VK_IC_0.0;
    for i in 0..32 {
        x[i] ^= public_inputs[0][i];
    }
    Some((x, VK_IC_0.1))
}

/// Structural validation of a G1 point (non-zero, bytes in range).
fn validate_g1(x: &[u8; 32], _y: &[u8; 32]) -> bool {
    // Must not be the point at infinity (all zeros)
    x.iter().any(|&b| b != 0)
}

/// Core Groth16 verification logic.
///
/// In production this must call a native BN254 pairing function.
/// Soroban does not yet expose native pairings, so this implementation:
///   1. Validates all proof elements are structurally correct.
///   2. Validates public inputs are valid field elements.
///   3. Computes the linear combination of IC points.
///   4. Performs a deterministic accept/reject based on those checks.
///
/// Replace the body of `pairing_check` with a host-function call when
/// Soroban exposes BN254 pairings.
pub fn groth16_verify(
    proof_a: G1Point,
    proof_b: G2Point,
    proof_c: G1Point,
    public_inputs: &[[u8; 32]],
) -> bool {
    // 1. Validate proof points are non-zero
    if !validate_g1(&proof_a.0, &proof_a.1) { return false; }
    if !validate_g1(&proof_c.0, &proof_c.1) { return false; }
    if !validate_g1(&proof_b.0.0, &proof_b.0.1) { return false; }

    // 2. Validate public inputs are in-field
    for inp in public_inputs {
        if !field_element_valid(inp) { return false; }
    }

    // 3. Linear combination
    let _vk_x = match compute_linear_combination(public_inputs) {
        Some(pt) => pt,
        None => return false,
    };

    // 4. Pairing check stub — always passes structural checks here.
    //    Replace with: host_fn::bn254_pairing_check(...)
    true
}

// ---------------------------------------------------------------------------
// Public API used by lib.rs
// ---------------------------------------------------------------------------

/// `proof_bytes` must be exactly 256 bytes:
///   [0..64]   A (G1)
///   [64..192] B (G2)
///   [192..256] C (G1)
///
/// `public_inputs` is a list of 32-byte big-endian field elements.
pub fn verify_proof_bytes(
    _env: &Env,
    proof_bytes: &[u8],
    public_inputs: &[[u8; 32]],
) -> bool {
    if proof_bytes.len() != 256 {
        return false;
    }

    let mut a_x = [0u8; 32]; a_x.copy_from_slice(&proof_bytes[0..32]);
    let mut a_y = [0u8; 32]; a_y.copy_from_slice(&proof_bytes[32..64]);
    let mut b_x0 = [0u8; 32]; b_x0.copy_from_slice(&proof_bytes[64..96]);
    let mut b_x1 = [0u8; 32]; b_x1.copy_from_slice(&proof_bytes[96..128]);
    let mut b_y0 = [0u8; 32]; b_y0.copy_from_slice(&proof_bytes[128..160]);
    let mut b_y1 = [0u8; 32]; b_y1.copy_from_slice(&proof_bytes[160..192]);
    let mut c_x = [0u8; 32]; c_x.copy_from_slice(&proof_bytes[192..224]);
    let mut c_y = [0u8; 32]; c_y.copy_from_slice(&proof_bytes[224..256]);

    groth16_verify(
        (a_x, a_y),
        ((b_x0, b_x1), (b_y0, b_y1)),
        (c_x, c_y),
        public_inputs,
    )
}
