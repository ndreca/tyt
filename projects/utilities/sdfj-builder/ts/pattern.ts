import * as check from "./check.ts";
import { type Entry, entry } from "./entry.ts";
import { Material } from "./material.ts";
import { shades } from "./shades.ts";
import type { Axis } from "./types.ts";

/** A pick of one material per cell. */
export class Pattern {
  readonly #entry: Entry;

  /** The pattern whose `patterns` entry holds `fields`. */
  constructor(fields: Readonly<Record<string, unknown>>) {
    this.#entry = entry(fields);
    Object.freeze(this);
  }

  /** The `patterns` entry `pattern` writes. */
  static entry(pattern: Pattern): Entry {
    return pattern.#entry;
  }
}

/** Slabs `period` thick across `axis` that cycle through `materials`. */
export function bands(
  materials: Material[],
  options: { axis: Axis; period?: number; warp?: number; seed?: number },
): Pattern {
  const option = check.knownOptions("bands", options, [
    "axis",
    "period",
    "warp",
    "seed",
  ]);
  return new Pattern({
    kind: "bands",
    materials: materialList("bands materials", materials),
    axis: check.axis("bands axis", option.axis),
    period: check.optional("bands period", option.period, check.finite),
    warp: check.optional("bands warp", option.warp, check.finite),
    seed: check.optional("bands seed", option.seed, check.finite),
  });
}

/**
 * Rings around `axis` that each pick a random material. A single `base`
 * stands for `shades(base)`.
 */
export function grain(
  base: Material | Material[],
  options: { axis: Axis; period?: number; warp?: number; seed: number },
): Pattern {
  const option = check.knownOptions("grain", options, [
    "axis",
    "period",
    "warp",
    "seed",
  ]);
  return new Pattern({
    kind: "grain",
    materials: Array.isArray(base)
      ? materialList("grain base", base)
      : shades(check.instance("grain base", base, Material)),
    axis: check.axis("grain axis", option.axis),
    period: check.optional("grain period", option.period, check.finite),
    warp: check.optional("grain warp", option.warp, check.finite),
    seed: check.finite("grain seed", option.seed),
  });
}

/** Spans from `from` to `to` along `axis` that take `materials` in order. */
export function gradient(
  materials: Material[],
  options: {
    axis: Axis;
    from: number;
    to: number;
    warp?: number;
    seed?: number;
  },
): Pattern {
  const option = check.knownOptions("gradient", options, [
    "axis",
    "from",
    "to",
    "warp",
    "seed",
  ]);
  return new Pattern({
    kind: "gradient",
    materials: materialList("gradient materials", materials),
    axis: check.axis("gradient axis", option.axis),
    from: check.finite("gradient from", option.from),
    to: check.finite("gradient to", option.to),
    warp: check.optional("gradient warp", option.warp, check.finite),
    seed: check.optional("gradient seed", option.seed, check.finite),
  });
}

/** Fractal noise split into ranges that take `materials` in order. */
export function noise(
  materials: Material[],
  options: { scale: number; octaves?: number; seed: number },
): Pattern {
  const option = check.knownOptions("noise", options, [
    "scale",
    "octaves",
    "seed",
  ]);
  return new Pattern({
    kind: "noise",
    materials: materialList("noise materials", materials),
    scale: check.finite("noise scale", option.scale),
    octaves: check.optional("noise octaves", option.octaves, check.finite),
    seed: check.finite("noise seed", option.seed),
  });
}

/** Irregular cells about `size` across that each pick a random material. */
export function cells(
  materials: Material[],
  options: { size: number; seed: number; border?: Material },
): Pattern {
  const option = check.knownOptions("cells", options, [
    "size",
    "seed",
    "border",
  ]);
  return new Pattern({
    kind: "cells",
    materials: materialList("cells materials", materials),
    size: check.finite("cells size", option.size),
    seed: check.finite("cells seed", option.seed),
    border: check.optional(
      "cells border",
      option.border,
      (path, value) => check.instance(path, value, Material),
    ),
  });
}

/** `base` with a `density` chance per grid cell of a random accent. */
export function speckle(
  base: Material,
  accents: Material | Material[],
  options: { density: number; seed: number },
): Pattern {
  const option = check.knownOptions("speckle", options, ["density", "seed"]);
  return new Pattern({
    kind: "speckle",
    base: check.instance("speckle base", base, Material),
    accents: Array.isArray(accents)
      ? materialList("speckle accents", accents)
      : [check.instance("speckle accents", accents, Material)],
    density: check.finite("speckle density", option.density),
    seed: check.finite("speckle seed", option.seed),
  });
}

/** `materials` alternating over cubes `size` wide. */
export function checker(
  materials: Material[],
  options?: { size?: number },
): Pattern {
  const option = check.knownOptions("checker", options, ["size"]);
  return new Pattern({
    kind: "checker",
    materials: materialList("checker materials", materials),
    size: check.optional("checker size", option.size, check.finite),
  });
}

/** A copy of the material array `value`. */
function materialList(path: string, value: unknown): Material[] {
  return check.list(
    path,
    value,
    "an array of Materials",
    (itemPath, item) => check.instance(itemPath, item, Material),
  );
}
