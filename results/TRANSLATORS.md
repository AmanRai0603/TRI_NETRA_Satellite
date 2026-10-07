# The translators, held to the interpreter

**In one line:** every package of pseudocode translated to each language, built, and run on every vector the interpreter drew; an exact function bit for bit, any other within 1e-12 relative (`tools/translators.py`, `docs/PLAN_2_0.md` S5).

| Package | Language | Functions | Values | Bit for bit | Worst relative error | Failing |
|---|---|---|---|---|---|---|
| physics | rust | 41 | 540 | 530 | 7.21e-16 | 0 |
| physics | c | 41 | 540 | 530 | 7.21e-16 | 0 |
| physics | matlab | 41 | 540 | 530 | 7.21e-16 | 0 |
| selftest | rust | 24 | 2448 | 2439 | 2.02e-16 | 0 |
| selftest | c | 24 | 2448 | 2439 | 2.02e-16 | 0 |
| selftest | matlab | 24 | 2448 | 2439 | 2.02e-16 | 0 |
| fsw | rust | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| fsw | c | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| fsw | matlab | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| groups | rust | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| groups | c | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| groups | matlab | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| env | rust | 96 | 7212 | 7068 | 1.23e-13 | 0 |
| env | c | 96 | 7212 | 7068 | 1.23e-13 | 0 |
| env | matlab | 96 | 7212 | 7068 | 1.23e-13 | 0 |
