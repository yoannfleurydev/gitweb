# gitweb

`gitweb` is a command line interface I created mainly to learn Rust.

## Usage

`gitweb` will by default open the remote in the browser of the current
repository.

`gitweb` tries to start one of the following (in that order):

**Linux**:

- `$BROWSER` (this is a non standard variable)
- firefox
- google-chrome-stable

**Windows**:

🚧 WIP 🚧

The order is defined by the author. If you want a custom browser, set the
`$BROWSER` environment variable for your user.

# TODO

- [ ] handle behavior when no remote origin is set.
- [ ] add an option `--help` to tell the user how to use the command line
- [ ] add `--branch` option (I think that C. Delafarge) did a good talk about
      Rust, and how to handle parameters at Devoxx 2018.
- [ ] add an option `--file` to specify the file to open
