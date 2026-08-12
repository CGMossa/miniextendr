# Jiff timezone CHARSXP crossed an allocation unprotected

## Finding

`cached_class::set_posixct_tz` created a dynamic timezone CHARSXP and then
allocated its one-element STRSXP container before protecting or installing the
CHARSXP. R's extension manual requires callers to assume every returned R
object needs immediate protection and every later API call may allocate.

The relevant sequence was:

```rust
let tzone_charsxp = SEXP::charsxp(iana);        // unprotected
let tzone_sexp = Rf_allocVector(STRSXP, 1);     // may collect it
let guard = OwnedProtect::new(tzone_sexp);
tzone_sexp.set_string_elt(0, tzone_charsxp);
```

R's CHARSXP cache is weak during garbage collection: unmarked cached strings
are removed. Relying on interning is therefore not a rooting strategy.

## Test gap

The existing `gc_stress_jiff_zoned_vec` fixture claimed to cover the timezone
protection path but only forced numeric ALTREP element access. It never read or
asserted the `tzone` attribute, so a collected/reused timezone string was
outside its oracle. The public `jiff_zoned_vec_new` constructor also had no
testthat coverage.

Repeated gctorture runs did not reproduce corruption on this machine. The fix
is still required by R's documented API contract and removes dependence on GC
timing and implementation details.

## Resolution

- Protect the CHARSXP immediately, before allocating the STRSXP container.
- Keep the container protected until it is attached to the rooted POSIXct.
- Make the gctorture fixture assert the exact timezone attribute.
- Dogfood `jiff_zoned_vec_new` and `jiff_zoned_vec_first_element` from R,
  including the mixed-timezone rejection contract.
- Correct the stale cached-SEXPs examples and feature-gate matrix found during
  the module audit.
