# minirextendr source-package check notes (#1409, #1410)

Checking the built package reproduced the non-standard top-level `AGENTS.md`
note. `.Rbuildignore` excluded its sibling `CLAUDE.md` but omitted the required
agent guidance file. Added an anchored exclusion while retaining both files in
the repository.

The release-workflow help linked to `miniextendr::assert_utf8_locale_now`, a
fixture in the separate framework package, not a topic in `minirextendr`.
The proposed same-package link would also be unresolved. An installed framework
masked this during local R CMD check; enabling R's declared-package xref check
reported the undeclared dependency. Replaced the reference with the published
release-workflow guide, whose page includes the UTF-8 rationale.

The regression builds the actual source tarball, checks its contents, extracts
it, and runs R's Rd resolver with declared-package checks enabled. Before the
fix it failed both the maintainer-file and help-link assertions. The first test
attempt also used an unsupported `info` argument to `expect_length`; switching
to `expect_identical` exposed the intended Rd failure directly.

The initial built-package check failed inside the new test because R's Rd
resolver scans all recommended-package help, while R CMD check shadows
undeclared test dependencies with dummy packages. `MASS` was installed in
`.Library` but hidden by that test library. The artifact test now temporarily
puts `.Library` first for the metadata scan; declared-package link checking
remains enabled. This does not add a runtime dependency on recommended packages.
