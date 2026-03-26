import { verifyProof } from '../scripts/verify_proof';
import fs from 'fs';
import path from 'path';
import assert from 'assert';

async function testVerifier() {
  // Test non_inclusion proof (valid)
  const proofPath = path.join(__dirname, '../tests/fixtures/non_inclusion_proof.json');
  const signalsPath = path.join(__dirname, '../tests/fixtures/non_inclusion_signals.json');
  if (fs.existsSync(proofPath) && fs.existsSync(signalsPath)) {
    const proof = JSON.parse(fs.readFileSync(proofPath, 'utf-8'));
    const signals = JSON.parse(fs.readFileSync(signalsPath, 'utf-8'));
    const result = await verifyProof('non_inclusion', proof, signals);
    assert(result.valid, 'Valid non-inclusion proof should pass');
  } else {
    console.warn('Non-inclusion proof fixture missing, skipping test.');
  }

  // Test update proof (valid)
  const updateProofPath = path.join(__dirname, '../tests/fixtures/update_proof.json');
  const updateSignalsPath = path.join(__dirname, '../tests/fixtures/update_signals.json');
  if (fs.existsSync(updateProofPath) && fs.existsSync(updateSignalsPath)) {
    const proof = JSON.parse(fs.readFileSync(updateProofPath, 'utf-8'));
    const signals = JSON.parse(fs.readFileSync(updateSignalsPath, 'utf-8'));
    const result = await verifyProof('update', proof, signals);
    assert(result.valid, 'Valid update proof should pass');
  } else {
    console.warn('Update proof fixture missing, skipping test.');
  }

  // Test tampered proof (should fail)
  if (fs.existsSync(proofPath) && fs.existsSync(signalsPath)) {
    const proof = JSON.parse(fs.readFileSync(proofPath, 'utf-8'));
    const signals = JSON.parse(fs.readFileSync(signalsPath, 'utf-8'));
    // Tamper with the proof
    proof.pi_a[0] = '0';
    const result = await verifyProof('non_inclusion', proof, signals);
    assert(!result.valid, 'Tampered proof should fail');
  }
}

// To generate proof fixtures for verifier tests, add after proof generation:
// require('fs').writeFileSync('tests/fixtures/non_inclusion_proof.json', JSON.stringify(proof, null, 2));
// require('fs').writeFileSync('tests/fixtures/non_inclusion_signals.json', JSON.stringify(publicSignals, null, 2));
// Do the same for update proofs in test_update_proof.js.
// Then run: node tests/test_non_inclusion_proof.js and node tests/test_update_proof.js

testVerifier().catch(err => {
  console.error('Verifier test failed:', err);
  process.exit(1);
});
