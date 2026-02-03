# Shell Completions

Enable tab completion for redisctl commands.

## Generate Completions

```bash
redisctl completions <shell>
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`

## Installation

=== "Bash"

    ```bash
    # Create completions directory if needed
    mkdir -p ~/.local/share/bash-completion/completions

    # Generate and install
    redisctl completions bash > ~/.local/share/bash-completion/completions/redisctl

    # Reload shell or source the file
    source ~/.local/share/bash-completion/completions/redisctl
    ```

=== "Zsh"

    ```bash
    # Create completions directory if needed
    mkdir -p ~/.zfunc

    # Add to fpath (add this to ~/.zshrc)
    fpath=(~/.zfunc $fpath)

    # Generate completions
    redisctl completions zsh > ~/.zfunc/_redisctl

    # Rebuild completion cache
    rm -f ~/.zcompdump; compinit
    ```

=== "Fish"

    ```bash
    # Generate and install
    redisctl completions fish > ~/.config/fish/completions/redisctl.fish

    # Reload shell
    source ~/.config/fish/completions/redisctl.fish
    ```

=== "PowerShell"

    ```powershell
    # Add to profile
    redisctl completions powershell >> $PROFILE

    # Reload profile
    . $PROFILE
    ```

## Usage

After installation, press `Tab` to complete:

```bash
# Complete commands
redisctl ent<Tab>
# → redisctl enterprise

# Complete subcommands
redisctl enterprise cl<Tab>
# → redisctl enterprise cluster

# Complete options
redisctl enterprise cluster get --<Tab>
# → --output  --query  --profile  ...
```

## Homebrew Users

If you installed via Homebrew, completions may be automatically available. If not:

```bash
# Bash
echo 'source $(brew --prefix)/etc/bash_completion.d/redisctl' >> ~/.bashrc

# Zsh
echo 'source $(brew --prefix)/share/zsh/site-functions/_redisctl' >> ~/.zshrc
```

## Troubleshooting

### Completions Not Working

1. Verify the file was created:
   ```bash
   ls -la ~/.local/share/bash-completion/completions/redisctl
   ```

2. Check your shell's completion system is enabled:
   ```bash
   # Bash - add to ~/.bashrc
   if [ -f /etc/bash_completion ]; then
     . /etc/bash_completion
   fi
   ```

3. Restart your shell or source the completion file.

### Zsh: Command Not Found: compinit

Add to your `~/.zshrc`:
```bash
autoload -Uz compinit
compinit
```
