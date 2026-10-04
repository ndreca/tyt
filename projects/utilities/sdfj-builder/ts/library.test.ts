import assert from "node:assert/strict";
import { libraryMaterials } from "./library.ts";
import { add } from "./step.ts";
import { ball, document } from "./test/tables.ts";

const LIBRARY = { oak: { baseColor: "#8A5A2B", roughness: 0.8 } };

Deno.test("mat writes the library's properties once per name", () => {
  const mat = libraryMaterials(LIBRARY);
  const { materials } = document([
    add("seat", ball, mat.oak),
    add("back", ball, mat.oak),
  ]);
  assert.deepStrictEqual(materials, [
    { kind: "material", properties: { baseColor: "#8A5A2B", roughness: 0.8 } },
  ]);
});

Deno.test("mat reads only the library's names", () => {
  const mat = libraryMaterials(LIBRARY);
  assert.throws(() => mat.pine, { message: 'mat has no material "pine"' });
});

Deno.test("the library maps names to properties", () => {
  assert.throws(() => libraryMaterials({ oak: "#8A5A2B" }), {
    message: `the library's oak must be an object, not "#8A5A2B"`,
  });
});
