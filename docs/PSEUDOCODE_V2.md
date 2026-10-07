# Pseudocode v2

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

The language the relations and algorithms of TRI-NETRA are written in once, then run, checked
and translated (docs/RELEASE_PLAN.md, P2). One implementation, in JavaScript with no
dependencies, serves every use:

| use | what runs it |
|---|---|
| the checker in the browser | `design/pcode_checker.html`: one offline page, built by `tools/pcode.py gen` with the language inlined |
| check, run, translate from the command line | `python3 tools/pcode.py …`, which runs `design/js/pcode_cli.mjs` under Node |
| the interpreter | `design/js/pcode.js` (`makeInterpreter`) |
| the translators to Rust and to MATLAB | `design/js/pcode_gen.js` |

The physics relations of `spec/plan/physics.toml` are written in it (`spec/physics/*.pc`); their
Rust translation is the crate `engine/crates/adcs-physics`, their MATLAB translation the package
`matlab_sils/+asils/+physics`. The flight algorithms of `fsw/pseudocode/` are written in it too
(`fsw/pseudocode/*.pc`, beside the prose `.md` of each chapter); the hand-written C and Rust flight
software are held to them (see *The flight software against its pseudocode*).

## A file

```
## Fluid momentum rings. source: idmas_v2
module fmr

## The viscous spin-down time constant of a channel: T = rho d^2/(32 mu).
fn spin_down_time(d: real[mm] in 1 .. 20, rho: real[kg/m^3] in 1000 .. 13600, mu: real[Pa s] in 0.0005 .. 0.005) -> t: real[s]
    t = rho*d^2/(32*mu)
end
```

- `module NAME` names the file's module: the Rust module, the MATLAB package `+NAME`, the first
  half of the registry name `fmr::spin_down_time`.
- `##` lines are documentation: they go into the generated Rust doc comments and MATLAB help.
  `#` starts an ordinary comment.
- A line ends a statement; a line ending in `\` continues, and so does any line inside
  `( )` or `[ ]`.
- Names are global across the files checked together: a function in one module calls one in
  another by its bare name (or `module.name(...)`).

## Values and units

Every value is held in **SI**. A unit in brackets states a dimension, and the checker holds every
expression to it; a literal's unit converts it to SI where it is read.

| written | means |
|---|---|
| `real[N m s]`, `real[kg/m^3]`, `real[kg/(m s)]` | a real number of that dimension, held in SI |
| `real`, `real[1]` | a plain number (an angle in rad is one: SI makes the radian dimensionless) |
| `int`, `bool` | a whole number; true or false |
| `vec3[m/s]`, `vec2[…]`, `vec4[…]`, `mat3[1/s]`, `quat` | fixed arrays (a quat is four plain numbers, scalar last) |
| `real[m][8]`, `int[7]` | an array of any length known when it is written |
| `Guid` | a record declared with `record` |
| `90 [deg]`, `500 [km]`, `[1, 2, 3] [mm]` | literals with a unit: 1.5707963267948966, 500000, [0.001, 0.002, 0.003] |
| `0` | a bare zero fits any dimension (and an array of zeros any array) |

The units are `m km cm mm um AU s ms min h day yr kg g N mN uN J W mW Pa kPa A mA C V ohm T uT nT
K Hz rpm rad deg arcsec`, combined with spaces (products), `^` (whole powers) and one `/`.
A type's unit is the unit it is *shown* in: `real[km]` and `real[m]` are the same type, and both
hold metres. An input's range `in 150 .. 2000` is in its own unit (km here), and on an array it
bounds each element.

The checker refuses, by name and line: adding or comparing different dimensions; `sin` of a
length; `sqrt` of a dimension with an odd power; a power that is not a whole-number literal
(`pow(x, y)` takes plain numbers); a vector times a vector (write `dot`, `cross`, or index);
an index that is not an int, or a literal index outside the array.

## Functions, several outputs, state between ticks

```
fn name(input: type [in lo .. hi], …) -> (out1: type, out2: type)    # or -> out: type
    …
