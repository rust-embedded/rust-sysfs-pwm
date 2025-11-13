# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Added
- Add ability to read capture file descriptor #18
- Add functions for polarity get/set #16
- Add `get_duty_cycle` and `set_duty_cycle` methods #13
- Add getter for checking state of `enable`

### Fixed
- Fix addition of explicit `dyn`
- Handle errors in closure for function `with_exported` #14

### Removed
- Remove implementation of deprecated `Error::description` function

## [0.1.0] - 2016-03-02

- Initial release.

<!-- next-url -->
[Unreleased]: https://github.com/rust-embedded/rust-sysfs-pwm/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/rust-embedded/rust-sysfs-pwm/releases/tag/v0.1.0
