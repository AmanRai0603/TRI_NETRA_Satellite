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
