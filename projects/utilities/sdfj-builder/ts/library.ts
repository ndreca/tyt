import * as check from "./check.ts";
import { Material } from "./material.ts";

/**
 * The `mat` record over `library`, which maps each name to properties in the
 * sdfj format's form. Reading a name `library` lacks errors.
 */
export function libraryMaterials(
  library: unknown,
): Readonly<Record<string, Material>> {
  if (!check.isPlainObject(library)) {
    check.fail("the library", "an object", library);
  }
  const materials: Record<string, Material> = Object.create(null);
  for (const [name, properties] of Object.entries(library)) {
    if (!check.isPlainObject(properties)) {
      check.fail(`the library's ${name}`, "an object", properties);
    }
    materials[name] = new Material({ kind: "material", properties });
  }
  return new Proxy(Object.freeze(materials), {
    get(target, key) {
      if (typeof key === "string" && !Object.hasOwn(target, key)) {
        throw new Error(`mat has no material ${JSON.stringify(key)}`);
      }
      return Reflect.get(target, key);
    },
  });
}
