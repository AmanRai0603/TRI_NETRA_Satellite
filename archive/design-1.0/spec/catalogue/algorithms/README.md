# Algorithms

One file per flight algorithm a product can carry: `adcs-algorithm/1`.

An algorithm names the flight-software modes it implements and the parameters
that are **tuned per case** (SPEC.md §8.5). A parameter either sets a tree row
(`tree_id`, a row whose supplier is `tuned` in plan/case_inputs.toml) or is
internal to the flight software. Its bounds are the search box `adcs-tune`
works inside; a bound is a number in the parameter's unit, or
`"part:<slot>.<field>"`, read from the part filling that slot.

Every bound is a design decision. Until a person confirms it, it carries
`confirmed_by = "UNCONFIRMED · ..."`, and a product tuned inside an unconfirmed
box cannot be promoted to `offered` (H14).

Every parameter is restricted (D1): a client sees that a product was tuned,
and the margins it reached, never the values.
