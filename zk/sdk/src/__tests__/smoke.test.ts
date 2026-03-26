import * as sdk from "../index";

declare const describe: (name: string, fn: () => void) => void;
declare const it: (name: string, fn: () => void) => void;
declare const expect: (value: unknown) => { toBeDefined: () => void };

describe("zk sdk smoke", () => {
  it("imports index without throwing", () => {
    expect(sdk).toBeDefined();
  });
});
