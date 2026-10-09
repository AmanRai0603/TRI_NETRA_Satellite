# The translators, held to the interpreter

**In one line:** every package of pseudocode translated to each language, built, and run on every vector the interpreter drew; an exact function bit for bit, any other within 1e-12 relative, or within the tolerance its documentation states relative to its largest output (a central difference, `## tolerance:`) (`tools/translators.py`, `docs/PLAN_2_0.md` S5).

| Package | Language | Functions | Values | Bit for bit | Worst relative error | Failing |
|---|---|---|---|---|---|---|
| physics | rust | 41 | 540 | 530 | 7.21e-16 | 0 |
| physics | c | 41 | 540 | 530 | 7.21e-16 | 0 |
| physics | matlab | 41 | 540 | 530 | 7.21e-16 | 0 |
| selftest | rust | 27 | 2760 | 2751 | 2.02e-16 | 0 |
| selftest | c | 27 | 2760 | 2751 | 2.02e-16 | 0 |
| selftest | matlab | 27 | 2760 | 2751 | 2.02e-16 | 0 |
| fsw | rust | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| fsw | c | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| fsw | matlab | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| groups | rust | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| groups | c | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| groups | matlab | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| env | rust | 606 | 2469872 | 2466534 | 1.23e-09 | 0 |
| env | c | 606 | 2469872 | 2466534 | 1.23e-09 | 0 |
| env | matlab | 606 | 2469872 | 2466534 | 1.23e-09 | 0 |