end

proc name(input: type, …) -> (outputs…)
    state count: real[s] = 0 [s]          # kept between calls; starts at a constant
    …
end
```

- A `fn` has no memory. A `proc` keeps `state` from one call to the next (a filter, a dwell
  timer, a fault counter): in Rust it takes `st: &mut NameState` (a struct with its `Default`),
  in MATLAB it takes and returns `st` (empty on the first call).
- Every output is set on every path, or the checker says which is not. Inputs are not
  changed; copy one with `let`.
- A call of a function with several outputs is unpacked: `let h0, rho0, scale = atmosphere(h)`.
- No function calls itself, directly or around a loop of calls (flight code does not recurse).

## Statements

```
let x = expr                    # declare (the type is the expression's)
let x: real[m] = 0              # declare with a type
let a, b = f(…)                 # several outputs
x = expr        v[i] = expr     g.field = expr      a, b = f(…)
if cond … elif cond … else … end
for i in 0 .. n … end           # i = 0, 1, …, n-1; i is an int and is not assigned
settle max 50 until cond        # a loop that settles: the body, then the test, at most 50 times
    …
else                            # runs if it never settled (optional)
    …
end
```

A name is declared once in a function (no shadowing). `settle`'s test runs after each pass of
the body; the `else` block is the answer when the loop did not settle.

## Expressions

`+ - * /`, `^` (a whole-number literal power), comparisons `== != < <= > >=` (they do not chain),
`and or not`, `if c then a else b`, `|x|` (absolute value), `v[i]`, `M[i][j]`, `g.field`,
calls. Arrays: `+ -` element by element, a number times or over an array, matrix times vector,
matrix times matrix.

Builtins: `sqrt abs sin cos tan asin acos atan atan2 exp log log10 pow hypot min max clamp floor
ceil round sign fmod dot cross norm unit transpose real int len div rem band bor bxor shl shr`,
and `pi`. `div(a, b)` and `rem(a, b)` take ints and truncate as C and Rust do. `band bor bxor shl
shr` take non-negative ints below 2^53 (a byte, a CRC, a bit field) and are exact. `real(n)` makes
an int a real; an int also goes wherever a plain real is wanted. `int(x)` turns a plain real into an
int by dropping its fraction, as a C cast does (`int(v + 0.5)` is C's `(int16_t)(v + 0.5)`); it
stops the run on a value that is not finite or not below 2^53.

## Tables

```
## base altitude, base density, scale height
table atmosphere(h: real[km]) -> (h0: real[km], rho0: real[kg/m^3], scale: real[km]) step
    0, 1.225, 7.249
    25, 3.899e-2, 6.349
    …
end
```

Every row lists every column, the first being the key (so its dimension is the key's). Keys
rise strictly. `step` gives the last row at or below the key (the first row below the first
key); `linear` interpolates every column between the rows around the key and holds the end
rows beyond them. A table is called like a function: `let h0, rho0, scale = atmosphere(h)`.

## Records

```
record Guid
    q_off: quat
    axis: vec3[1]
    flip: bool
