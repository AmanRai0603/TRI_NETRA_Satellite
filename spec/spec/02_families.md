
---

## 2. Product configurations

The product is fixed; the configuration is chosen. There are four families, defined as data in `catalogue/families.toml`:

| Family id | What it is | IDMAS | Offered for | Counts it sets |
|---|---|---|---|---|
| `mtq` | Magnetorquers only | V1 | 0.5–30 kg | coils 3–6; wheels, rings, thrusters 0 |
| `mtq_rw` | Magnetorquers + reaction wheels | — | 1–500 kg | coils 3–6, wheels 3–4 |
| `mtq_fmr` | Magnetorquers + fluid momentum rings | V2 | 1–50 kg | coils 3–6, rings 3–4 |
| `mtq_fmr_rcs` | Magnetorquers + fluid rings + RCS | V3 | 50–500 kg | coils 3–6, rings 3–4, thrusters 4–16 |

Four rules follow from treating families as data.

1. **Absence is a count of zero, never a missing row.** Every family runs on the same tree. A satellite with no wheels has `Reaction wheels fitted = 0`, and every wheel row answers zero cleanly: zero momentum, zero power, zero mass. So one closure run answers for every family, and comparing families is comparing candidates.
2. **The family comes from the product, not the case.** A case states the satellite and the requirements. The product (or, in designer mode, the part combination) supplies every count: the filled slots' counts, and zero for the rest (§8.4). A case may narrow the search with `meta.families`, but it never has to name one.
3. **The tree says which rows each family puts in play.** The fifth field of every tree row carries its families (§5.6). The face hides out-of-play groups from a client, and the solver uses the same field to skip whole branches.
4. **Adding a family is a catalogue change, not code.** Magnetorquers + wheels + RCS would be a new `[[family]]`, with its slots and the algorithms it may carry. The solver never names a family in code.

The coils are in every family because they are the safety floor. They detumble, they dump, and they own safe mode. IDMAS v2 §01 is explicit that detumble, safe mode and momentum dumping never depend on the newer hardware. The solver refuses a configuration without coils by name, and every product carries the `bdot` algorithm.

What the case's requirements decide, through the solver:

- the product: in client mode among the offered products, in designer mode the family, parts and counts (by agility, pointing, mass class and fault tolerance: IDMAS v2 §15 "What it can fly");
- the parts within the family (by the closures);
- the counts within the family's range (by fault tolerance: IDMAS v2 §05.2 baseline, fault-tolerant and large-bus layouts);
- the sensor suite (by the knowledge requirements);
- the tuned algorithm parameters (by SILS against the case, §8.5).

One gap is stated rather than hidden: step-and-stare agility on large buses (IDMAS v2 §09.1, "Rings + RCS cover every agility class: No — the gap"). The solver returns it as a named refusal for `mtq_fmr_rcs` and screens `mtq_rw` for comparison.
