// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
use std::net::UdpSocket;
use vita49::prelude::*;

fn main() -> Result<(), std::io::Error> {
    // Bind to a UDP socket
    let socket = UdpSocket::bind("0.0.0.0:4991")?;
    let mut buf = [0; 40960];

    println!("Entering receive loop...");
    loop {
        // Read in data from the socket
        let (bytes_read, src) = socket.recv_from(&mut buf)?;

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
                let command = packet.payload().command().unwrap();
                // Create an ACK packet based on what the control packet
                // requested (or just return here if no ACK is requested).
                let mut reply = if command.cam().execution() {
                    Vrt::new_exec_ack_packet()
                } else if command.cam().validation() {
                    Vrt::new_validation_ack_packet()
                } else if command.cam().state() {
                    Vrt::new_query_ack_packet()
                } else {
                    continue;
                };
                // Mirror some of the header values from the command packet to make
                // sure the controller knows which command we're replying to.
                reply.set_stream_id(packet.stream_id());
                reply.update_packet_size();

                // Send a VITA 49.2 ACK back to the client
                socket.send_to(&reply.to_bytes()?, src).unwrap();
            }
            // Other packet types are not covered in this example
            _ => unimplemented!(),
        }
    }
}
