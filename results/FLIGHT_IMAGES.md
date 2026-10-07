# The sealed flight images

Owner: Agastya. Written by `python3 tools/flight_build.py seal` (docs/PLAN_2_0.md S6). Each image is one target's
build of the flight software written from the design: the generated sources, the binary and the configuration
blobs of every scenario, each with its sha256, and the checks made on it. The `.tnfsw` files sit in
`results/flight_images/` and are not kept in git (the Rust host library alone is 8 MB); `index.json` is.
`python3 tools/flight_build.py verify FILE` rechecks one; `which RUN` names the image a result flew (its manifest's
fsw.build_id is the image's runtime version). They are the developer's builds, not signed.

| target | design | runtime version (build id) | image sha256 | files | checks |
|---|---|---|---|---:|---|
| posix | today 2026-10-06 (`b4dbd114fc09`) | `trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg d0b5944afc9464e4` | `b0728c67695acffd` | 75 | 5 passed |
| posix-rs | today 2026-10-06 (`b4dbd114fc09`) | `trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg d0b5944afc9464e4` | `f9e9cc098337e1cb` | 75 | 4 passed |
| qemu | today 2026-10-06 (`b4dbd114fc09`) | `trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg d0b5944afc9464e4` | `2bc9fce1cb799136` | 75 | 6 passed |
| qemu-rs | today 2026-10-06 (`b4dbd114fc09`) | `trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg d0b5944afc9464e4` | `e57fe323ddfdd7f0` | 75 | 4 passed; not run: stack |

## Checks

**posix** (design-today-2026-10-06.posix.tnfsw, built 2026-10-07T08:26:49Z with gcc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0; GNU ar (GNU Binutils for Ubuntu) 2.42):
- passed: flight_build gen --check: every generated source is what the design gives (algorithms d0b5944afc9464e4)
- passed: from_design: the flight algorithm blocks: fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives
- passed: C flight flags: every warning an error, no malloc/time/rand (make check)
- passed: vectors: pcode: 78 functions (10 by name through the dispatcher), 2616 calls, 45657 values, 45597 bit for bit, worst relative difference 6.6e-15; 0 failure(s); on the host build of these sources (fsw/build/test_pcode, the library posix seals)
- passed: build id: the image carries 'trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg d0b5944afc9464e4'

**posix-rs** (design-today-2026-10-06.posix-rs.tnfsw, built 2026-10-07T08:26:51Z with rustc 1.94.1 (e408947bf 2026-03-25); cargo 1.94.1 (29ea6fb6a 2026-03-24)):
- passed: flight_build gen --check: every generated source is what the design gives (algorithms d0b5944afc9464e4)
- passed: from_design: the flight algorithm blocks: fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives
- passed: vectors: 78 functions (24 by name through the dispatcher), 2616 calls, 45657 values, 45597 bit for bit, worst relative difference 6.592861854817441e-15; on the host build of these sources (cargo test --test pcode, std)
- passed: build id: the image carries 'trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg d0b5944afc9464e4'

**qemu** (design-today-2026-10-06.qemu.tnfsw, built 2026-10-07T08:27:01Z with arm-none-eabi-gcc (15:13.2.rel1-2) 13.2.1 20231009):
- passed: flight_build gen --check: every generated source is what the design gives (algorithms d0b5944afc9464e4)
- passed: from_design: the flight algorithm blocks: fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives
- passed: C flight flags: every warning an error, no malloc/time/rand (make check)
- passed: vectors: pcode: 78 functions (10 by name through the dispatcher), 2616 calls, 45657 values, 45597 bit for bit, worst relative difference 6.6e-15; 0 failure(s); on the host build of these sources (fsw/build/test_pcode, the library posix seals); the Cortex-M image is not run on its vectors here
- passed: stack: deepest stack 12208 bytes of the 32768 reserved (37 %) (tools/fsw_stack.py)
- passed: build id: the image carries 'trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg d0b5944afc9464e4'

**qemu-rs** (design-today-2026-10-06.qemu-rs.tnfsw, built 2026-10-07T08:27:02Z with rustc 1.94.1 (e408947bf 2026-03-25); arm-none-eabi-gcc (15:13.2.rel1-2) 13.2.1 20231009):
- passed: flight_build gen --check: every generated source is what the design gives (algorithms d0b5944afc9464e4)
- passed: from_design: the flight algorithm blocks: fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives
- passed: vectors: 78 functions (24 by name through the dispatcher), 2616 calls, 45657 values, 45597 bit for bit, worst relative difference 6.592861854817441e-15; on the host build of these sources (cargo test --test pcode, std); the Cortex-M image is not run on its vectors here
- not run: stack: NOT RUN: tools/fsw_stack.py reads GCC's call graph of the C firmware; the Rust firmware has no stack analysis here
- passed: build id: the image carries 'trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg d0b5944afc9464e4'
