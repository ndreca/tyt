import * as check from "./check.ts";
import { type Entry, entry } from "./entry.ts";
import { Material } from "./material.ts";
import { Part } from "./part.ts";
import { Pattern } from "./pattern.ts";
import { Shades } from "./shades.ts";
import { Shape2d } from "./shape2d.ts";
import { Shape3d } from "./shape3d.ts";
import { Step } from "./step.ts";
import type { Vec3 } from "./types.ts";

/** The sdfj format version the builder writes. */
const SDFJ_VERSION = 1;

/** An `.sdfj` document as JSON values. */
export interface SdfjDocument {
  /** The format version. */
  version: number;

  /** The `Shape3d` entries. */
  shapes3d: Entry[];

  /** The `Shape2d` entries. */
  shapes2d: Entry[];

  /** The `Material` entries. */
  materials: Entry[];

  /** The `shades` calls. */
  shades: Entry[];

  /** The `Pattern` entries. */
  patterns: Entry[];

  /** The `Step` entries. */
  steps: Entry[];

  /** An object per part whose list holds steps. */
  objects: Entry[];

  /** A node per part. */
  nodes: Entry[];

  /** The nodes at the top of the hierarchy. */
  rootNodes: number[];
}

/**
 * The document of the model `name` whose default export is `list`. A value met
 * again reuses its entry.
 */
export function sdfjDocument(
  name: string,
  list: readonly (Step | Part)[],
): SdfjDocument {
  const writer = new SdfjWriter();
  writer.document.rootNodes.push(writer.node(name, undefined, undefined, list));
  return writer.document;
}

/** A value with an entry in one of the document's tables. */
type TableValue = Material | Pattern | Shades | Shape2d | Shape3d | Step;

/** The document so far and the index each written value took. */
class SdfjWriter {
  readonly document: SdfjDocument = {
    version: SDFJ_VERSION,
    shapes3d: [],
    shapes2d: [],
    materials: [],
    shades: [],
    patterns: [],
    steps: [],
    objects: [],
    nodes: [],
    rootNodes: [],
  };

  readonly #indices = new Map<TableValue | Part, number>();

  /** Writes a node, and an object when `list` holds steps. */
  node(
    name: string,
    pivot: Vec3 | undefined,
    offset: Vec3 | undefined,
    list: readonly (Step | Part)[],
  ): number {
    const steps: number[] = [];
    const childNodes: number[] = [];
    for (const item of list) {
      if (item instanceof Step) {
        steps.push(this.#index(item));
      } else {
        childNodes.push(this.#part(item));
      }
    }
    const childObjects = steps.length === 0
      ? []
      : [this.document.objects.push({ name, steps }) - 1];
    const node = entry({ name, pivot, offset, childObjects, childNodes });
    return this.document.nodes.push(node) - 1;
  }

  /** The index of the node `part` writes. */
  #part(part: Part): number {
    let index = this.#indices.get(part);
    if (index === undefined) {
      const { name, pivot, offset, list } = Part.entry(part);
      index = this.node(name, pivot, offset, list);
      this.#indices.set(part, index);
    }
    return index;
  }

  /** The index of the entry `value` writes, after the entries it references. */
  #index(value: TableValue): number {
    let index = this.#indices.get(value);
    if (index === undefined) {
      const [table, fields] = this.#tableEntry(value);
      index = table.push(this.#write(fields) as Entry) - 1;
      this.#indices.set(value, index);
    }
    return index;
  }

  /** `value` with an index in place of each value it holds. */
  #write(value: unknown): unknown {
    if (
      value instanceof Material || value instanceof Pattern ||
      value instanceof Shades || value instanceof Shape2d ||
      value instanceof Shape3d || value instanceof Step
    ) {
      return this.#index(value);
    }
    if (Array.isArray(value)) {
      return value.map((item) => this.#write(item));
    }
    if (check.isPlainObject(value)) {
      return Object.fromEntries(
        Object.entries(value).map(([key, item]) => [key, this.#write(item)]),
      );
    }
    return value;
  }

  /** The table `value` writes into and the entry it writes. */
  #tableEntry(value: TableValue): [Entry[], Entry] {
    if (value instanceof Material) {
      return [this.document.materials, Material.entry(value)];
    }
    if (value instanceof Pattern) {
      return [this.document.patterns, Pattern.entry(value)];
    }
    if (value instanceof Shades) {
      return [this.document.shades, Shades.entry(value)];
    }
    if (value instanceof Shape2d) {
      return [this.document.shapes2d, Shape2d.entry(value)];
    }
    if (value instanceof Shape3d) {
      return [this.document.shapes3d, Shape3d.entry(value)];
    }
    return [this.document.steps, Step.entry(value)];
  }
}
