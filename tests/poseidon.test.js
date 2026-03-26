const assert = require('assert');
const { hashUsername, bigintToBytes32 } = require('../src/poseidon');

describe('hashUsername', function () {
  it('should output a 32-byte hex string for "amar"', async function () {
    const hash = await hashUsername('amar');
    const hex = bigintToBytes32(hash);
    // ...existing code...
    assert.strictEqual(typeof hex, 'string');
    assert(hex.startsWith('0x'));
    assert(hex.length === 66);
  });
});
