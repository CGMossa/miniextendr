# `xtask`


### `libgcc_mock`

It is not possible to make `xtask` create the mock `libgcc` mock work, because the `linker` needs this before it may compile the `xtask` whose supposed to solve this issue in the first place.

Instead this is solved by having a `build.rs` that does this for `xtask`, and then `xtask` can be used to set this further.
