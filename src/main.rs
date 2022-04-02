
use std::{io::Write, error::Error};

use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    // Subcommands
    #[clap(subcommand)]
    command: Option<Commands>,

    // Some random flags 
    #[clap(long)]
    locked: bool,
    #[clap(long)]
    log_file: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// initialize buggyhelp for your project
    #[clap(disable_version_flag = true)]
    Init(InitArgs),

    /// Fetch the source of `$crate $version`
    #[clap(disable_version_flag = true)]
    Fetch(FetchArgs),

    /// Print --help as markdown (for generating docs)
    #[clap(disable_version_flag = true)]
    #[clap(hide = true)]
    HelpMarkdown(HelpMarkdownArgs),
}

#[derive(clap::Args)]
struct InitArgs {}

#[derive(clap::Args)]
struct FetchArgs {
    krate: String,
    version: String,
}

#[derive(clap::Args)]
struct HelpMarkdownArgs {}


fn main() -> Result<(), Box<dyn Error>>  {
    let cli = Cli::parse();

    let mut out = std::io::stdout();

    match &cli.command {
        Some(Commands::HelpMarkdown(..)) => cmd_help_markdown(&mut out)?,
        _ => unimplemented!(),
    }

    Ok(())
}

/// Perform crimes on clap long_help to generate markdown docs
fn cmd_help_markdown(
    out: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    // Make a new App to get the help message this time.

    writeln!(out, "# buggyhelp CLI manual")?;
    writeln!(out)?;
    writeln!(
        out,
        "> This manual can be regenerated with `buggyhelp help-markdown`"
    )?;
    writeln!(out)?;

    let mut full_command = Cli::command();
    let mut todo = vec![&mut full_command];
    let mut is_full_command = true;

    while let Some(command) = todo.pop() {
        let mut help_buf = Vec::new();
        command.write_long_help(&mut help_buf).unwrap();
        let help = String::from_utf8(help_buf).unwrap();

        // First line is --version
        let mut lines = help.lines();
        let version_line = lines.next().unwrap();
        let mut subcommand_name = format!("buggyhelp {} ", command.get_name());

        if is_full_command {
            writeln!(out, "Version: `{version_line}`")?;
            writeln!(out)?;
            subcommand_name = String::new();
        } else {
            // Give subcommands some breathing room
            writeln!(out, "<br><br><br>")?;
            writeln!(out, "## {}", subcommand_name)?;
        }

        let mut in_subcommands_listing = false;
        for line in lines {
            // Use a trailing colon to indicate a heading
            if let Some(heading) = line.strip_suffix(':') {
                if !line.starts_with(' ') {
                    // SCREAMING headers are Main headings
                    if heading.to_ascii_uppercase() == heading {
                        in_subcommands_listing = heading == "SUBCOMMANDS";

                        writeln!(out, "### {subcommand_name}{heading}")?;
                    } else {
                        writeln!(out, "### {heading}")?;
                    }
                    continue;
                }
            }

            if in_subcommands_listing && !line.starts_with("     ") {
                // subcommand names are list items
                let own_subcommand_name = line.trim();
                write!(
                    out,
                    "* [{own_subcommand_name}](#buggyhelp-{own_subcommand_name}): "
                )?;
                continue;
            }
            // The rest is indented, get rid of that
            let line = line.trim();

            // Usage strings get wrapped in full code blocks
            if line.starts_with("buggyhelp ") {
                writeln!(out, "```")?;
                writeln!(out, "{}", line)?;
                writeln!(out, "```")?;
                continue;
            }

            // argument names are subheadings
            if line.starts_with('-') || line.starts_with('<') {
                writeln!(out, "#### `{}`", line)?;
                continue;
            }

            // escape default/value strings
            if line.starts_with('[') {
                writeln!(out, "\\{}", line)?;
                continue;
            }

            // Normal paragraph text
            writeln!(out, "{}", line)?;
        }
        writeln!(out)?;

        todo.extend(command.get_subcommands_mut());
        is_full_command = false;
    }

    Ok(())
}