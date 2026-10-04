import * as api from "./api.ts";
import { type Library, libraryGlobals } from "./library.ts";

/** Puts the API on `globalThis`, with `lib` and `mat` reading `libraries`. */
export function putApiInScope(libraries: readonly Library[]): void {
  const globals: Pick<typeof globalThis, keyof typeof api | "lib" | "mat"> = {
    ...api,
    ...libraryGlobals(libraries),
  };
  Object.assign(globalThis, globals);
}
