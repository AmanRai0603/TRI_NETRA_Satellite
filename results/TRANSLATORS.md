# The translators, held to the interpreter

**In one line:** every package of pseudocode translated to each language, built, and run on every vector the interpreter drew; an exact function bit for bit, any other within 1e-12 relative, or within the tolerance its documentation states relative to its largest output (a central difference, `## tolerance:`) (`tools/translators.py`, `docs/PLAN_2_0.md` S5).

| Package | Language | Functions | Values | Bit for bit | Worst relative error | Failing |
|---|---|---|---|---|---|---|
| physics | rust | 41 | 540 | 530 | 7.21e-16 | 0 |
| physics | c | 41 | 540 | 530 | 7.21e-16 | 0 |
| physics | matlab | 41 | 540 | 530 | 7.21e-16 | 0 |
| selftest | rust | 29 | 2988 | 2979 | 2.02e-16 | 0 |
| selftest | c | 29 | 2988 | 2979 | 2.02e-16 | 0 |
| selftest | matlab | 29 | 2988 | 2979 | 2.02e-16 | 0 |
| fsw | rust | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| fsw | c | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| fsw | matlab | 78 | 45657 | 45597 | 6.59e-15 | 0 |
| groups | rust | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| groups | c | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| groups | matlab | 82 | 20480 | 20469 | 1.31e-15 | 0 |
| env | rust | 614 | 2307603 | 2303770 | 1.93e-09 | 0 |
| env | c | 614 | 2307603 | 2303774 | 1.93e-09 | 0 |
| env | matlab | 614 | 2307603 | 2303770 | 1.93e-09 | 0 |
