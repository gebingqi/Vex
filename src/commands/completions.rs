use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::{Shell, generate};
use std::io;

use crate::commands::Cli;

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Shell type to generate completions for.
    pub shell: clap_complete::Shell,
}

pub fn completions_command(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    eprintln!("Generating completion script for {}", bin_name);

    generate(shell, &mut cmd, bin_name.clone(), &mut io::stdout());

    match shell {
        Shell::Bash => print_bash_dynamic_completion(&bin_name),
        Shell::Zsh => print_zsh_dynamic_completion(&bin_name),
        Shell::Fish => print_fish_dynamic_completion(&bin_name),
        _ => {
            eprintln!(
                "# Note: Dynamic config name completion not yet implemented for {:?}",
                shell
            );
        }
    }

    Ok(())
}

fn print_bash_dynamic_completion(bin_name: &str) {
    println!(
        r#"
# === Dynamic configuration name completion ===
_vex_get_configs() {{
    {bin_name} list 2>/dev/null | grep ' - ' | awk '{{print $1}}'
}}

_vex_original=$(declare -f _vex)
eval "${{_vex_original/_vex/_vex_base}}"

_vex() {{
    local cur prev subcmd
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"

    if [[ ${{COMP_CWORD}} -ge 2 ]]; then
        subcmd="${{COMP_WORDS[1]}}"
        case "$subcmd" in
            exec|rm|edit)
                if [[ ${{COMP_CWORD}} -eq 2 ]]; then
                    COMPREPLY=( $(compgen -W "$(_vex_get_configs)" -- "${{cur}}") )
                    return 0
                fi
                ;;
            rename)
                if [[ ${{COMP_CWORD}} -eq 2 ]]; then
                    COMPREPLY=( $(compgen -W "$(_vex_get_configs)" -- "${{cur}}") )
                    return 0
                fi
                ;;
        esac
    fi

    _vex_base "$@"
}}

complete -F _vex {bin_name}
"#,
        bin_name = bin_name
    );
}

fn print_zsh_dynamic_completion(bin_name: &str) {
    println!(
        r#"
# === Zsh dynamic configuration name completion ===
_vex_configs() {{
    local configs
    configs=($({bin_name} list 2>/dev/null | grep ' - ' | awk '{{print $1}}'))
    _describe 'configurations' configs
}}

_vex() {{
    local line state
    local -a cmds

    _arguments -C \
        "1: :->cmds" \
        "*::arg:->args"

    case "$state" in
        cmds)
            cmds=(
                "save:Save QEMU configuration"
                "rename:Rename a saved QEMU configuration"
                "rm:Remove a saved QEMU configuration"
                "list:List all saved QEMU configurations"
                "print:Print details of a configuration"
                "exec:Execute a saved QEMU configuration"
                "edit:Edit a saved QEMU configuration"
                "completions:Generate shell completion scripts"
            )
            _describe -t commands 'vex command' cmds
            ;;
        args)
            case $words[1] in
                exec|rm|edit)
                    if (( CURRENT == 2 )); then
                        _vex_configs
                    fi
                    ;;
                rename)
                    if (( CURRENT == 2 )); then
                        _vex_configs
                    fi
                    ;;
            esac
            ;;
    esac
}}
"#,
        bin_name = bin_name
    );
}

fn print_fish_dynamic_completion(bin_name: &str) {
    println!(
        r#"
# === Fish dynamic configuration name completion ===
function __vex_configs
    {bin_name} list 2>/dev/null | grep ' - ' | awk '{{print $1}}'
end

complete -c vex -f
complete -c vex -n "__fish_seen_subcommand_from exec" -a "(__vex_configs)" -d "Configuration name"
complete -c vex -n "__fish_seen_subcommand_from rm" -a "(__vex_configs)" -d "Configuration name"
complete -c vex -n "__fish_seen_subcommand_from edit" -a "(__vex_configs)" -d "Configuration name"
complete -c vex -n "__fish_seen_subcommand_from rename; and not __fish_seen_subcommand_from (__vex_configs)" -a "(__vex_configs)" -d "Old configuration name"
"#,
        bin_name = bin_name
    );
}
