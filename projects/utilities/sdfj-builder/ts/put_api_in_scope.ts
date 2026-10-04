import * as api from "./api.ts";
import { libraryMaterials } from "./library.ts";

/** Puts the API on `globalThis`, with `mat` reading `library`. */
export function putApiInScope(library: unknown): void {
  const globals: Pick<typeof globalThis, keyof typeof api | "mat"> = {
    ...api,
    mat: libraryMaterials(library),
  };
  Object.assign(globalThis, globals);
}
