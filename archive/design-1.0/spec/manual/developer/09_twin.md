# The MATLAB twin, in lockstep
<!-- kind: how-to; depth: read -->

**In one line:** every element of SILS is written in the platform and in the MATLAB twin in the same pull request, from the phase that builds it, so the MATLAB SILS zip on every push is whole and nobody ports anything afterwards.

Why the twin exists and what it holds is in SPEC.md §10.8; the rule and its checks are §10.8.7. The idea is ESA's: one model definition carried from the MATLAB engineering simulator into every later facility, not a second model written after the first.

## Know where each element lives

1. Look it up in the map:

<!-- since P1 -->
```
cargo xtask twin list            # python3 tools/twin_check.py --list in this package
```

   Each line gives the element, the phase that builds it, the rungs that reuse it, its platform side and its twin side. `plan/twin_map.toml` is the source: four families (physics functions, algorithms, metric kinds, part kinds) expand over their registries, and twenty single elements cover the rest.
2. A new physics function, algorithm, metric kind or part kind is in the map as soon as it is in its registry. You do not edit the map for it; the check then asks for both sides.

## Make a change to an element

1. Change the platform side (`crates/…`, or `fsw/src/<id>.c`).
2. Change the twin side (`matlab_sils/+asils/…`) in the same branch, from the same relation, source or algorithm description.
3. Run the element's shared test in both engines (the map's `test` field names it: fixtures, the golden scenarios, the reference draws).
4. Run the checks:

<!-- since P1 -->
```
cargo xtask twin check                              # the map is whole
cargo xtask twin check --repo . --phase P3          # every element P3 builds exists on both sides
cargo xtask twin check --changed changed.txt        # what CI runs on your pull request
```

5. If the other side truly needs nothing (a speed-up with identical results, a comment, a rename the map follows), label the pull request `twin:none` and say why in one line; the job passes that line as `--reason` and refuses the label without it.
6. When P3M runs beside P5, P6 or P7, the job is given every green branch (`--phase P6,P3M`), so the twin's app is asked for only once P3M is green.

## Add an element that is not in a registry

1. Add an `[[element]]` to `plan/twin_map.toml`: the id, both sides and their files, the phase, the rungs and the shared test.
2. If it exists on one side by design, write `only = "platform"` or `only = "twin"` and the reason in `why`. One reviewer from GNC, which owns the twin's risk (R-10), besides the usual one.
3. `cargo xtask twin check` must pass.

## Prototype an algorithm

1. Write `+asils/+fsw/<id>.m` and `catalogue/algorithms/<id>.toml` with `prototype = true`. The map marks it twin-only.
2. When it is ready, write `fsw/src/<id>.c` and set `prototype = false` in the same pull request. From then on the two move together, and `matlab-parity` compares them.

## If it goes wrong

| Finding | Means | Do |
|---|---|---|
| TW05 | an element the phase builds is missing on one side | write the missing side; never lower the phase |
| TW06 | one side of an element changed without the other | change the other side, or add `twin:none` with the reason |
| `pack_matlab` refuses | a twin folder exists without one of its files, or the phase's twin is incomplete | the refusal names the elements: write them |
| the two engines disagree | the parity ledger line has no cause yet | find which one is wrong and state the cause (H15); never tune one to match the other |

## Every rule

The table is generated from `tools/twin_check.py`, and CI fails if the two differ.

<!-- twin:begin -->
| Code | The rule |
|---|---|
| TW01 | the map is adcs-twin-map/1; every element has a unique id, a phase that exists and rungs from sils, pil, oils, hils |
| TW02 | an element is on both sides, platform and twin, or is marked only = platform or only = twin, with the reason why |
| TW03 | a twin side is an asils. name in matlab_sils/+asils/; a platform side is a file under crates/ or fsw/ |
| TW04 | every family's registry yields its items, so each physics function, algorithm, metric kind and part kind is in the map |
| TW05 | in a checkout, every element built by the phases reached (and the phases they need) exists on both sides |
| TW06 | a change to one side of an element changes the other in the same pull request, or carries twin:none with a reason; a module file holding several functions changes with at least one of their twins |
<!-- twin:end -->
