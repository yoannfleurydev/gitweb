# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2019-02-18

### Added

-   working CI

## [0.1.2] - 2019-02-17

### Added

-   editorconfig
-   git2 library to use git wrapper instead of system command

### Removed

-   custom commands to get git information

## [0.1.1] - 2019-02-16

### Added

-   this changelog
-   build status

### Changed

-   set default browser as the first to be open
-   allow `$BROWSER` to override the default browser of the system
-   allow `--browser` to override the `$BROWSER` environment variable and the default browser

## [0.1.0] - 2019-02-13

### Added

-   default behavior of the command is to open the current repository in the browser
-   add `--branch` option to open a custom branch (default behavior is the current branch of the repo)
-   add `--browser` to open a custom browser
