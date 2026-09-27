# Integrators

Uniform signature via the dispatcher:
```matlab
sol = integ.run(method, f, t0, tf, y0, opts, tout);
% f = @(t,y) [v; a],  y = [r; v] (6x1)
% sol.t sol.y sol.r sol.v sol.nodes sol.raw   (raw = dense-output nodes)
```
`op.propagate` calls this for you; use it directly only for bare ODEs.

| method | order | type | opts | notes |
|--------|-------|------|------|-------|
| `rk4` | 4 | fixed | `.h` | classic; simplest |
| `nystrom4` | 4 | fixed (2nd-order system) | `.h` | efficient for `r''=a(r)` |
| `rk6luther` | 6 | fixed | `.h` | high accuracy fixed-step |
| `gaussJackson8` | 8 | fixed multistep (PECE) | `.h` | best fixed-step accuracy/cost; self-contained RK6 startup, Kahan-summed |
| `rk45` | 5(4) | adaptive (DP5, FSAL) | `.rtol,.atol` | general-purpose adaptive |
| `rk78` | 8(7) | adaptive (DP8) | `.rtol,.atol` | **workhorse**; fewest steps for tight tolerances |

Dense output: `sol.stateAt(tq)` (piecewise cubic Hermite from
`integ.hermite`) gives `r,v` at arbitrary times without re-propagating; accepts
vector `tq`.

### Picking one
- Quick/interactive: `rk78` with `rtol=1e-9`.
- Long precise arcs at fixed step: `gaussJackson8` (classic for orbit work).
- Reproducible fixed-step benchmarking: `rk4`/`rk6luther`.

`ex05_integrator_compare.m` tabulates accuracy vs node count for all six.
Representative two-body, one period: `rk78` reaches ~1e-4 m in ~40 nodes;
`gaussJackson8` (h=30 s) ~4e-5 m; `rk4` (h=30 s) ~1.5 m.
