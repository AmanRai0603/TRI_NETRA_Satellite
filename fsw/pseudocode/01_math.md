# 01 — Math kernel (`adcs_math`, `math`)

All fixed-size; no allocation. `v3`, `m3` (row-major 3×3), `q4`.

```
cross(a, b)            = [a2 b3 − a3 b2, a3 b1 − a1 b3, a1 b2 − a2 b1]
skew(a)                = [[0, −a3, a2], [a3, 0, −a1], [−a2, a1, 0]]
norm(a), unit(a)       unit(a) = a / max(norm(a), 1e-30)

qmult(a, b):           # Hamilton, scalar last
    v = a.w b.v + b.w a.v + cross(a.v, b.v)
    w = a.w b.w − dot(a.v, b.v)
qconj(q)               = [−q.v, q.w]
qnorm(q)               = q / norm(q)
dcm(q):                # passive ECI -> body
    x,y,z,w = q
    [[1−2(y²+z²), 2(xy+zw),   2(xz−yw)],
     [2(yx−zw),   1−2(x²+z²), 2(yz+xw)],
     [2(zx+yw),   2(zy−xw),   1−2(x²+y²)]]
fromdcm(R):            # the largest of (trace, R11, R22, R33) picks the form (no sign forcing)
    0: [R23−R32, R31−R13, R12−R21, 1+tr]
    1: [1+2R11−tr, R12+R21, R13+R31, R23−R32]
    2: [R21+R12, 1+2R22−tr, R23+R32, R31−R13]
    3: [R31+R13, R32+R23, 1+2R33−tr, R12−R21]        then qnorm
fromrotvec(θ)          see 00_conventions
qangle(a, b)           = 2 acos(min(1, |dot(a, b)|))

solve3(M, b)           Cramer / adjugate; if |det| < 1e-300 return 0
inv3(M)
pinv_rows(A[3×n], n ≤ 8):     # minimum-norm right inverse, rank-safe
    S = A Aᵀ + ε I3, ε = 1e-12 · max(trace(A Aᵀ)/3, 1e-30)
    return Aᵀ inv3(S)         # equals pinv(A) for full row rank; a damped
                              # least-squares inverse when a column set is rank deficient
jacobi_eig4(K sym 4×4) -> (λ[4], V[4×4]):
    cyclic Jacobi, fixed 12 sweeps (converges to 1e-15 for 4×4), deterministic order
    (p,q) = (0,1),(0,2),(0,3),(1,2),(1,3),(2,3)
```

`pinv_rows` replaces MATLAB `pinv` for allocation: the rotor axis sets are 3×n with n ≤ 8, and
the damped form never divides by a vanishing singular value (a lost axis after FDIR).
