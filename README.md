rustup-find
===========

A rust binary that automatically finds the latest version of Rust that has all
the currently installed components.

## Usage
```
rustup-find 0.1.0
Grégoire Geis <git@gregoirege.is>
  Use rustup to automatically find and/or install the latest Rust version that
  supports all the currently installed components.

USAGE:
    rustup-find.exe [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help         Prints help information
    -n, --no-colors    Whether colors should be disabled.
    -q, --quiet        Whether nothing should be logged.
    -s, --skip         Do not try to install already existing components.
    -V, --version      Prints version information
    -v, --verbose      Whether we should log more informations than needed.

OPTIONS:
    -c, --components <components>...
            Components that must be available for a release to be considered
            valid.
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
