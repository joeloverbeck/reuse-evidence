import test from "node:test";
import assert from "node:assert/strict";

const adapters = await Promise.all(
  Array.from({ length: 9 }, (_, index) =>
    import(`../src/adapter${String(index + 1).padStart(2, "0")}.js`),
  ),
);

test("all adapters trim and lowercase", () => {
  for (const adapter of adapters) {
    assert.equal(adapter.normalizeTag("  MiXeD  "), "mixed");
  }
});
