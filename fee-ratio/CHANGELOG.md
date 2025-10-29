# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.0] - 2025-10-29

sync version release with `sanctum-u64-ratio`

## [2.1.0] - 2025-10-24

### Changed

- Apply changes from PR #5

## [2.0.0] - 2025-09-29

### Breaking

- `Fee::new` now returns `None` if inner `Ratio`'s `d=0`. Use a `Ratio` with `n=0` to express zero fees instead.

## [1.0.0] - 2025-03-30

Initial release
