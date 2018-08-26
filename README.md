rustup-find
===========

A Rust binary that automatically finds the latest version of Rust that has all
the currently installed components.

## Usage
```
rustup-find 0.1.2
Grégoire Geis <git@gregoirege.is>
  Use rustup to automatically find and/or install the latest Rust version that
  supports all the currently installed components.

USAGE:
    rustup-find.exe [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help              Prints help information
    -n, --no-colors         Whether colors should be disabled.
    -q, --quiet             Whether nothing should be logged.
    -s, --skip-installed    Do not try to install already installed components.
    -V, --version           Prints version information
    -v, --verbose           Whether we should log more informations than needed.

OPTIONS:
    -c, --components <components>...
            Space-separated list of components that must be available for a
            release to be considered valid.
    -d, --days <days>
            Number of days to check starting at the given offset. [default: 30]

    -o, --offset <offset>
            Number of days before today at which to start checking. [default: 0]

    -b, --rustup-bin <rustup_bin>
            Path to the Rustup binary. [default: rustup]

    -r, --rustup-dir <rustup_dir>
            Path to the Rustup config directory. [default: ~/.rustup]

    -t, --toolchain <toolchain>         Target toolchain.

SUBCOMMANDS:
    find       Find the latest available release that matches the current
               components.
    install    Find, download and install the latest available release that
               matches the current components.
    replace    Find and download the latest available release that matches
               the current components, and replace the given toolchain by
               the newly downloaded one.
```

If the `toolchain` is not provided, it will be resolved using `rustup toolchain list | grep default`.

## Examples

As a `x86_64-pc-windows-gnu` user, releases of `rls-preview` are quite rare,
which is why I created this app.

Here are a different examples of how this binary can be used.


### Install latest release that matches the current default toolchain and its components
```bash
# This command will:
#  - Find the correct release.
#  - Install it using `rustup toolchain install`.
#  - Move the newly installed toolchain from `channel-date-target` to `channel-target`,
#    overriding the previous one.
$ rustup-find --verbose --offset 25 replace

[i] Channel: nightly.
[i] Target: x86_64-pc-windows-gnu.
[i] Required components: cargo, rls-preview, rust-analysis, rust-docs, rust-mingw, rust-std, rustc.
[i] The following component was missing in 2018-07-21: rls-preview.
[i] The following component was missing in 2018-07-20: rls-preview.
[+] Found valid toolchain: nightly-2018-07-19-x86_64-pc-windows-gnu.
[i] Installing toolchain...
[+] Installed toolchain nightly-2018-07-19-x86_64-pc-windows-gnu.
[i] Replacing previous toolchain nightly-pc-windows-gnu...
[+] Replaced previous toolchain nightly-pc-windows-gnu by nightly-2018-07-19-x86_64-pc-windows-gnu.
```

### Install latest release that matches the current default toolchain and its components
```bash
# This command will:
#  - Find the correct release.
#  - Install it using `rustup toolchain install`.
$ rustup-find install

[+] Found valid toolchain: nightly-2018-07-19-x86_64-pc-windows-gnu.
[+] Installed toolchain nightly-2018-07-19-x86_64-pc-windows-gnu.
```

### Find latest release that matches the given toolchain and its components
```bash
# This command will:
#  - Find the correct release, and return it.
$ rustup-find --toolchain nightly-x86_64-pc-windows-msvc

nightly-2018-08-17-x86_64-pc-windows-msvc
```

### Example failure
```bash
$ rustup-find --days 5 --verbose

[i] Channel: nightly.
[i] Target: x86_64-pc-windows-gnu.
[i] Required components: cargo, rls-preview, rust-analysis, rust-docs, rust-mingw, rust-std, rustc.
[i] The following component was missing in 2018-08-26: rls-preview.
[i] The following component was missing in 2018-08-25: rls-preview.
[i] The following component was missing in 2018-08-24: rls-preview.
[i] No components were available in 2018-08-23.
[i] No components were available in 2018-08-22.
[-] Could not find a match in the last 5 days.
```
