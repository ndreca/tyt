import { type Part, stepsAndParts } from "./part.ts";
import type { Step } from "./step.ts";

/** The steps and parts the model module `model` exports by default. */
export function modelList(
  model: Readonly<Record<string, unknown>>,
): (Step | Part)[] {
  if (!("default" in model)) {
    throw new Error("the model has no default export");
  }
  return stepsAndParts("the default export", model.default);
}
