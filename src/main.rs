#![deny(warnings)]
#![cfg_attr(feature = "cargo-clippy", deny(clippy))]

extern crate chrono;
extern crate dirs;
extern crate reqwest;
#[macro_use]
extern crate structopt;
extern crate termcolor;

use std::io::Write;
use std::path::PathBuf;

use structopt::{clap::AppSettings, StructOpt};
use termcolor::{Color, ColorChoice, ColorSpec, WriteColor};


#[derive(StructOpt)]
#[structopt(
    raw(global_settings = "&[AppSettings::DisableHelpSubcommand,
                             AppSettings::InferSubcommands,
                             AppSettings::VersionlessSubcommands]")
)]
struct Args {
    /// Whether we should log more informations than needed.
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Whether nothing should be logged.
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// Whether colors should be disabled.
    #[structopt(short = "n", long = "no-colors")]
    no_colors: bool,

    /// Number of days to check starting at the given offset.
    #[structopt(short = "d", long = "days", default_value = "30")]
    days: usize,

    /// Number of days before today at which to start checking.
    #[structopt(short = "o", long = "offset", default_value = "0")]
    offset: usize,

    /// Path to the Rustup binary.
    #[structopt(short = "b", long = "rustup-bin", default_value = "rustup", parse(try_from_str = "parse_path"))]
    rustup_bin: PathBuf,

    /// Path to the Rustup config directory.
    #[structopt(short = "r", long = "rustup-dir", default_value = "~/.rustup", parse(try_from_str = "parse_path"))]
    rustup_dir: PathBuf,

    /// Target toolchain.
    #[structopt(short = "t", long = "toolchain", parse(try_from_str = "parse_toolchain"))]
    toolchain: Option<(String, String)>,

    /// Components that must be available for a release to be considered valid.
    #[structopt(short = "c", long = "components")]
    components: Vec<String>,

    /// Do not try to install already existing components.
    #[structopt(short = "s", long = "skip")]
    skip_components: bool,

    /// Command.
    #[structopt(subcommand)]
    command: Option<Cmd>
}

#[derive(Copy, Clone, PartialEq, Eq, StructOpt)]
enum Cmd {
    /// Find the latest available release that matches the current components.
    #[structopt(name = "find")]
    Find,

    /// Find, download and install the latest available release that matches the current components.
    #[structopt(name = "install")]
    Install,

    /// Find and download the latest available release that matches the current components,
    /// and replace the given toolchain by the newly downloaded one.
    #[structopt(name = "replace")]
    Replace {
        /// Keep previous directory as '[old-name]-old'.
        #[structopt(short = "k", long = "keep-previous")]
        keep_old: bool
    }
}

#[cfg_attr(feature = "cargo-clippy",
           allow(cyclomatic_complexity))] // Allows us to have macros that use the parsed arguments.
#[cfg_attr(feature = "cargo-clippy",
           allow(write_literal))] // Necessary for the status! macro.
