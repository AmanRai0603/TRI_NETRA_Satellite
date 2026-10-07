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
| `Device` | a choice declared with `choice` (*Choices*) |
| `stream` | a random stream, the toolbox's (*Random streams*) |
| `90 [deg]`, `500 [km]`, `[1, 2, 3] [mm]` | literals with a unit: 1.5707963267948966, 500000, [0.001, 0.002, 0.003] |
| `0` | a bare zero fits any dimension (and an array of zeros any array) |
| `inf`, `-inf`, `nan` | the values that are not finite; like a bare 0 they fit any dimension (`let t: real[s] = inf`, `min(x, inf)`) |

**Named capacities.** An array's length may be a const that is a whole-number literal from 1, so a capacity is
stated once: `const NR = 8`, then `real[N m s][NR]`, `int[NR]`, `for i in 0 .. NR`. (A real's first brackets are its
unit: write `real[1][NR]`, not `real[NR]`.) The translations write the number; the generated documentation keeps the
name.

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

**Arrays by reference.** An input written `img: inout real[1][4096]` (an array, a record or a stream) may be changed by
the function, and the caller's variable is changed with it: a star-tracker frame, a catalogue, a scratch table is
handed over, not copied. The call is the whole right side of a `let` or an assignment (`let n = render(img, q)`),
each inout input a variable the caller may change (a `let`, an output, a state or an inout input of its own), named
by no other input of the call and not the assignment's target. Rust hands it over as `&mut`, C by its address, MATLAB
gives it back as an extra output (`[n, img] = render(img, q)`). A vector flattens an inout input among the inputs,
and its value after the call after the outputs.

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

Builtins: `sqrt abs sin cos tan asin acos atan atan2 exp log log10 log2 pow hypot min max clamp floor
ceil round trunc sign fmod dot cross norm unit transpose real int len div rem band bor bxor shl shr isnan isfinite
sort argsort`, the stream's `stream uniform normal normal3` (*Random streams*), and the constants `pi`, `inf` and
`nan`.

