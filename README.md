# `miniextendr`

## Overview

- `rsys` generates the rust bindings to the C-facilities in R.
- `rapi` defines a rust-friendly interface to R.
- `xtask` is a CLI crate that facilitates developer routines. See [matklad/cargo-xtask](https://github.com/matklad/cargo-xtask) for more information.
  For implemented task, run `cargo xtask --help` to see available commands.
- `rapi-macros` (???)

## Contributor

- Use `cargo xtask copy-r-headers` to have the current used R headers for the bindings in `rsys` available for lookup.

## Installation / Requirements

### System requirements

```shell
scoop install r rtools llvm
```

Define Environment variables `R_HOME` and `MINGW_ROOT`.

```powershell
$env:R_HOME = "%USERPROFILE%\scoop\apps\r\current"
# $env:MINGW_ROOT = "C:\rtools43\x86_64-w64-mingw32.static.posix" // usually here
$env:MINGW_ROOT = "%USERPROFILE%\scoop\apps\rtools\current\x86_64-w64-mingw32.static.posix"
```

For `cargo`, you need to ensure that the following are set

```toml
[build]
target = "x86_64-pc-windows-gnu"

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32.static.posix-gcc.exe"
```

Typically this lives in `.cargo/config.toml`, see [The Cargo Book: Configuration](https://doc.rust-lang.org/cargo/reference/config.html).

This this requires

```
%MINGW_ROOT%\x86_64-w64-mingw32.static.posix\bin
```

to be available on `PATH`.

**REVISE THIS**
Also the path should contain

```
%USERPROFILE%/scoop/apps/r/current\bin\x64
%USERPROFILE%\scoop\apps\rtools\current\x86_64-w64-mingw32.static.posix\bin
%USERPROFILE%\scoop\apps\rtools\current\usr\bin
```

## Setup environment

3. Copy your `R/include` and `R/bin` directories to `miniextendr/rsys/r`.
  Use `cargo xtask copy-r-headers` to copy these over.

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
