# From a node to tested code, by example

**In one line:** a node's pseudocode is the one statement of what it computes; `tools/groupcode.py` wires every computing row of a group into one module, translates it to Rust (the engine), MATLAB (the twin) and WebAssembly (the test app), and holds every translation to the interpreter and to the node's own test vectors, so no row's physics is written by hand (`docs/RELEASE_PLAN.md` P10).

## Say it simply

The author writes a recipe once. Three kitchens cook from it: the engine's, the twin's and the browser's. A taster (the interpreter) cooks it too, and every kitchen's dish must taste the same as the taster's, and as the author said it should.

**Where the story lies:** the taster is not a person. Agreement with the interpreter shows the translations are faithful to the pseudocode; only the node's own test vectors, with answers from outside the code, show the pseudocode is right.

## One node, all the way: `gd_0`, the worst gravity-gradient torque

**1. The node** (env group). Its author's question: *what is the largest gravity-gradient torque on the satellite?* Its answer is `tau_gg`, from three inputs it reads: the orbit radius `r` (`m2_4`), and the largest and smallest moments of inertia `i_max` (`s1_1`), `i_min` (`s1_3`). Its pseudocode (carried from `spec/physics/gnc.pc`, then the author's own):

```
fn gravity_gradient_torque_worst(r: real[km] in 6500 .. 8500, i_max: real[kg m^2] in 0.01 .. 100, i_min: real[kg m^2] in 0.01 .. 100) -> tau: real[N m]
    tau = 3*MU_E/(2*r^3)*|i_max - i_min|*sin(2*45 [deg])
end

## The node's answer, tau_gg, as gravity_gradient_torque_worst gives it.
fn gd_0(r: real[km] in 6500 .. 8500, i_max: real[kg m^2] in 0.01 .. 100, i_min: real[kg m^2] in 0.01 .. 100) -> tau_gg: real[N m]
    tau_gg = gravity_gradient_torque_worst(r, i_max, i_min)
end
```

**2. Wired** (`python3 tools/groupcode.py wire`): into `design/groups/env.pc` with every other computing row of `env`, and `design/groups/env.wire.json` says `gd_0` is answered by the function `gd_0`, with inputs `r, i_max, i_min` and the node's own test vectors.

**3. Translated** (`python3 tools/groupcode.py gen`): Rust, in `engine/crates/adcs-groups/src/env.rs`:

```rust
pub fn gd_0(r: f64, i_max: f64, i_min: f64) -> f64 {
    let mut tau_gg: f64 = 0.0;
    tau_gg = crate::env::gravity_gradient_torque_worst(r, i_max, i_min);
    tau_gg
}
```

MATLAB, in `matlab_sils/+asils/+groups/+env/gd_0.m`:

```matlab
function [tau_gg] = gd_0(r, i_max, i_min)
    tau_gg = asils.groups.env.gravity_gradient_torque_worst(r, i_max, i_min);
end
```

Every value is passed in SI (metres, kilograms, seconds), whatever unit the pseudocode states; the translator writes the conversion once.

**4. Tested** (`python3 tools/groupcode.py test`):
- the interpreter draws vectors inside each input's stated range; the Rust (`tests/vectors.rs`) and the twin (`t_physics_vectors` in Octave) must give the same outputs, bit for bit where no transcendental is involved, else within 1e-12;
- every node's own test vectors (`tests/fixtures.json`, answers from outside the code) through the generated Rust, within each vector's tolerance.

**5. Delivered** (`python3 tools/groupcode.py deliver`): `dist/test-apps/env.test-app.html`, one file the lead opens from disk. It runs each test vector in the interpreter and in WebAssembly built from the same Rust, side by side, and lets the lead try `gd_0` on numbers of their own.

## Why it can be trusted

| Claim | What holds it |
|---|---|
| The Rust is the pseudocode | every drawn vector reproduced (`tests/vectors.rs`) |
| The twin is the pseudocode | the same vectors in Octave (`matlab_sils/tests/run_all_tests.m`, `t_physics_vectors`) |
| The browser's WebAssembly is the Rust | it is the Rust, compiled for `wasm32-unknown-unknown`; the test apps compare it with the interpreter (`tests/browser/testapp.test.mjs`) |
| The pseudocode is right | the node's own test vectors, answers from outside the code |
| Nothing is hand-written | every computing row with pseudocode is in the generated code (`tests/test_groupcode.py`); the generated files say "do not edit" and `gen --check` finds an edit |
| Today's numbers are unchanged | the generated crates sit beside the engine; `check_all` flies the engine's own tests and parity as before |

## When something fails

- **The Rust (or twin) disagrees with the interpreter:** a translator bug. Fix `design/js/pcode_gen.js` (backend agent), regenerate, never edit the output.
- **A node's own test vector fails:** its pseudocode does not give its own answer. It goes back to its author, through the group's lead.
- **A function is written two ways in two groups:** the wiring stops and names both; the two leads agree on one.
