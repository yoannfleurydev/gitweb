# gitweb

![Publish](https://github.com/yoannfleurydev/gitweb/workflows/Publish/badge.svg)

> Some of the flags and options are subject to change in the future.
> Ideas are welcome. Ideas are bulletproof (V).

`gitweb` is a command line interface I created mainly to learn Rust.

![preview](./docs/gitweb.gif)

## Intallation

### 🍺 The homebrew way

```sh
brew install yoannfleurydev/gitweb/gitweb
# or
brew tap yoannfleurydev/gitweb
brew install gitweb
```

### 📦 The Cargo way

```sh
cargo install gitweb
```

### ⚙️ The binary way

Download the binary from the [latest release](https://github.com/yoannfleurydev/gitweb/releases/latest) and put it in your PATH.

### 🖥 The MacPorts way

```
sudo port selfupdate
sudo port install gitweb
```

## Shell completion

`gitweb` completes its options on <kbd>TAB</kbd>, as well as the branches, tags
and remotes of the current repository (`gitweb --branch <TAB>`). Add the line
for your shell to its configuration. The completion is generated at startup, so
it stays in sync when you upgrade `gitweb`.

```sh
# Bash
echo 'source <(COMPLETE=bash gitweb)' >> ~/.bashrc

# Zsh
echo 'source <(COMPLETE=zsh gitweb)' >> ~/.zshrc

# Fish
echo 'COMPLETE=fish gitweb | source' >> ~/.config/fish/completions/gitweb.fish

# Elvish
echo 'eval (E:COMPLETE=elvish gitweb | slurp)' >> ~/.elvish/rc.elv
```

```powershell
# PowerShell
echo '$env:COMPLETE = "powershell"; gitweb | Out-String | Invoke-Expression; Remove-Item Env:\COMPLETE' >> $PROFILE
```

## Usage

`gitweb` will by default open the remote in the browser of the current
repository.

```plain
Open the current remote repository in your browser

Usage: gitweb [OPTIONS]

Options:
  -b, --branch <BRANCH>    Set the branch (alias for --tag)
  -t, --tag <TAG>          Set the tag (alias for --branch)
  -M, --merge-request      Set the merge request flag
  -c, --commit <COMMIT>    Set a commit
  -B, --browser <BROWSER>  Set the browser [env: BROWSER=]
  -r, --remote <REMOTE>    Set the remote
  -v, --verbose            Set the verbosity of the command
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version
```

## --branch, --tag

`gitweb` will open the current branch or tag on the remote repository. You can
override the behavior by giving either `--branch` or `--tag` flag with the
custom branch or tag you want to open in the browser.

## --browser

`gitweb` tries to start one of the following browser (in that order of priority):

- `--browser` option given in the command line
- `$BROWSER` on Linux 🐧 or `%BROWSER%` on Windows 🏁 (this is a non standard variable)
- the default web browser on the system

## --commit

`gitweb` will open the commit given as a parameter on the remote repository.

## --remote

`gitweb` will open the origin remote if it exists. You can override the behavior
by giving the `--remote` flag with the custom remote you want to open.

The remote is the name of one of the git remotes of the repository (`git remote`
lists them, usually `origin`, and `upstream` when working on a fork). It does
not change the shape of the URL that is opened, see [Custom URLs](#custom-urls)
for that.

## Custom URLs

`gitweb` knows the URLs of github.com, gitlab.com, bitbucket.org and gitea.io,
and opens GitHub-like URLs for every other host. For a self-hosted forge with
other URLs, set URL templates for its host in your git config:

```shell
git config --global gitweb.<host>.branch-template '<template>'
git config --global gitweb.<host>.tag-template '<template>'
git config --global gitweb.<host>.commit-template '<template>'
git config --global gitweb.<host>.merge-request-template '<template>'
```

Here is the configuration for a Bitbucket Server (or Data Center), that works
with both the SSH and the HTTPS remotes:

```shell
git config --global gitweb.bitbucket.company.com.branch-template 'https://{host}/projects/{owner}/repos/{repo}/browse?at=refs%2Fheads%2F{branch_encoded}'
git config --global gitweb.bitbucket.company.com.tag-template 'https://{host}/projects/{owner}/repos/{repo}/browse?at=refs%2Ftags%2F{tag_encoded}'
git config --global gitweb.bitbucket.company.com.commit-template 'https://{host}/projects/{owner}/repos/{repo}/commits/{commit}'
git config --global gitweb.bitbucket.company.com.merge-request-template 'https://{host}/projects/{owner}/repos/{repo}/pull-requests'
```

The placeholders of the templates are:

| Placeholder        | Template          | Value                                                  |
| ------------------ | ----------------- | ------------------------------------------------------ |
| `{host}`           | all               | the host of the remote, without the port               |
| `{repository}`     | all               | the path of the repository, like `group/subgroup/repo` |
| `{owner}`          | all               | the part of the path just before the repository name   |
| `{repo}`           | all               | the repository name                                    |
| `{branch}`         | `branch-template` | the branch                                             |
| `{branch_encoded}` | `branch-template` | the branch, percent-encoded (`/` becomes `%2F`)        |
| `{tag}`            | `tag-template`    | the tag                                                |
| `{tag_encoded}`    | `tag-template`    | the tag, percent-encoded (`/` becomes `%2F`)           |
| `{commit}`         | `commit-template` | the commit                                             |

Write `{{` for a `{` that is not a placeholder. A template is a full URL, so it
can use `http://` or a port. The host is the one written in the remote: with an
SSH alias (`git@work:org/repo.git`), the config is `gitweb.work.*` and the
template has to contain the real host instead of `{host}`.

A template always wins over the built-in URL, even for github.com, and the
built-in URL is used when no template is set for what is opened. The templates
can be set per repository with `git config --local`. Run `gitweb --verbose` to
see the template that is used.
