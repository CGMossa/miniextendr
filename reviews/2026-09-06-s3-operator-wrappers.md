# S3 operator wrapper names (#1475)

Attempted to generate and install S3 operator methods from free functions and
impl blocks. The code generators interpolated operator names as bare R source:
`[.Foo <- function(...)` cannot be parsed. Trait wrappers also emitted bare
operators in generic guards and S7 registration references. The original package
install reproduced the syntax error at the bare `$.S3Counter` definition.

Use a shared R symbol renderer for these code positions, preserving ordinary
ASCII names and backtick-quoting operators and other non-syntactic names. Keep
roxygen method metadata unquoted so registration uses the original names. Teach
the duplicate-definition scan to recognize the quoted symbols, including
replacement names containing `<-`, instead of silently skipping their collisions.

Regression coverage includes installed `[`, `[[`, and `$` dispatch; both free
function prelude paths; inherent and trait wrapper generation; escaping; and
quoted-name collision detection. The first inherent test fixture used the old
flat `generic` attribute spelling; corrected it to `s3(generic = ...)` before
validating the generated R code.
