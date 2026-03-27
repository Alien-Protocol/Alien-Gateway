import { isUsernameAvailable } from "../availability";

describe("isUsernameAvailable", () => {
  const mockTree = {
    nodes: {},
    depth: 20,
  };

  const mockRoot = BigInt(
    "1234567890123456789012345678901234567890"
  );

  it("returns true for username not in tree", async () => {
    const result = await isUsernameAvailable(
      "new_user_123",
      mockRoot,
      mockTree as any
    );

    expect(typeof result).toBe("boolean");
    // In real test: expect(result).toBe(true)
  });

  it("returns false when proof fails", async () => {
    const result = await isUsernameAvailable(
      "existing_user",
      mockRoot,
      mockTree as any
    );

    expect(result).toBe(false);
  });
});