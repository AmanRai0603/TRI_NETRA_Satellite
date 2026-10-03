// integrity.mjs -- design/js/structure.js integrity() over a design folder on disk, as JSON:
// tests/test_structure.py holds it and tools/group.py check to the same answers.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { NodeFolder } from "./node_folder.mjs";

const require = createRequire(import.meta.url);
const ROOT = new URL("../../", import.meta.url).pathname;
const SQL = await require(ROOT + "design/vendor/sqljs/sql-wasm.js")({ wasmBinary: readFileSync(ROOT + "design/vendor/sqljs/sql-wasm.wasm") });
const S = await import(ROOT + "design/js/structure.js");
process.stdout.write(JSON.stringify(await S.integrity(SQL, new NodeFolder(process.argv[2]))));
