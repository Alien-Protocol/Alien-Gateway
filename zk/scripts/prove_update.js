const { buildPoseidon } = require("circomlibjs");
const fs = require("fs");

function buildTree(poseidon, F, leaves) {
  let level = leaves;
  const tree = [level];

  while (level.length > 1) {
    const next = [];
    for (let i = 0; i < level.length; i += 2) {
      next.push(poseidon([level[i], level[i + 1]]));
    }
    tree.push(next);
    level = next;
  }

  return tree;
}

(async () => {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;

  // Depth 2 tree → 4 leaves
  let leaves = [
    F.e(0),
    F.e(0),
    F.e(0),
    F.e(0)
  ];

  const oldTree = buildTree(poseidon, F, leaves);
  const oldRoot = oldTree[oldTree.length - 1][0];

  console.log("Old Root:", F.toString(oldRoot));

  // Insert new leaf at position 0
  const insertionIndex = 0;
  const newLeaf = F.e(12345);

  let index = insertionIndex;
  const pathElements = [];
  const pathIndices = [];

  for (let level = 0; level < oldTree.length - 1; level++) {
    const siblingIndex = index ^ 1;
    pathElements.push(F.toString(oldTree[level][siblingIndex]));
    pathIndices.push(index % 2);
    index = Math.floor(index / 2);
  }

  // Apply insertion
  leaves[insertionIndex] = newLeaf;

  const newTree = buildTree(poseidon, F, leaves);
  const newRoot = newTree[newTree.length - 1][0];

  console.log("New Root:", F.toString(newRoot));

  const input = {
    oldLeaf: "0",
    newLeaf: F.toString(newLeaf),
    pathElements,
    pathIndices,
    oldRoot: F.toString(oldRoot),
    newRoot: F.toString(newRoot)
  };

  fs.writeFileSync(
    "build/update_input.json",
    JSON.stringify(input, null, 2)
  );

  console.log("Generated build/update_input.json");
})();