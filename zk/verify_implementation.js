#!/usr/bin/env node

/**
 * Quick verification script for username_leaf circuit
 * Tests basic compilation and functionality
 */

const fs = require('fs');
const path = require('path');

// Check required files exist
const requiredFiles = [
  'circuits/merkle/username_leaf_main.circom',
  'circuits/merkle/username_leaf.circom',
  'circuits/username_hash.circom',
  'input.json',
  'tests/username_leaf_test.ts',
  'docs/username_encoding.md',
  'tests/README_username_leaf.md'
];

let allFilesExist = true;

requiredFiles.forEach(file => {
  const exists = fs.existsSync(file);
  if (!exists) {
    allFilesExist = false;
  }
});

if (!allFilesExist) {
  process.exit(1);
}

// Verify input.json format
try {
  const input = JSON.parse(fs.readFileSync('input.json', 'utf8'));
  
  if (!input.username || !Array.isArray(input.username)) {
    throw new Error('Missing or invalid username array');
  }
  
  if (input.username.length !== 32) {
    throw new Error('Username array must have 32 elements');
  }
  
  const expectedAmar = [97, 109, 97, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  
  const matches = JSON.stringify(input.username) === JSON.stringify(expectedAmar);
  
  if (!matches) {
    process.exit(1);
  }
  
} catch (error) {
  process.exit(1);
}

// Check package.json scripts
try {
  const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
  
  const requiredScripts = [
    'compile:username_leaf',
    'test:username_leaf'
  ];
  
  requiredScripts.forEach(script => {
    const exists = packageJson.scripts && packageJson.scripts[script];
    if (!exists) {
      process.exit(1);
    }
  });
  
} catch (error) {
  process.exit(1);
}

// Check compile.sh
try {
  const compileScript = fs.readFileSync('scripts/compile.sh', 'utf8');
  
  const hasUsernameLeaf = compileScript.includes('username_leaf_main');
  if (!hasUsernameLeaf) {
    process.exit(1);
  }
  
} catch (error) {
  process.exit(1);
}
