# Contributing to vita49-rs
<!--
SPDX-FileCopyrightText: 2025 The vita49-rs Authors

SPDX-License-Identifier: MIT OR Apache-2.0
-->

First of all, thank you for taking the time to contribute!

Any kind of contribution is welcome. This includes:

- [Reporting bugs](https://github.com/voyager-tech-inc/vita49-rs/issues)
- Fixing bugs
- Adding features
- Asking questions

If you have a question, feel free to open an issue. It doesn't need
to be an actual issue with the software - general questions about usage
and strategy are welcome as well.

## Merge Requirements

For new code to be merged, please make sure the following criteria are
met:

- `cargo test -p vita49` passes (see [testing](#testing) below).
    - If new features are added, please add documentation and tests!
- `cargo clippy` passes with no warnings.
- `cargo fmt` has been run.
- Any new files have correct SPDX headers compliant with [REUSE](https://reuse.software/spec-3.3/).
- Pull requests add commits that apply cleanly onto `main`.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
- GitHub Workflows pass.
- If the change is significant, add a line to the [`CHANGELOG.md`](CHANGELOG.md) file.

### Testing

This crate uses `cargo` for unit and integration testing. The various
example programs/demos in this repo do not have unit tests, but should
at least build. The individual demo READMEs have info on build requirements.

For some tests, we use [Wireshark](https://www.wireshark.org/) (>= 4.5.0)
to test parsing of the VRT packets we generate. If you want to run
tests without wireshark, set `SKIP_WIRESHARK_TESTS=1` in your
environment.

Beyond that, running `cargo test -p vita49` runs all the tests
necessary for merge.

If you are adding an API, please add a doc comment that includes
an example. These are automatically built and tested which helps
show users how to use the crate _and_ tests your API.

### Signing Your Contribution

We require that all contributors "sign-off" on their commits. This
certifies that the contribution is your original work, or you have rights
to submit it under the same license, or a compatible license.

Any contribution which contains commits that are not Signed-Off will
not be accepted.

* To sign off on a commit you simply use the `--signoff` (or `-s`) option when committing your changes:
  ```bash
  $ git commit -s -m "feat: add cool feature"
  ```
  This will append the following to your commit message:
  ```
  Signed-off-by: Your Name <your@email.com>
  ```

* Full text of the DCO:

  ```
    Developer Certificate of Origin
    Version 1.1

    Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
    1 Letterman Drive
    Suite D4700
    San Francisco, CA, 94129

    Everyone is permitted to copy and distribute verbatim copies of this license document, but changing it is not allowed.
  ```

  ```
    Developer's Certificate of Origin 1.1

    By making a contribution to this project, I certify that:

    (a) The contribution was created in whole or in part by me and I have the right to submit it under the open source license indicated in the file; or

    (b) The contribution is based upon previous work that, to the best of my knowledge, is covered under an appropriate open source license and I have the right under that license to submit that work with modifications, whether created in whole or in part by me, under the same open source license (unless I am permitted to submit under a different license), as indicated in the file; or

    (c) The contribution was provided directly to me by some other person who certified (a), (b) or (c) and I have not modified it.

    (d) I understand and agree that this project and the contribution are public and that a record of the contribution (including all personal information I submit with it, including my sign-off) is maintained indefinitely and may be redistributed consistent with this project or the open source license(s) involved.
  ```

## Copyright & License

Unless explicitly stated otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed under Apache-2.0 OR MIT license, without any additional terms or
conditions.

If you feel you've added significant or original code to this project,
please feel free to add you or your company's name to the [`AUTHORS.md`](AUTHORS.md)
file in the root of this repo.
