# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.16.0] - 2022-10-19

### Added

- Add annotations specific to the `nstack`

### Changed

- Update to use `ranno` annotations
- Update `microkelvin` from `0.13.0-rc.0` to `0.17`. The wide range is due to
  experimentation with `rkyv` in `microkelvin`
- Change `rust-toolchain` to `stable`

## [0.10.0] - 2021-07-27

### Changed

- Update `microlelvin` version to `0.10.0-rc` [#32]
- Change `persistance` by `persistence` for the feature name. [#32]

## [0.9.0] - 2021-07-02

### Added

- Add microkelvin persistance feature and integration

### Change

- Change all `unwrap`s in tests to `?`

## [0.8.0] - 2021-04-21

### Changed

- Change microkelvin and canonical versions to 0.6 and 0.7 respectively

## [0.7.0] - 2021-01-25

### Changed

- Change microkelvin and canonical versions to 0.5 and 0.6 respectively

## [0.6.1] - 2020-10-30

### Changed

- Change microkelvin version to `v0.5`

### Removed

- Remove unused canonical_host

## [0.6.0] - 2020-10-20

### Changed

- Change microkelvin and canonical to version 0.4

[Unreleased]: https://github.com/dusk-network/nstack/compare/v0.16.0...HEAD
[0.16.0]: https://github.com/dusk-network/nstack/compare/v0.10.0...v0.16.0
[0.10.0]: https://github.com/dusk-network/nstack/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/dusk-network/nstack/compare/v0.8.1...v0.9.0
[0.8.1]: https://github.com/dusk-network/nstack/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/dusk-network/nstack/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/dusk-network/nstack/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/dusk-network/nstack/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/dusk-network/nstack/releases/tag/v0.6.0
[#32]: https://github.com/dusk-network/nstack/issues/32
