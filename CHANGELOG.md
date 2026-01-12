# Changelog
<!--
SPDX-FileCopyrightText: 2025 The vita49-rs Authors

SPDX-License-Identifier: MIT OR Apache-2.0
-->

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.7] - 2026-01-12

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

[0.0.6]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.6
[0.0.5]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.5
[0.0.4]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.4
[0.0.3]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.3
[0.0.2]: https://github.com/voyager-tech-inc/vita49-rs/releases/tag/0.0.2
[0.0.1]: Unreleased
