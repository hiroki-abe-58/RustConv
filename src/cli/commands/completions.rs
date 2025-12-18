//! Shell completions subcommand implementation

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::args::{Cli, CompletionsArgs};

/// Execute the completions subcommand
pub fn execute(args: CompletionsArgs) -> Result<()> {
    let mut cmd = Cli::command();
    let cmd_name = cmd.get_name().to_string();

    generate(args.shell, &mut cmd, cmd_name, &mut io::stdout());

    // Print installation instructions to stderr
    print_installation_instructions(args.shell);

    Ok(())
}

fn print_installation_instructions(shell: Shell) {
    eprintln!();
    eprintln!("# Installation instructions for {:?}:", shell);
    eprintln!();

    match shell {
        Shell::Bash => {
            eprintln!("# Add to ~/.bashrc:");
            eprintln!("# eval \"$(dtx completions bash)\"");
            eprintln!();
            eprintln!("# Or save to a file:");
            eprintln!("# dtx completions bash > ~/.local/share/bash-completion/completions/dtx");
        }
        Shell::Zsh => {
            eprintln!("# Add to ~/.zshrc (before compinit):");
            eprintln!("# eval \"$(dtx completions zsh)\"");
            eprintln!();
            eprintln!("# Or save to a file in your fpath:");
            eprintln!("# dtx completions zsh > ~/.zsh/completions/_dtx");
            eprintln!("# Then add to ~/.zshrc: fpath=(~/.zsh/completions $fpath)");
        }
        Shell::Fish => {
            eprintln!("# Save to fish completions directory:");
            eprintln!("# dtx completions fish > ~/.config/fish/completions/dtx.fish");
        }
        Shell::PowerShell => {
            eprintln!("# Add to your PowerShell profile:");
            eprintln!("# dtx completions powershell | Out-String | Invoke-Expression");
            eprintln!();
            eprintln!("# Or save to a file and source it in your profile:");
            eprintln!("# dtx completions powershell > dtx.ps1");
        }
        Shell::Elvish => {
            eprintln!("# Save to elvish completions directory:");
            eprintln!("# dtx completions elvish > ~/.elvish/lib/dtx.elv");
        }
        _ => {
            eprintln!("# Please refer to your shell's documentation for completion setup.");
        }
    }
    eprintln!();
}