fn main() {
    let Args {
        command,
        mut components,
        days,
        no_colors,
        offset,
        quiet,
        rustup_bin,
        mut rustup_dir,
        skip_components,
        toolchain, 
        verbose,
    } = Args::from_args();

    let colors = if no_colors { ColorChoice::Never } else { ColorChoice::Auto };

    macro_rules! status {
        (error, $( $arg: expr ),+ ) => (
            status!(termcolor::StandardStream::stderr(colors), Color::Red,    "[-] ", $($arg),+)
        );
        (warning, $( $arg: expr ),+ ) => (
            status!(termcolor::StandardStream::stderr(colors), Color::Yellow, "[!] ", $($arg),+)
        );
        (info, $( $arg: expr ),+ ) => (
            if verbose {
                status!(termcolor::StandardStream::stdout(colors), Color::Blue,   "[i] ", $($arg),+)
            }
        );
        (success, $( $arg: expr ),+ ) => (
            status!(termcolor::StandardStream::stdout(colors), Color::Green,  "[+] ", $($arg),+)
        );

        ( $out: expr, $color: expr, $( $arg: expr ),+ ) => (
            if !quiet {
                let mut out = $out;
                let mut _has_color = true;

                $(
                    if _has_color {
                        _has_color = false;
                        let _ = out.set_color(ColorSpec::new().set_fg(Some($color)));
                    } else {
                        _has_color = true;
                        let _ = out.reset();
                    }

                    let _ = write!(out, "{}", $arg);
                );+

                let _ = out.write(b"\n");
            }
        )
    }

    macro_rules! fail {
        ( $status: expr, $( $arg: expr ),+ ) => ({
            status!(error, $( $arg ),+);
            std::process::exit($status)
        });
    }

    macro_rules! rustup {
        (output, $( $arg: expr ),+ ) => ({
            let result = rustup!( $( $arg ),+ );

            if !result.status.success() {
                if !quiet {
                    use std::fmt::Write;

                    let mut bin = rustup_bin.to_str().unwrap().to_string();

                    $(
                        let _ = write!(bin, " {}", $arg);
                    );+

                    eprintln!("Failed to execute \"{:#?}\":", bin);
                    std::io::stderr().write_all(&result.stderr).is_ok();
                }

                std::process::exit(1)
            }

            match String::from_utf8(result.stdout) {
                Ok(output) => output,
                Err(err)   => fail!(2, "Failed to convert output of Rustup command: ", err, '.')
            }
        });

        ( $( $arg: expr ),+ ) => (
            std::process::Command::new(&rustup_bin)
                .args(&[ $( $arg ),+ ])
                .output()
                .expect("Failed to spawn rustup process.")
        );
    }


    // Find channel & target
    let (channel, target) = match toolchain {
        Some(values) => values,
        None => {
            let output = rustup!(output, "toolchain", "list");

            match output.lines().find(|line| line.ends_with(" (default)")) {
                Some(line) => parse_toolchain(&line[..line.len() - 10]).unwrap(),
                None => fail!(3, "Could not find default toolchain.")
            }
        }
    };

    let toolchain = format!("{}-{}", channel, target);

    status!(info, "Channel: ", &channel, '.');
    status!(info, "Target: ", &target, '.');

    // Find needed components
    if !skip_components {
        let output = rustup!(output, "component", "list", "--toolchain", &toolchain);

        for line in output.lines() {
            if line.ends_with(" (default)") {
                let line = &line[.. line.len() - 10];

                if line.ends_with(&target) {
                    components.push(line[.. line.len() - target.len() - 1].to_string())
                } else {
                    components.push(line.to_string())
                }
            } else if line.ends_with(" (installed)") {
                let line = &line[.. line.len() - 12];

                if line.ends_with(&target) {
                    components.push(line[.. line.len() - target.len() - 1].to_string())
                } else {
                    components.push(line.to_string())
                }
            }
        }
    }

    // Filter unwanted components
    components.retain(|x| {
        x != "rust-src" && x != "rust-std-wasm32-unknown-unknown"
    });

    status!(info, "Required components: ", {
        use std::fmt::Write;

        let mut s = components[0].clone();

        for component in &components[1..] {
            let _ = write!(s, ", {}", component);
        }

        s
    }, ".");
    
    // Find latest version that matches the needed components
    let mut date = chrono::Utc::now() - chrono::Duration::days(offset as i64 - 1);

    let one_day = chrono::Duration::days(1);
    let start_date = date;

    let new_toolchain = 'main: loop {
        date = date - one_day;

        if start_date - date > chrono::Duration::days(days as _) {
            fail!(5, "Could not find a match in the last ", days, " days.");
        }

        let date_str = date.format("%Y-%m-%d");
        let url = format!("https://static.rust-lang.org/dist/{}/channel-rust-{}.toml",
                          date_str, channel);

        match reqwest::get(&url) {
            Ok(mut res) => {
                let text = match res.text() {
                    Ok(text) => text,
                    Err(_) => {
                        status!(error, "Cannot get toolchain for ", date_str, ".");
                        continue 'main
                    }
                };

                let mut lines = text.lines();
                let mut components = components.clone();
                
                'match_components: loop {
                    if let Some(line) = lines.next() {
                        if !line.starts_with("[pkg.") || !line.ends_with(']') {
                            continue
                        }

                        if let Some(next_line) = lines.next() {
                            if next_line != "available = true" {
                                continue
                            }
                        } else {
                            continue 'main
                        }

                        let mut i = 0;

                        while i < components.len() {
                            let remove = {
                                let component = &components[i];

                                line[5..].starts_with(component) && (
                                    (line.len() == 6 + component.len()) ||
                                    (
                                        line[5 + component.len() ..].starts_with(".target") &&
                                        line[13 + component.len() ..].starts_with(&target)
                                    )
                                )
                            };

                            if remove {
                                components.swap_remove(i);

                                if components.is_empty() {
                                    break 'match_components
                                }
                            } else {
                                i += 1;
                            }
                        }
                    } else {
                        status!(info, "Some components were missing in ", &date_str, "; trying previous day...");

                        continue 'main
                    }
                }

                // All components found, return date
                break format!("{}-{}-{}", channel, date_str, target)
            },
            Err(_) => continue
        }
    };

    let command = match command {
        None | Some(Cmd::Find) => {
            println!("{}", new_toolchain);

            std::process::exit(0);
        },

        Some(command) => command
    };

    // Install toolchain
    status!(success, "Found valid toolchain: ", &new_toolchain, ".");
    status!(info, "Installing toolchain...");

    let output = rustup!("toolchain", "install", &new_toolchain);

    if !output.status.success() {
        status!(error, "Could not install toolchain ", &new_toolchain, ":");
        
        let _ = std::io::stdout().write_all(&output.stdout);

        std::process::exit(6);
    }

    status!(success, "Installed toolchain ", &new_toolchain, ".");

    if let Cmd::Replace { keep_old } = command {
        status!(info, "Replacing previous toolchain ", &toolchain, "...");

        // Move toolchain directory
        rustup_dir.push("toolchains");

        if keep_old {
            let r = std::fs::rename(
                        rustup_dir.join(&toolchain),
                        rustup_dir.join(format!("{}-old", toolchain)));
            
            if r.is_err() {
                fail!(7, "Could not move previous toolchain ", &toolchain, " to new location.")
            }
        } else {
            let r = std::fs::remove_dir_all(rustup_dir.join(&toolchain));

            if r.is_err() {
                fail!(8, "Could not remove previous toolchain ", &toolchain, ".");
            }
        }

        let r = std::fs::rename(rustup_dir.join(&new_toolchain), rustup_dir.join(&toolchain));

        if r.is_err() {
            fail!(9, "Could not move toolchain ", &new_toolchain, " to new location ", &toolchain, ".");
        }

        // Move toolchain hash
        rustup_dir.pop();
        rustup_dir.push("update-hashes");

        if keep_old {
            let r = std::fs::rename(
                        rustup_dir.join(&toolchain),
                        rustup_dir.join(format!("{}-old", toolchain)));
            
            if r.is_err() {
                fail!(10, "Could not move previous hashes for toolchain ", &toolchain, " to new location.")
            }
        } else {
            let r = std::fs::remove_dir_all(rustup_dir.join(&toolchain));

            if r.is_err() {
                fail!(11, "Could not remove previous hashes for toolchain ", &toolchain, ".");
            }
        }

        let r = std::fs::rename(rustup_dir.join(&new_toolchain), rustup_dir.join(&toolchain));

        if r.is_err() {
            fail!(12, "Could not move hashes of toolchain ", &new_toolchain, " to new location ", &toolchain, ".");
        }

        status!(success, "Replaced previous toolchain ", &toolchain, " by ", &new_toolchain, ".");
    }
}

fn parse_toolchain(toolchain: &str) -> Result<(String, String), &'static str> {
    let hyphen = toolchain.find('-').ok_or("Invalid toolchain format.")?;

    Ok((toolchain[..hyphen].to_string(), toolchain[hyphen+1..].to_string()))
}

fn parse_path(path: &str) -> Result<PathBuf, &'static str> {
    if path.starts_with('~') {
        let home = dirs::home_dir().ok_or("Unable to resolve path.")?;

        Ok(home.join(&path[2..]))
    } else {
        Ok(PathBuf::from(path))
    }
}
