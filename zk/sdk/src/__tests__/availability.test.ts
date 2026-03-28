import { isUsernameAvailable } from "../availability";

// Mock dependencies
jest.mock("../hashUsername", () => ({
  hashUsername: jest.fn(),
}));

jest.mock("../generateProof", () => ({
  generateNonInclusionProof: jest.fn(),
}));

jest.mock("snarkjs", () => ({
  groth16: {
    fullProve: jest.fn(),
    verify: jest.fn(),
  },
}));

import { hashUsername } from "../hashUsername";
import { generateNonInclusionProof } from "../generateProof";
import { groth16 } from "snarkjs";

describe("isUsernameAvailable", () => {
  const mockTree = {
    nodes: {},
    depth: 20,
  };

  const mockRoot = BigInt(
    "1234567890123456789012345678901234567890"
  );

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it("returns true when username is not in the tree (happy path)", async () => {
    (hashUsername as jest.Mock).mockResolvedValue("hashed_username");

    (generateNonInclusionProof as jest.Mock).mockResolvedValue({
      path: [],
      siblings: [],
    });

    (groth16.fullProve as jest.Mock).mockResolvedValue({
      proof: { pi_a: [], pi_b: [], pi_c: [] },
      publicSignals: ["valid"],
    });

    (groth16.verify as jest.Mock).mockResolvedValue(true);

    const result = await isUsernameAvailable(
      "new_user_123",
      mockRoot,
      mockTree as any
    );

    expect(result).toBe(true);
  });

  it("returns false when proof verification fails", async () => {
    (hashUsername as jest.Mock).mockResolvedValue("hashed_username");

    (generateNonInclusionProof as jest.Mock).mockResolvedValue({
      path: [],
      siblings: [],
    });

    (groth16.fullProve as jest.Mock).mockResolvedValue({
      proof: {},
      publicSignals: [],
    });

    (groth16.verify as jest.Mock).mockResolvedValue(false);

    const result = await isUsernameAvailable(
      "existing_user",
      mockRoot,
      mockTree as any
    );

    expect(result).toBe(false);
  });

  it("returns false when an error is thrown", async () => {
    (hashUsername as jest.Mock).mockRejectedValue(
      new Error("hashing failed")
    );

    const result = await isUsernameAvailable(
      "error_user",
      mockRoot,
      mockTree as any
    );

    expect(result).toBe(false);
  });
});
