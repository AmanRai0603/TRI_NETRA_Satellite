#!/usr/bin/env python3
"""Decode (and check) an adcs-fswcfg/1 blob with the layout in fsw/params/params.toml.

  python3 tools/fswcfg.py blob.fswcfg            -> JSON of every field on stdout
The same layout the C decoder (fsw/src/adcs_params.c) and the Rust decoder
(fsw-rs/src/params.rs) are generated from. Copyright (c) 2026 Agastya.
"""
import json, pathlib, struct, sys, tomllib, zlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = tomllib.loads((ROOT / "fsw" / "params" / "params.toml").read_text())
FMT = {"f64": ("<d", 8), "u32": ("<I", 4), "u8": ("<B", 1)}


def decode(blob: bytes) -> dict:
    if blob[:8] != b"ADCSCFG1":
        raise ValueError("not an adcs-fswcfg/1 blob")
    n = struct.unpack_from("<I", blob, 8)[0]
    pay = blob[12:12 + n]
    if zlib.crc32(pay) != struct.unpack_from("<I", blob, 12 + n)[0]:
        raise ValueError("CRC-32 mismatch")
    i, out = 0, {}
    for f in SPEC["field"]:
        fmt, sz = FMT[f["type"]]
        shape = f.get("shape", [])
        cnt = 1
        for d in shape:
            cnt *= d
        vals = []
        for _ in range(cnt):
            vals.append(struct.unpack_from(fmt, pay, i)[0]); i += sz
        if not shape:
            out[f["name"]] = vals[0]
        elif len(shape) == 1:
            out[f["name"]] = vals
        else:
            out[f["name"]] = [vals[r * shape[1]:(r + 1) * shape[1]] for r in range(shape[0])]
    if i != n:
        raise ValueError(f"layout length {i} != payload {n}")
    return out


if __name__ == "__main__":
    print(json.dumps(decode(pathlib.Path(sys.argv[1]).read_bytes()), indent=1))
