// tncontent_cli.mjs -- the page's content hash of a design file, from the command line, for the
// cross-language test: node tncontent_cli.mjs FILE  ->  the hex hash.
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { contentHash } from "./tncontent.js";

const require = createRequire(import.meta.url);
const here = new URL(".", import.meta.url).pathname;
const SQL = await require(here + "../vendor/sqljs/sql-wasm.js")({ wasmBinary: readFileSync(here + "../vendor/sqljs/sql-wasm.wasm") });
const db = new SQL.Database(readFileSync(process.argv[2]));
const kind = db.exec(`SELECT "value" FROM meta WHERE "key" = 'format'`)[0].values[0][0];
console.log(await contentHash(db, kind));
