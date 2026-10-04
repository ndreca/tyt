/// The help for `integration completion print` with the command that installs
/// `bin`'s completions in each shell.
pub fn completion_install_help(bin: &str) -> String {
    let print = format!("{bin} integration completion print");
    format!(
        "Installing:
  bash        {print} bash > ~/.local/share/bash-completion/completions/{bin}
  elvish      echo 'eval ({print} elvish | slurp)' >> ~/.config/elvish/rc.elv
  fish        {print} fish > ~/.config/fish/completions/{bin}.fish
  powershell  Add-Content $PROFILE '{print} powershell | Out-String | Invoke-Expression'
  zsh         {print} zsh > ~/.zsh/completions/_{bin}

Each target directory has to exist. zsh also needs
fpath=(~/.zsh/completions $fpath) before compinit in ~/.zshrc. A new shell then
completes {bin} on Tab."
    )
}

#[cfg(test)]
mod tests {
    use crate::completion_install_help;

    #[test]
    fn the_help_installs_the_binary_completions_in_each_shell() {
        let help = completion_install_help("fs");

        let installs: Vec<_> = help.lines().skip(1).take(5).collect();
        assert_eq!(
            installs,
            [
                "  bash        fs integration completion print bash > ~/.local/share/bash-completion/completions/fs",
                "  elvish      echo 'eval (fs integration completion print elvish | slurp)' >> ~/.config/elvish/rc.elv",
                "  fish        fs integration completion print fish > ~/.config/fish/completions/fs.fish",
                "  powershell  Add-Content $PROFILE 'fs integration completion print powershell | Out-String | Invoke-Expression'",
                "  zsh         fs integration completion print zsh > ~/.zsh/completions/_fs",
            ]
        );
    }
}
