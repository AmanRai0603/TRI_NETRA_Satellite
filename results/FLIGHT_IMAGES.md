# The sealed flight images

Owner: Agastya. Written by `python3 tools/flight_build.py seal` (docs/PLAN_2_0.md S6). Each image is one target's
build of the flight software written from the design: the generated sources, the binary and the configuration
blobs of every scenario, each with its sha256, and the checks made on it. The `.tnfsw` files sit in
`results/flight_images/` and are not kept in git (the Rust host library alone is 8 MB); `index.json` is.
`python3 tools/flight_build.py verify FILE` rechecks one; `which RUN` names the image a result flew (its manifest's
fsw.build_id is the image's runtime version). They are the developer's builds, not signed.

| target | design | runtime version (build id) | image sha256 | files | checks |
|---|---|---|---|---:|---|
| posix | today 2026-10-06 (`b25aba885f8f`) | `trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg 01e2e89cb3b39b4f` | `f22f81c566188fde` | 111 | 5 passed |
| posix-rs | today 2026-10-06 (`b25aba885f8f`) | `trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg 01e2e89cb3b39b4f` | `131f6d47d39f2fa5` | 111 | 4 passed |
| qemu | today 2026-10-06 (`b25aba885f8f`) | `trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg 01e2e89cb3b39b4f` | `4f2662b718f684f3` | 111 | 6 passed |
| qemu-rs | today 2026-10-06 (`b25aba885f8f`) | `trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg 01e2e89cb3b39b4f` | `d467cb132c8be92d` | 111 | 4 passed; not run: stack |

## Checks

**posix** (design-today-2026-10-06.posix.tnfsw, built 2026-11-13T03:53:59Z with gcc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0; GNU ar (GNU Binutils for Ubuntu) 2.42):
- passed: flight_build gen --check: every generated source is what the design gives (algorithms 01e2e89cb3b39b4f)
- passed: from_design: the flight algorithm blocks: fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives
- passed: C flight flags: every warning an error, no malloc/time/rand (make check)
- passed: vectors: pcode: 101 functions (32 by name through the dispatcher), 3430 calls, 50113 values, 50027 bit for bit, worst relative difference 5.7e-14; 0 failure(s); on the host build of these sources (fsw/build/test_pcode, the library posix seals)
- passed: build id: the image carries 'trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg 01e2e89cb3b39b4f'

**posix-rs** (design-today-2026-10-06.posix-rs.tnfsw, built 2026-11-13T03:54:05Z with rustc 1.94.1 (e408947bf 2026-03-25); cargo 1.94.1 (29ea6fb6a 2026-03-24)):
- passed: flight_build gen --check: every generated source is what the design gives (algorithms 01e2e89cb3b39b4f)
- passed: from_design: the flight algorithm blocks: fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives
- passed: vectors: 101 functions (47 by name through the dispatcher), 3430 calls, 50113 values, 50027 bit for bit, worst relative difference 5.6630434964908815e-14; on the host build of these sources (cargo test --test pcode, std)
- passed: build id: the image carries 'trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg 01e2e89cb3b39b4f'

**qemu** (design-today-2026-10-06.qemu.tnfsw, built 2026-11-13T03:54:12Z with arm-none-eabi-gcc (15:13.2.rel1-2) 13.2.1 20231009):
- passed: flight_build gen --check: every generated source is what the design gives (algorithms 01e2e89cb3b39b4f)
- passed: from_design: the flight algorithm blocks: fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives
- passed: C flight flags: every warning an error, no malloc/time/rand (make check)
- passed: vectors: pcode: 101 functions (32 by name through the dispatcher), 3430 calls, 50113 values, 50027 bit for bit, worst relative difference 5.7e-14; 0 failure(s); on the host build of these sources (fsw/build/test_pcode, the library posix seals); the Cortex-M image is not run on its vectors here
- passed: stack: deepest stack 13576 bytes of the 32768 reserved (41 %) (tools/fsw_stack.py)
- passed: build id: the image carries 'trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg 01e2e89cb3b39b4f'

**qemu-rs** (design-today-2026-10-06.qemu-rs.tnfsw, built 2026-11-13T03:54:15Z with rustc 1.94.1 (e408947bf 2026-03-25); arm-none-eabi-gcc (15:13.2.rel1-2) 13.2.1 20231009):
- passed: flight_build gen --check: every generated source is what the design gives (algorithms 01e2e89cb3b39b4f)
- passed: from_design: the flight algorithm blocks: fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives
- passed: vectors: 101 functions (47 by name through the dispatcher), 3430 calls, 50113 values, 50027 bit for bit, worst relative difference 5.6630434964908815e-14; on the host build of these sources (cargo test --test pcode, std); the Cortex-M image is not run on its vectors here
- not run: stack: NOT RUN: tools/fsw_stack.py reads GCC's call graph of the C firmware; the Rust firmware has no stack analysis here
- passed: build id: the image carries 'trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg 01e2e89cb3b39b4f'
