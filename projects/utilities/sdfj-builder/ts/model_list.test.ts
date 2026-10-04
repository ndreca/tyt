import assert from "node:assert/strict";
import { modelList } from "./model_list.ts";
import { add } from "./step.ts";
import { ball, white } from "./test/tables.ts";

Deno.test("the default export lists the model's steps and parts", () => {
  const step = add("ball", ball, white);
  assert.deepStrictEqual(modelList({ default: [step] }), [step]);
});

Deno.test("a default export the builder cannot write errors", () => {
  assert.throws(() => modelList({}), {
    message: "the model has no default export",
  });
  assert.throws(() => modelList({ default: add("ball", ball, white) }), {
    message:
      "the default export must be an array of Steps and Parts, not a Step",
  });
  assert.throws(() => modelList({ default: [ball] }), {
    message: "the default export[0] must be a Step or a Part, not a Shape3d",
  });
});
