# gitweb

> 💡 This README is in a 🚧 WIP 🚧 status like all the code in this repository.
> Some of the flags and options are subject to change in the future.
> Ideas are welcome. Ideas are bulletproof (V).

`gitweb` is a command line interface I created mainly to learn Rust.

## Usage

`gitweb` will by default open the remote in the browser of the current
repository.

```
gitweb 0.1.0
Yoann Fleury <yoann.fleury@yahoo.com>
Open the current remote repository in your browser

USAGE:
    gitweb.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Set the verbosity of the command

OPTIONS:
    -b, --branch <branch>      Set the branch
    -B, --browser <browser>    Set the browser
```

## --branch

`gitweb` will open the current branch on the remote repository. You can override
the behavior by giving the `--branch` flag with the custom branch you want to
open in the browser.

## --browser

`gitweb` tries to start one of the following browser (in that order):

**Linux**:

- `$BROWSER` (this is a non standard variable)
- firefox
- google-chrome-stable

**Windows**:

- `%BROWSER%` (this is a non standard variable)

The order is defined by the author. If you want a custom browser, set the
`$BROWSER` environment variable for your user.

# TODO

- [ ] handle behavior when no remote origin is set.
- [ ] add an option `--help` to tell the user how to use the command line
- [ ] add `--branch` option (I think that C. Delafarge) did a good talk about
      Rust, and how to handle parameters at Devoxx 2018.
- [ ] add an option `--file` to specify the file to open
