// Records a model as an `.sdfj` document. The builder runs as
// `main.ts <model> <output>` and reads the material library from the
// `library.json` beside this file.
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import url from "node:url";
import { jsonText } from "./json_text.ts";
import { modelList } from "./model_list.ts";
import { putApiInScope } from "./put_api_in_scope.ts";
import { sdfjDocument } from "./sdfj_document.ts";

const [modelPath, outputPath, ...extra] = process.argv.slice(2);
if (modelPath === undefined || outputPath === undefined || extra.length > 0) {
  throw new Error("the builder takes a model path and an output path");
}
const libraryUrl = new URL("library.json", import.meta.url);
putApiInScope(JSON.parse(fs.readFileSync(libraryUrl, "utf8")));
const model = await import(url.pathToFileURL(path.resolve(modelPath)).href);
const document = sdfjDocument(path.parse(modelPath).name, modelList(model));
fs.writeFileSync(outputPath, `${jsonText(document)}\n`);