`div(a, b)` and `rem(a, b)` take ints and truncate as C and Rust do. `band bor bxor shl shr` take non-negative ints
below 2^53 (a byte, a CRC, a bit field) and are exact. `real(n)` makes an int a real; an int also goes wherever a
plain real is wanted. `int(x)` turns a plain real into an int by dropping its fraction, as a C cast does
(`int(v + 0.5)` is C's `(int16_t)(v + 0.5)`); it stops the run on a value that is not finite or not below 2^53; of a
choice it is the option's number.

`isnan(x)` and `isfinite(x)` take a number of any unit and give a bool; `x == nan` is false for every x, as in C. A
translation writes inf and nan as its language does (Rust `f64::INFINITY`, `f64::NAN`; C `INFINITY`, `NAN`; MATLAB
`Inf`, `NaN`), in a constant or a state's start too.

`sort(v)` is a vector of numbers (reals of any unit, or ints) in ascending order, and `argsort(v)` the indices that
order it (an `int` array): both stable, equal values keeping their order, by the one insertion sort every translation
writes (so even a nan lands in the same place). A median is `sort(v)[n/2]`; the three largest are the first three of
`argsort(-v)`. They are the toolbox's (Rust `rt::sort`, C `pc_sort_*`, MATLAB `asils.pc.sort_`), not the platform's
sort, whose order of equal values and of nan differs.

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

## Data tables

```
## IGRF-13 Gauss coefficients, a row per 5-year epoch (IAGA)
data IGRF_GH: real[nT][195][26]
    -31543, -2298, 5922, …
    …
end
```

A published model's coefficients, or any table of numbers the design holds, is a `data` table: a named 1-D or 2-D
array of reals (with a unit) or ints, its values written row by row (the outer index first: for `real[nT][195][26]`,
26 rows of 195), on as many lines as they take. The checker counts them against the type and holds an int table to
whole numbers; a real's values are converted to SI once. It is read like a constant array, `IGRF_GH[i][k]`,
`IGRF_GH[i]` (a row), `len(IGRF_GH)`, and never set. Where a design's node holds a table, the generator writes its
values here.

Each translation holds one copy and indexes it in place: Rust a `static` (`DATA_IGRF_GH`), C a `const` array
(`module_DATA_IGRF_GH`), MATLAB a function that keeps the table in a persistent variable (`IGRF_GH()` is the
table, `IGRF_GH(i, j)` an element, `IGRF_GH(i, ':')` a row). A `const` array is written into the MATLAB at each
use; a table of more than a few values is `data`.

## Choices

```
## A momentum device's kind
choice Device = wheel, ring, gyro, vsgyro

fn store(k: Device, h: real[N m s]) -> s: real[N m s]
    s = if k == Device.ring then 2*h else h
end
```

A choice names the options a value may take (a device's kind, a model, a law). It is a type of its own: an input,
an output, a record's field or a state may be one; an option is written `Device.ring`; two values of the same choice
compare with `==` and `!=`, and nothing else (no arithmetic, no order, no int in its place). `int(k)` is an option's
number, from 0 in the order declared (to index an array by it). The translations write a choice as that number: Rust
`i64` with a `pub const DEVICE_RING: i64 = 1`, C `int64_t` with `#define module_DEVICE_RING INT64_C(1)`, MATLAB the
number. A vector draws an input of a choice among its options, and flattens it as its number.

## Random streams

```
proc gyro(w: vec3[1/s], dt: real[s]) -> m: vec3[1/s]
    state g: stream = stream(42, "gyro")        # a seed and an id (an int, or a name in quotes)
    let n = normal3(g)                           # three normal draws; g advances
    m = w + (1e-4 [1/s])*n
end
```

A `stream` is the toolbox's counter-based random stream, value for value the engine's (adcs-sim-core `rng.rs`):
`stream(seed, id)` is SplitMix64 over a counter keyed by the seed and the id (an int, or a name in quotes hashed by
FNV-1a, as `rng.rs`'s `stream_id` does), `uniform(g)` a draw on (0, 1) from the counter's top 53 bits, `normal(g)` a
normal draw by Box-Muller with the spare kept for the next, `normal3(g)` three of them. A draw advances the stream it
is given: the stream is an inout input (see *Arrays by reference*), so the draw is the whole right side of a `let` or
an assignment, from a variable (a `let`, an output, a state, an inout input). A stream may be an input (inout, to be
drawn from and given back), an output, a state (starting at a `stream(...)` of constants) or a record's field; it is
not an array's element, and nothing compares or computes with it.

A uniform draw is exact: every translation gives `rng.rs`'s bits. A normal draw uses sqrt, log, sin and cos, so it is
held, as any transcendental, to 1e-12 relative (the Rust interpreter takes them from the libm crate, as `rng.rs` does,
and gives its bits; tests/streams.rs in trinetra-pcode). The translations: Rust `rt::Stream` (`rt::uniform(&mut g)`),
C `pc_stream` (`pc_uniform(&g)`), MATLAB six numbers with the 64-bit arithmetic in 32-bit halves
(`asils.pc.stream_uniform`). A vector flattens a stream as six numbers: its key's and its counter's high and low 32
bits, the spare, and whether there is one.

## Records

```
record Guid
    q_off: quat
    axis: vec3[1]
    flip: bool
end
```

`Guid()` makes one with every field zero. A field of a call's result, `f().x`, is written `getfield(f(), 'x')` in MATLAB,
which cannot index a call.

## Numbers: what every translation does the same way

The interpreter defines the arithmetic, and the translators write the same operations in the
same order, so a translation reproduces the interpreter bit for bit except where a maths
library's last bits differ:

- `x^k` is repeated multiplication from the left (`x^3 = (x*x)*x`; `x^-k = 1/(x^k)`), never `pow`.
- `min`, `max`, `abs`, `clamp`, `sign` are written out (`min(a, b)` is `b` only when `b < a`).
- `dot`, a matrix product and `norm` sum their products from the left; `unit(a) = a / max(norm(a), 1e-30)`.
- `round` rounds half away from zero; `trunc` drops the fraction (toward zero); `fmod` is C's.
- Every literal is converted to SI once, by the checker, and written into the translation as the
  shortest decimal that reads back as the same double.
- An expression is evaluated in the order it is written, with no fused multiply-add.

`sin cos tan asin acos atan atan2 exp log log10 log2 pow hypot` come from each platform's library and may
differ in their last bits (more after a large argument is reduced). `hypot` is the library's own (Rust `f64::hypot`, C
`hypot`, MATLAB `hypot`), not `sqrt(a*a + b*b)`, so a transcription of code that calls it gives that code's bits
(trinetra-toolbox/3, S7.3). `pow` is the library's pow in every build too: the Rust translation for std hands
`f64::powf` its exponent through `core::hint::black_box`, because the optimiser would otherwise rewrite `pow(x, 2.0)` as
`x*x`, whose last bit can differ from the library's (S7.3b); a square is written `x^2`, and a transcription of code that
squared through `powf(x, 2.0)`, which the compiler made `x*x`, writes `x^2`. The tests hold a function
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
`## vectors: 160`; otherwise the count is the package's, fewer for a function of many values (a function over large
workspaces may ask for fewer, `## vectors: 2`).

A function whose result amplifies its maths library's last bit (a central difference amplifies it about a million
times) states the tolerance its translations are held to, with its reason beside it, `## tolerance: 1e-8`: each output
within that of the largest of its outputs, in place of 1e-12 relative (`tools/translators.py`; S7.3d's tides).

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
- Arrays of records, strings (but a stream's name), and variable-length arrays (a count beside a fixed array is
  the way: `real[m][8]` and `n: int`).
- A record as a state's start in the Rust and MATLAB translations (C has it): a state that is a record starts at
  zero and is filled on the first call.
