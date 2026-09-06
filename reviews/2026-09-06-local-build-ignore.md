# Local checkout marker was not excluded from package builds

Attempted: verify that `use_local_miniextendr()` excludes its local checkout
marker from R source packages and can repair the previously written rule.

Failure: four regression assertions failed. The ignore pattern did not match
`.miniextendr-local`, and enabling or disabling the override left the malformed
entry behind. The old test only searched for a substring in the ignore file.

Root cause: a pre-escaped regular expression was passed to
`usethis::use_build_ignore()`, which escapes and anchors literal filenames.

Fix: pass the plain filename, remove the exact malformed rule on both helper
paths (including disabling an absent marker), and test actual regex matching,
repeat calls, and preservation of unrelated entries. See #1405.
