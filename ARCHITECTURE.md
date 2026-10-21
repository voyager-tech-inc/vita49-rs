# Architecture
<!--
SPDX-FileCopyrightText: 2025 The vita49-rs Authors

SPDX-License-Identifier: MIT OR Apache-2.0
-->

1. [Project Layout](#project-layout)
2. [Data Model](#data-model)
3. [Macros](#macros)
4. [Tests](#tests)

## Project Layout

The `vita49-rs` project consists of two main crates:

1. `vita49`: The main user-facing crate.
2. `vita49_macros`: Procedural macros that help implement `vita49`.

Under `vita49`, you'll find the following directories:

- `benches`: Benchmarking programs (using [criterion](https://github.com/bheisler/criterion.rs)).
- `examples`: Example programs that use the `vita49` crate.
- `src`: The main source code for the VITA 49 data structure implementations.
- `tests`: VRT test data and integration tests.

The vast majority of the crate's implementation lives under `vita49/src`.
Each module implements a different piece of the VITA 49 standard, broken
out into logical chunks.

## Data Model

The most important data structure, `Vrt`, is defined in [`vrt.rs`](vita49/src/vrt.rs).
All other structures and enumerations nest under `Vrt`. Here's what it looks like:

```rust
pub struct Vrt {
    /// VRT packet header (present on all packets).
    header: PacketHeader,
    /// Stream identifier.
    #[deku(cond = "header.stream_id_included()")]
    stream_id: Option<u32>,
    /// Class identifier.
    #[deku(cond = "header.class_id_included()")]
    class_id: Option<ClassIdentifier>,
    /// Integer timestamp.
    #[deku(cond = "header.integer_timestamp_included()")]
    integer_timestamp: Option<u32>,
    /// Fractional timestamp.
    #[deku(cond = "header.fractional_timestamp_included()")]
    fractional_timestamp: Option<u64>,
    /// Packet payload. For signal data, this would be raw bytes. For
    /// context, this would be context information, etc..
    #[deku(ctx = "header")]
    payload: Payload,
    /// Data trailer.
    #[deku(cond = "header.trailer_included()")]
    trailer: Option<Trailer>,
}
```

Now, there are a few interesting features about this structure. First,
you'll notice the `deku` attributes on some of the fields. These are
provided by the fantastic [Deku crate](https://docs.rs/deku/latest/deku/).

Deku provides a way to annotate a data structure such that it can be parsed
from binary given various attributes. You can read about the different
possible parameters you can pass to the `deku()` macro
[here](https://docs.rs/deku/latest/deku/attributes/index.html).

The killer feature of Deku that made it a natural fit for VITA 49 is
the `cond` parameter. *Many* fields in a VRT packet are conditional based
on some bit or value being set earlier in the packet.

Above, you can see the `stream_id` field in the VRT packet will only
be parsed if the packet header indicates there should be a stream ID
present:

```rust
    #[deku(cond = "header.stream_id_included()")]
    stream_id: Option<u32>,
```

If there is no stream ID, the field will simply be set to `None`.

The other interesting member of the `Vrt` struct is `payload`.

The `Payload` type isn't actually a structure; it's an `enum`.
The contents of a VRT packet's payload vary greatly depending
on the type of packet you're dealing with. For example, a context
packet's payload is very different from a signal data packet payload.

To that end, instead of implementing a patchwork of different top-level
data structures for the different packet types, this crate enumerates
the different payload types (wrapping the underlying structure):

From [`payload.rs`](vita49/src/payload.rs):
```rust
#[deku(ctx = "packet_header: &PacketHeader", id = "packet_header.packet_type()")]
#[non_exhaustive]
pub enum Payload {
    /// Payload for a context packet.
    #[deku(id = "PacketType::Context | PacketType::ExtensionContext")]
    Context(Context),
    /// Payload for a command packet.
    #[deku(id = "PacketType::Command | PacketType::ExtensionCommand")]
    Command(Command),
    /// Payload for a signal data packet (types 0 and 1).
    #[deku(id = "PacketType::SignalData | PacketType::SignalDataWithoutStreamId")]
    SignalData(#[deku(ctx = "packet_header")] SignalData),
    /// Payload for an extension data packet (types 2 and 3).
    #[deku(id = "PacketType::ExtensionData | PacketType::ExtensionDataWithoutStreamId")]
    ExtensionData(Vec<u8>),
}
```

Note that this enumeration use's Rust's ability to [attach a type to
variants directly](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#enum-values).

When parsing a packet, Deku will check the packet header for the
packet type and match the payload variant accordingly. The underlying
structure implementations are defined in [`context.rs`](vita49/src/context.rs),
[`command.rs`](vita49/src/command.rs), and [`signal_data.rs`](vita49/src/signal_data.rs)
respectively. Extension data payloads are application-defined, so they are plain
bytes that reuse the signal data wire format. The packet's class identifier says
which application format they follow.

The end-user is then expected to "unwrap" the payload to whatever type
of packet is found. So, if the user checks the packet header and finds a
context packet, they can safely run `packet.payload().context()`. This
returns a `Result<&Context, VitaError>`. If the user tries to run the
`context()` unwrapper on a non-context packet, an error will be returned.

While the ergonomics of this data model aren't perfect, the value of having
one unifying type for all VRT packets is large enough to warrant it.

## Macros

The `vita49_macros` crate nested in this project provides a few helpful
procedural macros to make implementations of many similar fields easier.

Let's look at one field example from CIF0 to see how the crate abstracts
the gritty details:

```rust
impl Cif0 {
    // ...
    cif_field!(bandwidth, 29);
    // ...
```

Each CIF indicator field is just one bit in the 32-bit CIF value. This
bit indicates if the `bandwidth` field is present or not.

The actual *data* fields are a separate struct:

```rust
#[cif_fields(cif0)]
pub struct Cif0Fields {
    // ...
    bandwidth: u64,
    // ...
}
```

The `cif_fields` attribute triggers a macro that adds the following underlying
fields in the struct (using `bandwidth` as an example):

```rust
pub struct Cif0Fields {
    // ...
    /// bandwidth data field
    #[deku(cond = "cif0.bandwidth() && cif7_opts.current_val")]
    pub(crate) bandwidth: Option<u64>,
    /// bandwidth data attributes field (only used if CIF7 is enabled)
    #[cfg(feature = "cif7")]
    #[deku(
        cond = "cif0.bandwidth() && cif7_opts.num_extra_attrs > 0",
        count = "cif7_opts.num_extra_attrs"
    )]
    pub(crate) bandwidth_attributes: Vec<u64>,
    // ...
}
```

Fundamentally, the macro takes the input field and adds the relevant
Deku parameters to make sure it's only parsed if the relevant CIF bit
is set.

The `bandwidth_attributes` field is only generated if feature `cif7`
is enabled.

Finally, the `Cif0Manipulators` trait is implemented:

```rust
pub trait Cif0Manipulators {
    // ...
    cif_radix!(cif0, bandwidth, bandwidth_hz, f64, FixedU64::<U20>);
    // ...
}
```

This trait defines shared behavior between context and command packets.
Here, the `cif_radix!()` macro is used to handle generating helpful
accessor methods for the bandwidth field. It takes the "friendly name"
(i.e. including a unit), the "friendly type", and the internal representation
type (in this case, a 64-bit fixed point with radix at bit 20). Fixed point
is a pain to deal with, so this crate presents accessor APIs that use simple
`f64` values.

Here's the code that `cif_radix!()` produces:

```rust
/// Get the current bandwidth_hz. If `None` is returned, the field is unset.
fn bandwidth_hz(&self) -> Option<f64> {
    self.cif0_fields().bandwidth.map(|v| FixedU64::<U20>::from_bits(v).to_num())
}

/// Get the current bandwidth_hz_attributes (CIF7 attributes). If `None` is returned, the field is unset.
#[cfg(feature = "cif7")]
fn bandwidth_hz_attributes(&self) -> Vec<f64> {
    self.cif0_fields()
        .bandwidth_attributes
        .iter()
        .map(|v| { FixedU64::<U20>::from_bits(*v).to_num() })
        .collect()
}

/// Set the bandwidth_hz. If `None` is passed, the field will be unset.
/// 
/// [`update_packet_size()`](Vrt::update_packet_size()) should be executed after running this method.
fn set_bandwidth_hz(&mut self, bandwidth_hz: Option<f64>) {
    if let Some(v) = bandwidth_hz {
        self.cif0_fields_mut().bandwidth = Some(
            FixedU64::<U20>::from_num(v).to_bits(),
        );
        self.cif0_mut().set_bandwidth();
    } else {
        self.cif0_fields_mut().bandwidth = None;
        self.cif0_mut().unset_bandwidth();
    }
}

/// Set the bandwidth_hz_attributes (CIF7 attributes). If `None` is passed, the field will be unset.
/// 
/// [`update_packet_size()`](Vrt::update_packet_size()) should be executed after running this method.
#[cfg(feature = "cif7")]
fn set_bandwidth_hz_attributes(
    &mut self,
    bandwidth_hz_attributes: Option<Vec<f64>>,
) {
    if let Some(vec) = bandwidth_hz_attributes {
        self.cif0_mut().set_field_attributes_enabled();
        self.cif0_fields_mut().bandwidth_attributes = vec
            .iter()
            .map(|v| FixedU64::<U20>::from_num(*v).to_bits())
            .collect();
        self.cif0_mut().set_bandwidth();
    } else {
        self.cif0_fields_mut().bandwidth_attributes.clear();
    }
}
```

A few things to note:

- If you set a field to some value, the setter will automatically set
  the relevant CIF field for you (and vice versa if you unset it).
- You can unset a field with `set_$field(None)`.
- The fixed point conversions are completely transparent to the user.
- Accessor methods use the relevant unit.
- The "attributes" accessor methods are only built if `cif7` is enabled.

Since most VRT fields operate in a similar way, using macros to define shared
behavior is a helpful way to cut back on boilerplate.

If you're curious what code is actually being generated by the macros,
[`cargo expand`](https://github.com/dtolnay/cargo-expand) is extremely helpful.

## Tests

This crate relies on a mix of unit tests, integration tests, and documentation tests.
When possible, implementing documentation tests in the form of examples is great as
they serve two purposes: give example code to the end-user and validate that the methods
work as expected.

Some integration tests use [Wireshark](https://www.wireshark.org/) to validate packet
behavior. Wireshark is one of the more complete implementations of a VITA 49 parser out
there, so if you're adding something that can be cross-checked by Wireshark, please
do add a test for it.

Additionally, if you have real VRT packets from real radios that aren't being parsed
correctly by this crate, we'd welcome adding them under `tests` along with a fix.
