# `miniextendr`

## Overview

- `rsys` generates the rust bindings to the C-facilities in R.
- `rapi` defines a rust-friendly interface to R.
- `xtask` is a CLI crate that facilitates developer routines. See [matklad/cargo-xtask](https://github.com/matklad/cargo-xtask) for more information.
  For implemented task, run `cargo xtask --help` to see available commands.

## Installation / Requirements

1. Copy your `R/include` and `R/bin` directories to `miniextendr/rsys/r`, i.e.

    ```shell
    Folder PATH listing for volume Windows
    Volume serial number is 7EFB-7332
    C:.
    ├───.cargo
    ├───.vscode
    ├───rsys
    │   ├───r
    │   │   ├───bin
    │   │   │   └───x64
    │   │   └───include
    │   │       └───R_ext
    │   └───src
    └───src
    ```
