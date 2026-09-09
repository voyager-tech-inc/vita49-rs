# Changelog
<!--
SPDX-FileCopyrightText: 2025 The vita49-rs Authors

SPDX-License-Identifier: MIT OR Apache-2.0
-->

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-09-09

### Added

- Add missing CIF manipulators
- Add missing accessor macro for `ephemeris_ref_id`

### Fixed

- [**breaking**] Fix logic inversion bug on packet type helpers
- Return parsing errors rather than panics
- Fix off-by-one bug in Window Type subfield
- Mask OUI value in setter to avoid clobbering upper bits
- Account for padding in size_words()
- Remove ActionMode::Reserved variant
- Fix undercounting bug in ACK packet size calculation
- Fix incorrect attribute counting in CIF7
- Use as_mut() to modify value instead of temp value
- Fix incorrect CIF7 field setter in macro
- Drop async tag list if length changes
- Swap padding bytes as well when using little-endian
- Set stream ID `None` to 0 for context/command packets
- Fix Averaging Type subfield representation
- Update nats control examples url

## [1.2.0] - 2026-07-22

### Added

- Public construction API for `FormattedGps` (`Default` plus getters/setters for the
  Manufacturer OUI, TSI/TSF, position-fix timestamps, and radix-encoded geolocation values)
- Public construction API for `EcefEphemeris` (`Default` plus getters/setters for the
  Manufacturer OUI, TSI/TSF, position-fix timestamps, and the radix-encoded ECEF position,
  attitude, and velocity)
- Public construction API for `GpsAscii` (`Default` plus getters/setters for the
  Manufacturer OUI and the ASCII sentence payload — `set_sentences` validates ASCII and
  `sentences` returns a `Result`, per Rule 9.4.7-4)
- Public construction API for `ContextAssociationLists` (`Default` plus getters/setters for
  the Source, System, Vector-Component, and Asynchronous-Channel lists — and the
  Asynchronous-Channel Tag list — keeping each list-size subfield in sync per §9.13.2)

### Fixed

- Multiple instances of implicit sign-extension incorrectly setting bits outside of the
  target field.
- Fixed incorrect bit shift in the window time delta interpretation setter method.

## [1.1.0] - 2026-05-12

### Added

- Exposed mutable `Trailer` API with field setters and exports
- Added indicator bit toggle API

### Changed

- Made `SampleFrameIndicator` public
- Improved some docs and tests

## [1.0.0] - 2026-04-13

### Added

- Example pcap reader program.

### Changed

- BREAKING CHANGE: added zero-copy path for signal data payload manipulations.
- Unpinned certain dependencies to allow wider range of MSRV-compatible dep combinations.

## [0.1.0] - 2026-01-12

With the 0.1.0 release, APIs are considered stable and semver will
change accordingly.

### Added

- Example of Python FFI (#17)

### Changed

- Fixed incorrect `if_ref_freq` type (#16)

## [0.0.6] - 2025-10-01

### Changed

- Fixed buggy ACK generator methods (#12)

## [0.0.5] - 2025-07-10

### Changed

- Fixed `class_id` indicator bit being set improperly (#10)

## [0.0.4] - 2025-05-08

### Added

- Example of NATS command & control flow

### Changed

- Major rework of command packet processing [#6]

### Removed

- Visibility of certain CAM field manipulators limited:
  - `{enable,disable}_{controllee,controller}()`
  - `{,set_}{controllee,controller}_{id,uuid}_format()`
  - Instead, these are derived from `set_{controllee,controller}_id()`.
- CIF accessor methods for control packets have been moved to a sub-payload
  of command packets. So, you'll need to unwrap the payload. Your code update
  may look like:
```diff
-    command.set_bandwidth_hz(bw_hz);
-    command.set_rf_ref_freq_hz(freq_hz);
-    command.set_sample_rate_sps(sr_hz);
+    let control = command.payload_mut().control_mut().unwrap();
+    control.set_bandwidth_hz(bw_hz);
+    control.set_rf_ref_freq_hz(freq_hz);
+    control.set_sample_rate_sps(sr_hz);
```

## [0.0.3] - 2025-04-07

### Added

- Full support for CIF1 threshold field

### Changed

- Fixed various bit arithmetic bugs
- Fixed build error in benchmark app
- Various small CI fixes/improvements

### Removed

- `window_time_delta_ns()` accessor removed - replaced by `window_time_delta()`
- Visibility of `set_tsi()` and `set_tsf()` methods limited
- Binary test data replaced by JSON representations

## [0.0.2] - 2025-03-14

### Added

- Initial crate release.
- Basic documentation, test, and examples.

[2.0.0]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/2.0.0
[1.2.0]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/1.2.0
[1.1.0]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/1.1.0
[1.0.0]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/1.0.0
[0.1.0]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.1.0
[0.0.6]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.6
[0.0.5]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.5
[0.0.4]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.4
[0.0.3]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.3
[0.0.2]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.2
[0.0.1]: Unreleased

