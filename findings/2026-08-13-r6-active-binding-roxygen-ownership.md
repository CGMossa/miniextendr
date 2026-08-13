# R6 active-binding docs are attached to nonexistent dynamic methods

## Finding

Generated `@field` blocks immediately precede
`Class$set("active", "name", ...)` calls. roxygen2 8.0.0's dynamic `$set()`
parser records every target as an R6 method without inspecting the first
`"active"` argument. Its method-documentation pass consequently tries to attach
each field tag and generated back-reference to a method that does not exist.

## User impact

Every `just force-document` emits fourteen warnings for the seven dogfooded
active bindings. The rendered fields currently survive only because roxygen2's
separate class-field pass also consumes the tags; routine documentation work is
therefore noisy and masks new warnings.

## Resolution

Emit active-binding `@field` tags on the R6 class documentation block, which is
where roxygen2 resolves fields and active bindings. Keep dynamic
`$set("active", ...)` calls free of adjacent roxygen blocks, preserve the
rendered Active bindings sections, and avoid duplicating explicit class-level
field documentation.
