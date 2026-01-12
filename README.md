# VITA 49
<!--
SPDX-FileCopyrightText: 2025 The vita49-rs Authors

SPDX-License-Identifier: MIT OR Apache-2.0
-->

> [!WARNING]
> APIs should not be considered stable until the 0.1.0 release.

A crate for parsing and creating packets compatible with the
ANSI/VITA-49.2-2017 standard.

The [ANSI/VITA 49.2 standard](https://www.vita.com/page-1855484)
"defines a signal/spectrum protocol that expresses spectrum
observation, spectrum operations, and capabilities of RF devices."

In general, it's difficult to write software for VITA 49 as software
defined radios (SDRs) often choose different "flavors" of VITA 49.

This crate is:

 * **Flexible**: No need to recompile your program for
   different flavors of VITA 49.

 * **Fast**: Parsing is quick even while supporting
   various packet types.

 * **Easy**: Simply pass in some bytes and this crate will give
   you a data structure you can query, mutate, and write.

[![Crates.io](https://img.shields.io/crates/v/vita49)](https://crates.io/crates/vita49)
[![Docs.rs](https://img.shields.io/docsrs/vita49)](https://docs.rs/vita49)
[![Build Status](https://github.com/voyager-tech-inc/vita49-rs/workflows/ci/badge.svg)](https://github.com/voyager-tech-inc/vita49-rs/actions)
![License](https://img.shields.io/crates/l/vita49)
[![REUSE status](https://api.reuse.software/badge/github.com/voyager-tech-inc/vita49-rs)](https://api.reuse.software/info/github.com/voyager-tech-inc/vita49-rs)

## Install

Add `vita49` to your project with:

```sh
cargo add vita49
```

## Examples

Here's some code to ingest VRT packets from a UDP socket. If it's a signal
data packet, it'll just print the length and stream ID. If it's a context
packet, it'll print all the fields that are present.

```rust,no_run
use std::net::UdpSocket;
use vita49::prelude::*;

fn main() -> Result<(), std::io::Error> {
    // Bind to a UDP socket
    let socket = UdpSocket::bind("0.0.0.0:4991")?;
    let mut buf = [0; 40960];

    println!("Entering receive loop...");
    loop {
        // Read in data from the socket
        let (bytes_read, _src) = socket.recv_from(&mut buf)?;

        // Try to parse it as a VRT packet
        let packet = Vrt::try_from(&buf[..bytes_read])?;

        // Do different things depending on the type of packet
        match packet.header().packet_type() {
            // If it's a signal data packet, just print the payload length
            PacketType::SignalData => {
                println!(
                    "Got signal data packet with stream ID 0x{:X} and a payload of length {}",
                    &packet.stream_id().unwrap(),
                    &packet.payload().signal_data().unwrap().payload_size_bytes()
                );
            }
            // If it's a context packet, print the fields (using the pre-
            // implemented Display trait)
            PacketType::Context => {
                println!(
                    "Got context packet:\n{}",
                    &packet.payload().context().unwrap()
                );
            }
            PacketType::Command => {
                println!(
                    "Got command packet:\n{}",
                    &packet.payload().command().unwrap()
                );
            }
            // Other packet types are not covered in this example
            _ => unimplemented!(),
        }
    }
}
```

See [`udp_recv.rs`](./examples/udp_recv.rs) for the full example.

Here's another example of generating and sending VRT packets:

```rust,no_run
use std::net::UdpSocket;
use vita49::prelude::*;

fn main() -> Result<(), std::io::Error> {
    // Bind to a UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Create a context packet with RF freq set to 100 MHz and
    // bandwidth set to 8 MHz.
    let mut packet = Vrt::new_context_packet();
    packet.set_stream_id(Some(0xDEADBEEF));
    let context = packet.payload_mut().context_mut().unwrap();
    context.set_rf_ref_freq_hz(Some(100e6));
    context.set_bandwidth_hz(Some(8e6));
    packet.update_packet_size();

    // Send the packet
    socket.send_to(&packet.to_bytes()?, "127.0.0.1:4991")?;

    // Create a signal data packet with some dummy data.
    let mut sig_packet = Vrt::new_signal_data_packet();
    sig_packet.set_stream_id(Some(0xDEADBEEF));
    sig_packet
        .set_signal_payload(&vec![1, 2, 3, 4, 5, 6, 7, 8])
        .unwrap();

    // Send the packet
    socket.send_to(&sig_packet.to_bytes()?, "127.0.0.1:4991")?;

    Ok(())
}
```

See [`udp_send.rs`](./examples/udp_send.rs) for the full example.

You can actually run these two examples locally to see the output.
In one terminal window, run:
```shell
cargo run --example udp_recv
```

Then, in another window, run:
```shell
cargo run --example udp_send
```

On the receive end, you should see something like:
```text
Entering receive loop...
Got context packet:
CIF0:
  Context field change indicator: false
  Reference point identifier: false
  Bandwidth: true
  IF reference frequency: false
  RF reference frequency: true
  RF reference frequency offset: false
  IF band offset: false
  Reference level: false
  Gain: false
  Over-range count: false
  Sample rate: false
  Timestamp adjustment: false
  Timestamp calibration time: false
  Temperature: false
  Device identifier: false
  State/event indicators: false
  Signal data format: false
  Formatted GPS: false
  Formatted INS: false
  ECEF ephemeris: false
  Relative ephemeris: false
  Ephemeris ref ID: false
  GPS ASCII: false
  Context association lists: false
  CIF7: false
  CIF3: false
  CIF2: false
  CIF1: false
Bandwidth: 8000000 Hz
RF reference frequency: 100000000 Hz

Got signal data packet with stream ID 0xDEADBEEF and a payload of length 8
```

### Command & Control

VITA 49.2 introduces the ability to perform command and control (C2) operations
using VITA packets. For an example of both sides of a C2 flow, see
[the NATS control example programs](vita49/examples/README.md).

## C++ Interoperability

For an example of how to use this crate from a C++ app, see
[`cxx_demo/README.md`](cxx_demo/README.md).

## Python Interoperability

For an example of how to use this crate from a Python app, see
[`pyo3_demo/README.md`](pyo3_demo/README.md).

## Crate features

By default, this crate does not enable any of its optional features, leaving
them as "opt-in" by the user.

### `cif7`

This feature enables CIF7 support.

To use this feature, enable it in your `Cargo.toml`:

```toml
vita49 = { version = "0.0.6", features = ["cif7"] }
```

CIF7, also known as "field attributes", add an ability to provide descriptive
statistics of various fields along with their current value. This does, however,
add some parsing overhead as every field then includes up to 31 additional,
optional fields

With benchmark tests (via `cargo bench`), we can see as of the time of this
writing that enabling CIF7 support yields a **20% performance reduction**
in signal data packets and a **63% performance reduction** in context packets.

Since the feature isn't used widely and impacts performance, it's left disabled
by default.

### `serde`

This feature enables [serde](https://serde.rs/) support.

To use this feature, enable it in your `Cargo.toml`:

```toml
vita49 = { version = "0.0.6", features = ["serde"] }
```

With this feature enabled, you can serialize/deserialize structures provided
by this crate with serde. For example, to print a VRT packet as JSON:

```rust
use vita49::prelude::*;
#[cfg(feature = "serde")]
{
    let mut packet = Vrt::new_context_packet();
    let context = packet.payload_mut().context_mut().unwrap();
    context.set_bandwidth_hz(Some(8e6));
    packet.update_packet_size();
    println!("{}", serde_json::to_string_pretty(&packet).unwrap())
}
```

This yields:
```json
{
  "header": {
    "hword_1": 17005,
    "packet_size": 29
  },
  "stream_id": 1,
  "class_id": null,
  "integer_timestamp": 60045,
  "fractional_timestamp": 411360110,
  "payload": {
    "Context": {
      "cif0": 673316866,
      "cif1": 1032,
  "etc": "...",
```

This repo has some test VRT packets stored as JSON strings for visibility.
An example program is provided to convert these to raw VRT files under
[`vita49/examples/json2vrt.rs`](vita49/examples/json2vrt.rs). You can
run this program via `cargo`:

```text
% cargo run --features=serde --example json2vrt vita49/tests/spectral_data_packet.json5
   Compiling vita49 v0.0.6 (vita49/vita49)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.71s
     Running `target/debug/examples/json2vrt vita49/tests/spectral_data_packet.json5`
Wrote VRT data to vita49/tests/spectral_data_packet.vrt
```

### `deku-log`

Enables the [`deku` crate's logging feature](https://docs.rs/deku/latest/deku/#debugging-decoders-with-the-logging-feature).

## TODO

According to Section 1.3 of ANSI/VITA-49.2-2017:
> V49.2 provides millions of options for data types and packet structures
> that are likely impossible to implement as one application program
> interface (API) without culling down to required attributes.

While we'd love to cover every possible combination VITA 49 can offer, it
has to be a community effort to get there. We've implemented full support
for things that we use and things that we figure would be most likely to
be used by others, but we don't have 100% coverage.

If there's a field you'd like to use and you don't feel able to add support
for it yourself (and hopefully contribute your code back!), feel free to
file an issue and we'll know what to prioritize.

There are basically three states of "doneness" for various fields:

1. Full support
2. Basic support
3. No support

Fully supported fields have getters and setters that deliver a clean
interface to the user (e.g. fixed point fields are translated to
primitive `f64` or a multi-word data structure is filled out in a
meaningful `struct`).

Basic support means the field is there and can be parsed/set, but
it'd be better to implement a fully-featured struct. Using these
fields means you might need to do some bit masking/shifting yourself.
These fields are marked with a comment: `// TODO: add full support`.

No support means the crate will `panic!()` if the field is encountered.
This is usually because the field can be variable length and, until
support for the field is added, we can't guarantee the packet will
be parsed correctly. These fields are marked with a macro:
`todo_cif_field!()`. The getters/setters associated with these
fields are marked with a comment: `// TODO: add basic support`.

## Debugging

If this crate is unable to parse some VITA 49 data you're working with,
there are two possible causes:

1. The VITA 49 packet is not compliant with the standard.
2. There is a bug in the crate.

Either or both causes are very possible!

The [`Deku`] crate used for binary parsing provides a helpful trace
logging option that can be invaluable for troubleshooting these
low-level issues.

To enable this logging:

1. Enable the `deku-log` feature of this crate.
2. Import the `log` crate and a compatible logging library.

For example, to log with `env_logger`, you may add to your `Cargo.toml`:

```toml
vita49 = { version = "*", features = ["deku-log"] }
log = "*"
env_logger = "*"
```

Then you’d call `env_logger::init()` or `env_logger::try_init()`
prior to attempting to parse a packet.

Then, each field being parsed will print as it goes:

```text
[TRACE vita49::vrt] Reading: Vrt.header
[TRACE vita49::packet_header] Reading: PacketHeader.hword_1
[TRACE deku::reader] read_bytes_const: requesting 2 bytes
[TRACE deku::reader] read_bytes_const: returning [41, 00]
[TRACE vita49::packet_header] Reading: PacketHeader.packet_size
[TRACE deku::reader] read_bytes_const: requesting 2 bytes
[TRACE deku::reader] read_bytes_const: returning [00, 07]
[TRACE vita49::vrt] Reading: Vrt.stream_id
[TRACE deku::reader] read_bytes_const: requesting 4 bytes
[TRACE deku::reader] read_bytes_const: returning [de, ad, be, ef]
```

From here, you can step through the packet you're trying to parse
and see if it's setting something incorrectly or the crate is parsing
something incorrectly.

If you think you've found a bug, please do report it! See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for more info on how to do that.

## Minimum Rust Version Policy

This crate's minimum supported `rustc` version is `1.71.0`.

The minimum supported `rustc` version may be increased in minor version
updates of this crate. For example, if `vita49` `1.2.0` requires Rust `1.60.0`,
versions `1.2.X` of `vita49` will also require Rust `1.6.0` or newer. However,
`vita49` version `1.3.0` may require a newer minimum version of Rust.

## Crate Versioning

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

After initial project release (0.1.0), any changes that would break API compatibility
will require a major version number bump.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([`LICENSES/Apache-2.0.txt`](LICENSES/Apache-2.0.txt) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([`LICENSES/MIT.txt`](LICENSES/MIT.txt) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for details.
