# Configure drift checks silently accepted outdated build logic

Attempted: validate a retained `configure.ac` containing all three historical
markers, and detect an arbitrary change to a freshly generated template.

Failure: both warning assertions failed. The checker treated both files as
current because they contained `CARGO_STATICLIB_NAME`, `AC_CONFIG_AUX_DIR`, and
`CARGO_TARGET_DIR`.

Root cause: the presence of old feature markers cannot establish that the rest
of the build-system template is current. The local-checkout helper also advised
an upgrade without `configure_ac = TRUE`, which retains the same old file.

Fix: compare the complete retained file with the current template after its
package-name substitution. Warn on any difference, explicitly acknowledging
custom edits, and preserve the file. The local-checkout advice now explicitly
requests configure replacement. See #1406.

Layout verification also caught two problems with using `template_path()`:
a prior monorepo scaffold could leave session state pointing at a directory
without a top-level `configure.ac`, and a current monorepo package could be
compared against the standalone template. Select the template from the target
package layout instead, and verify both layouts without changing session state.
