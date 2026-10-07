# Reference flight software (C99). Owner: Agastya. Copyright (c) 2026 Agastya.
#   make            libadcs_fsw.a + the host unit tests
#   make test       run them
#   make check      strict warnings, no malloc/time/rand/recursion scan
CC ?= gcc
CFLAGS ?= -O2 -Wall -Wextra -pedantic
# the determinism flags: every build keeps them whatever CFLAGS a caller passes (C = Rust bit for bit)
override CFLAGS += -std=c99 -ffp-contract=off -fno-fast-math -Iinclude -Ialg/include
SRC = src/adcs_math.c src/adcs_env.c src/adcs_est.c src/adcs_guid.c src/adcs_ctl.c src/adcs_alloc.c src/adcs_drv.c src/adcs_params.c src/adcs_modes.c src/adcs_fdir.c src/adcs_fsw.c
# the algorithms, written from the design by tools/flight_build.py (never edited); a translation keeps every name the
# pseudocode declares, so an unused local or input is not a warning there
ALG = $(sort $(wildcard alg/src/*.c))
ALGFLAGS = -Wno-unused-variable -Wno-unused-but-set-variable -Wno-unused-parameter
OBJ = $(SRC:src/%.c=build/%.o) $(ALG:alg/src/%.c=build/alg_%.o)

all: build/libadcs_fsw.a build/test_fsw build/test_pcode

build/%.o: src/%.c include/*.h src/*.h alg/include/*.h | build
	$(CC) $(CFLAGS) -c $< -o $@

build/alg_%.o: alg/src/%.c alg/include/*.h | build
	$(CC) $(CFLAGS) $(ALGFLAGS) -c $< -o $@

build/libadcs_fsw.a: $(OBJ)
	ar rcs $@ $^

build/test_fsw: tests/test_fsw.c tests/hal_stub.c build/libadcs_fsw.a
	$(CC) $(CFLAGS) -Itests tests/test_fsw.c tests/hal_stub.c build/libadcs_fsw.a -lm -o $@

# the flight algorithms against their pseudocode (the .pc files of pseudocode/): the vectors,
# tests/pcode_vectors.txt, are written by tools/pcode.py gen; tests/alg_dispatch.c, the translator's vector dispatcher
# (test code, never in the library), is written by tools/flight_build.py
build/test_pcode: tests/test_pcode.c tests/hal_stub.c tests/alg_dispatch.c build/libadcs_fsw.a
	$(CC) $(CFLAGS) $(ALGFLAGS) -Isrc -Itests tests/test_pcode.c tests/hal_stub.c tests/alg_dispatch.c build/libadcs_fsw.a -lm -o $@

build:
	mkdir -p build

test: build/test_fsw build/test_pcode
	./build/test_fsw
	./build/test_pcode

check:
	$(CC) $(CFLAGS) -Werror -fsyntax-only $(SRC)
	$(CC) $(CFLAGS) $(ALGFLAGS) -Werror -fsyntax-only $(ALG)
	@! grep -nE '\b(malloc|calloc|realloc|free|time|rand|srand)\s*\(' src/*.c alg/src/*.c || (echo "forbidden call in flight code" && false)

clean:
	rm -rf build

.PHONY: all test check clean

# ---------------- virtual OBC targets (docs/VIRTUAL_OBC.md) ----------------
LINK_SRC = targets/link/adcs_link.c
build/obc_posix: $(LINK_SRC) targets/posix/main.c build/libadcs_fsw.a
	$(CC) $(CFLAGS) -D_POSIX_C_SOURCE=200809L -Wno-pedantic -Itargets/link $(LINK_SRC) targets/posix/main.c build/libadcs_fsw.a -lm -o $@

RS_HOST = ../fsw-rs/target/release/libadcs_fsw.a
$(RS_HOST): ../fsw-rs/src/*.rs ../fsw-rs/src/alg/*.rs
	cd ../fsw-rs && cargo build --locked --release -q --no-default-features --features cabi
build/obc_posix_rs: $(LINK_SRC) targets/posix/main.c $(RS_HOST)
	$(CC) $(CFLAGS) -D_POSIX_C_SOURCE=200809L -Wno-pedantic -Itargets/link $(LINK_SRC) targets/posix/main.c $(RS_HOST) -lm -lpthread -ldl -o $@

ARMCC = arm-none-eabi-gcc
ARMFLAGS = -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16 -std=c99 -O2 -ffp-contract=off -fno-fast-math -ffunction-sections -fdata-sections -Wall -Wextra -Iinclude -Ialg/include -Itargets/link $(ALGFLAGS)
ARMLD = -T targets/qemu-mps2/link.ld -nostartfiles --specs=nano.specs --specs=nosys.specs -Wl,--gc-sections -lm
build/obc_qemu.elf: $(SRC) $(ALG) $(LINK_SRC) targets/qemu-mps2/main.c targets/qemu-mps2/link.ld | build
	$(ARMCC) $(ARMFLAGS) $(SRC) $(ALG) $(LINK_SRC) targets/qemu-mps2/main.c $(ARMLD) -o $@

RS_ARM = ../fsw-rs/target/thumbv7em-none-eabihf/release/libadcs_fsw.a
$(RS_ARM): ../fsw-rs/src/*.rs ../fsw-rs/src/alg/*.rs
	cd ../fsw-rs && cargo build --locked --release -q --no-default-features --features cabi --target thumbv7em-none-eabihf
build/obc_qemu_rs.elf: $(LINK_SRC) targets/qemu-mps2/main.c targets/qemu-mps2/link.ld $(RS_ARM) | build
	$(ARMCC) $(ARMFLAGS) $(LINK_SRC) targets/qemu-mps2/main.c $(RS_ARM) $(ARMLD) -Wl,--allow-multiple-definition -o $@

# soft OILS: exact per-step instruction counts from QEMU (a host shared object)
build/insn_count.so: targets/qemu-mps2/insn_count.c | build
	$(CC) -std=c99 -O2 -Wall -Wextra -D_POSIX_C_SOURCE=200809L -shared -fPIC $< -o $@

obc: build/obc_posix build/obc_posix_rs build/obc_qemu.elf build/obc_qemu_rs.elf build/insn_count.so
.PHONY: obc
