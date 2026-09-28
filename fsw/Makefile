# Reference flight software (C99). Owner: Agastya. Copyright (c) 2026 Agastya.
#   make            libadcs_fsw.a + the host unit tests
#   make test       run them
#   make check      strict warnings, no malloc/time/rand/recursion scan
CC ?= gcc
CFLAGS ?= -std=c99 -O2 -ffp-contract=off -fno-fast-math -Wall -Wextra -pedantic -Iinclude
SRC = src/adcs_math.c src/adcs_env.c src/adcs_est.c src/adcs_ctl.c src/adcs_alloc.c src/adcs_drv.c src/adcs_params.c src/adcs_fsw.c
OBJ = $(SRC:src/%.c=build/%.o)

all: build/libadcs_fsw.a build/test_fsw

build/%.o: src/%.c include/*.h | build
	$(CC) $(CFLAGS) -c $< -o $@

build/libadcs_fsw.a: $(OBJ)
	ar rcs $@ $^

build/test_fsw: tests/test_fsw.c tests/hal_stub.c build/libadcs_fsw.a
	$(CC) $(CFLAGS) -Itests tests/test_fsw.c tests/hal_stub.c build/libadcs_fsw.a -lm -o $@

build:
	mkdir -p build

test: build/test_fsw
	./build/test_fsw

check:
	$(CC) $(CFLAGS) -Werror -fsyntax-only $(SRC)
	@! grep -nE '\b(malloc|calloc|realloc|free|time|rand|srand)\s*\(' src/*.c || (echo "forbidden call in flight code" && false)

clean:
	rm -rf build

.PHONY: all test check clean

# ---------------- virtual OBC targets (docs/VIRTUAL_OBC.md) ----------------
LINK_SRC = targets/link/adcs_link.c
build/obc_posix: $(LINK_SRC) targets/posix/main.c build/libadcs_fsw.a
	$(CC) $(CFLAGS) -D_POSIX_C_SOURCE=200809L -Wno-pedantic -Itargets/link $(LINK_SRC) targets/posix/main.c build/libadcs_fsw.a -lm -o $@

RS_HOST = ../fsw-rs/target/release/libadcs_fsw.a
$(RS_HOST): ../fsw-rs/src/*.rs
	cd ../fsw-rs && cargo build --release -q --no-default-features --features cabi
build/obc_posix_rs: $(LINK_SRC) targets/posix/main.c $(RS_HOST)
	$(CC) $(CFLAGS) -D_POSIX_C_SOURCE=200809L -Wno-pedantic -Itargets/link $(LINK_SRC) targets/posix/main.c $(RS_HOST) -lm -lpthread -ldl -o $@

ARMCC = arm-none-eabi-gcc
ARMFLAGS = -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16 -std=c99 -O2 -ffp-contract=off -fno-fast-math -ffunction-sections -fdata-sections -Wall -Wextra -Iinclude -Itargets/link
ARMLD = -T targets/qemu-mps2/link.ld -nostartfiles --specs=nano.specs --specs=nosys.specs -Wl,--gc-sections -lm
build/obc_qemu.elf: $(SRC) $(LINK_SRC) targets/qemu-mps2/main.c targets/qemu-mps2/link.ld | build
	$(ARMCC) $(ARMFLAGS) $(SRC) $(LINK_SRC) targets/qemu-mps2/main.c $(ARMLD) -o $@

RS_ARM = ../fsw-rs/target/thumbv7em-none-eabihf/release/libadcs_fsw.a
$(RS_ARM): ../fsw-rs/src/*.rs
	cd ../fsw-rs && cargo build --release -q --no-default-features --features cabi --target thumbv7em-none-eabihf
build/obc_qemu_rs.elf: $(LINK_SRC) targets/qemu-mps2/main.c targets/qemu-mps2/link.ld $(RS_ARM) | build
	$(ARMCC) $(ARMFLAGS) $(LINK_SRC) targets/qemu-mps2/main.c $(RS_ARM) $(ARMLD) -Wl,--allow-multiple-definition -o $@

# soft OILS: exact per-step instruction counts from QEMU (a host shared object)
build/insn_count.so: targets/qemu-mps2/insn_count.c | build
	$(CC) -std=c99 -O2 -Wall -Wextra -D_POSIX_C_SOURCE=200809L -shared -fPIC $< -o $@

obc: build/obc_posix build/obc_posix_rs build/obc_qemu.elf build/obc_qemu_rs.elf build/insn_count.so
.PHONY: obc
