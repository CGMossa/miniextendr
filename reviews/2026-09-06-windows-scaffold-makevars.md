# Scaffolded packages omitted Windows linker settings

Attempted: scaffold standalone and monorepo R packages, check their Windows
makefiles, and protect customized linker settings during an upgrade.

Failure: the R scaffolder did not create `src/Makevars.win` in either layout;
the CLI binary failed the same file-presence assertion. The upgrade guard also
accepted a modified `src/Makevars.win` without reporting uncommitted changes.

Root cause: canonical Windows templates were bundled and drift-checked, but
neither the R helper nor the CLI scaffold plan wrote them. Consequently R could
fall back to the generic Makevars without the Windows system libraries.

Fix: write the existing canonical template through both scaffolders and include
it in the upgrade dirty-file guard. Cover both layouts, replacement of stale
copies, and preservation of uncommitted Windows linker customization. See #1361.

The first local `clippy_all` reproduction did not run: the temporary workflow
extractor matched a `--features full` example in a comment, including its closing
backtick. Restricting the extractor to the actual command line selected the
curated CI feature list; this was a verification-script error, not a code failure.
