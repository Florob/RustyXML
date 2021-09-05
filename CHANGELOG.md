# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Order of attributes can now optionally be tracked.
  This can be enabled via the `ordered_attrs` feature.
### Changed
- Error types no longer implementat the deprecated `Error::description` method

## [0.3.0] - 2020-03-08
### Added
- Error types implement `Error::source`
### Changed
- Minimal Supported Rust Version is now 1.31.
- The `prefixes` and `default_ns` fields of `Element` are no longer public.
  These were always hidden from documentation and never meant for public consumption.
  Until now they had to be public due to language restrictions.

## [0.2.0] - 2020-03-08
### Changed
- License changed from MIT to MIT/Apache-2.0.
- `Element::new()` accepts any `IntoIterator` instead of a `Vec` for attributes.

## [0.1.1] - 2015-04-05
### Changed
- Update to then latest Rust master (this release predates Rust 1.0).

## [0.1.0] - 2015-04-01
- Initial release

[Unreleased]: https://github.com/Florob/RustyXML/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/Florob/RustyXML/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Florob/RustyXML/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Florob/RustyXML/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Florob/RustyXML/releases/tag/v0.1.0