end
```

`Guid()` makes one with every field zero.

## Numbers: what every translation does the same way

The interpreter defines the arithmetic, and the translators write the same operations in the
same order, so a translation reproduces the interpreter bit for bit except where a maths
library's last bits differ:

- `x^k` is repeated multiplication from the left (`x^3 = (x*x)*x`; `x^-k = 1/(x^k)`), never `pow`.
- `min`, `max`, `abs`, `clamp`, `sign` are written out (`min(a, b)` is `b` only when `b < a`).
- `dot`, a matrix product and `norm` sum their products from the left; `unit(a) = a / max(norm(a), 1e-30)`.
- `round` rounds half away from zero; `fmod` is C's.
- Every literal is converted to SI once, by the checker, and written into the translation as the
  shortest decimal that reads back as the same double.
- An expression is evaluated in the order it is written, with no fused multiply-add.

`sin cos tan asin acos atan atan2 exp log log10 pow` come from each platform's library and may
differ in their last bits (more after a large argument is reduced). The tests hold a function
that uses none of them, directly or through what it calls, to **bit-for-bit** agreement, and the
others to 1e-12 relative.

## The tools

```
python3 tools/pcode.py check FILE...                  # units, types, outputs, recursion
python3 tools/pcode.py run FILE... --fn NAME --args JSON
python3 tools/pcode.py gen [--check]                  # the physics: Rust crate, MATLAB package, vectors, the checker page
python3 tools/pcode.py fixtures                       # the seeded test vectors of the physics rows, in the interpreter
```

`gen --check` (in `check_all` as `pseudocode`) also holds `spec/physics/*.pc` to
`spec/plan/physics.toml`: the same functions, with the same arguments in the same order.

**Translator = interpreter.** `gen` draws test vectors from the interpreter (inputs within each
input's range, from a fixed seed) into `engine/crates/adcs-physics/tests/vectors.json` and
`matlab_sils/data/physics_vectors.json`. The Rust test (`cargo test -p adcs-physics`) and the
twin's `t_physics_vectors` run every vector through their translation. Each vector also carries
each value's exact bits, for readers that do not parse decimals to the nearest double (Octave's
`jsondecode`).

A function with rare branches asks for more vectors with a line in its documentation,
`## vectors: 160`; otherwise the count is the package's, fewer for a function of many values.

Where random inputs would not reach the cases that matter (a byte stream almost never holds a
frame whose CRC checks), a function draws its inputs through another one,
`## inputs from: uart_stream`: that function runs on random inputs of its own, and its outputs give
the inputs of the same name; the rest are drawn as usual. A record input is drawn field by field,
each field in its own range.

**The flight software against its pseudocode.** `gen` also draws vectors for every function of
`fsw/pseudocode/*.pc` into `fsw/tests/pcode_vectors.txt`: a line per call, `name exact set nx ny`,
then each input's and output's IEEE-754 bits in hex (a `proc`'s calls in order, its state carried).
`fsw/tests/test_pcode.c` (in `make test`) and `fsw-rs/tests/pcode.rs` run every line through the
flight software, through an adapter per function that calls the runtime's own signature (which hands
the values to the algorithms written from the design, `fsw/alg`, `fsw-rs/src/alg`); the drivers go
through their public paths (`adcs_drv_read`, `adcs_drv_write`, `Drv::read`, `drv::write`) over an
in-memory HAL. A function the runtime has no signature of its own for (a helper only the generated
algorithms call, a vector generator, and in Rust the mode manager, FDIR and step laws it calls from
methods on its private state) is listed with its reason (`BY_NAME`) and called by its name through
the translator's vector dispatcher (`fsw/tests/alg_dispatch.c`, `fsw-rs/tests/alg/dispatch.rs`,
written by `tools/flight_build.py`; test code, never in an image). `NOT_IN_C` and `NOT_IN_RUST`,
the functions not checked, are empty; any other function with no adapter fails the test. A function
with no transcendental must agree bit for bit, one with, to 1e-12 relative.

**The physics against its sources.** `fixtures` runs every test vector `spec/plan/seed_content.toml`
transcribes from a source (IDMAS v2 today) through the interpreter, within the source's tolerance.

## What it does not do yet

- Generic units: a function is written for one dimension (`mission::closure` takes plain
  numbers, and its caller holds requirement and achieved value to one quantity).
- Calling a `proc` from another `proc` (each `proc` is called by the host with its own state).
- Arrays of records, strings, and variable-length arrays (a count beside a fixed array is the way:
  `real[m][8]` and `n: int`).
