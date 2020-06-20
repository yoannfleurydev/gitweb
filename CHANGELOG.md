# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.13] - 2020-06-20

### Added

- add support for `--commit` to open a specific commit by [@rubenrua](https://github.com/rubenrua)
- add alias `--tag` for `--branch` as they are the same from git host provider perspective

## [0.1.12] - 2020-05-16

### Changed

- rework regex to get domain and project path

## [0.1.11] - 2020-04-26

### Fixed

- the remove port function was a bit to restrictive (it removed the first part of the path like _/path_removed/the/rest/of/the/path_)

### Security

- upgrade dependencies

## [0.1.8] - 2019-07-19

### Added

- add empty string option to --browser so the user can only have the output in the console

## [0.1.7] - 2019-07-08

### Fixed

- remove the port from the url to fix the 404 (#3)

## [0.1.6] - 2019-07-02

### Added

- no more panic 💥, now the program exit smoothly on errors
- each error has its own code
- [yoannfleurydev.github.io/gitweb](https://yoannfleurydev.github.io/gitweb)

### Changed

- add more comment for the `--help` option.
- add `print` function to output easily when the program is in error
- renamed the old `print` method to `verbose_print` so the logger write onlyon verbose run
- improve browser openning readability by removing ifs

### Security

- fix all dependency on their minor release to have the latest ones

## [0.1.5] - 2019-07-01

### Fixed

- the program will give back shell prompt when the browser is not already running (#2)

## [0.1.4] - 2019-03-04

### Fixed

- now able to use gitweb in git repository subdirectories

## [0.1.3] - 2019-02-18

### Added

- working CI

## [0.1.2] - 2019-02-17

### Added

- editorconfig
- git2 library to use git wrapper instead of system command

### Removed

- custom commands to get git information

## [0.1.1] - 2019-02-16

### Added

- this changelog
- build status

### Changed

- set default browser as the first to be open
- allow `$BROWSER` to override the default browser of the system
- allow `--browser` to override the `$BROWSER` environment variable and the default browser

## [0.1.0] - 2019-02-13

### Added

- default behavior of the command is to open the current repository in the browser
- add `--branch` option to open a custom branch (default behavior is the current branch of the repo)
- add `--browser` to open a custom browser

[0.1.4]: https://github.com/yoannfleurydev/gitweb/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/yoannfleurydev/gitweb/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/yoannfleurydev/gitweb/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/yoannfleurydev/gitweb/compare/v0.1.0...v0.1.1
