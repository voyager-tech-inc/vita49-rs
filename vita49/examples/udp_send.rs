// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
use std::net::UdpSocket;
use vita49::prelude::*;

fn main() -> Result<(), std::io::Error> {
    // Bind to a UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Create a context packet with RF freq set to 100 MHz and
    // bandwidth set to 8 MHz.
    let mut packet = Vrt::new_context_packet();
    packet.set_stream_id(Some(0xDEADBEEF)).unwrap();
    let context = packet.payload_mut().context_mut().unwrap();
    context.set_rf_ref_freq_hz(Some(100e6));
    context.set_bandwidth_hz(Some(8e6));
    packet.update_packet_size().unwrap();

    // Send the packet
    socket.send_to(&packet.to_bytes()?, "127.0.0.1:4991")?;

    // Create a signal data packet with some dummy data.
    let mut sig_packet = Vrt::new_signal_data_packet();
    sig_packet.set_stream_id(Some(0xDEADBEEF)).unwrap();
    sig_packet
        .set_signal_payload(&[1, 2, 3, 4, 5, 6, 7, 8])
        .unwrap();

    // Send the packet
    socket.send_to(&sig_packet.to_bytes()?, "127.0.0.1:4991")?;

    Ok(())
}
