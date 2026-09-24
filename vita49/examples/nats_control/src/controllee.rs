// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Example program for a VITA 49.2 controllee. This app comes up and listens
on a NATS subject for VITA 49.2 command packets. It does some simple validation
of the command packet parameters and returns an ACK. It also replies to query
packets with its current internal state.
*/
use std::io::Error;
use std::io::ErrorKind;

use env_logger::Env;
use futures::StreamExt;
use log::{debug, error, info};
use vita49::command_prelude::*;
use vita49::prelude::*;

/// Dummy SDR simulation structure for holding "device" state.
#[derive(Copy, Clone, Debug, Default)]
struct Sdr {
    tune_freq_hz: f64,
    bandwidth_hz: f64,
}

impl Sdr {
    /// Dummy function to simulate sending a value to a device.
    fn send_to_device(&self, val: f64) -> Result<(), ()> {
        // Pretend if a value comes in below 5, it's an error.
        if val < 5.0 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Perform validation of the bandwidth parameter and return and
    /// `Err(AckResponse)` if there are any problems.
    fn set_bandwidth(&mut self, bw_hz: f64) -> Result<(), AckResponse> {
        let mut ack_response = AckResponse::default();
        if !(0.0..=100e6).contains(&bw_hz) {
            ack_response.set_param_out_of_range();
        }
        if let Err(_e) = self.send_to_device(bw_hz) {
            ack_response.set_device_failure()
        } else {
            self.bandwidth_hz = bw_hz;
        }
        if ack_response.empty() {
            Ok(())
        } else {
            Err(ack_response)
        }
    }

    /// Perform validation of the frequency parameter and return and
    /// `Err(AckResponse)` if there are any problems.
    fn set_freq(&mut self, freq_hz: f64) -> Result<(), AckResponse> {
        let mut ack_response = AckResponse::default();
        if !(0.0..=6e9).contains(&freq_hz) {
            ack_response.set_param_out_of_range();
        }
        if let Err(_e) = self.send_to_device(freq_hz) {
            ack_response.set_device_failure()
        } else {
            self.tune_freq_hz = freq_hz;
        }
        if ack_response.empty() {
            Ok(())
        } else {
            Err(ack_response)
        }
    }

    /// Process an individual command packet and return an ACK VRT
    /// packet if one is requested.
    fn process_command(&mut self, packet: &Vrt) -> Option<Vrt> {
        let command = packet.payload().command().unwrap();
        let control = command.payload().control().unwrap();
        info!("Processing control packet...");
        let mut bw_err = None;
        let mut freq_err = None;
        if let Some(bw_hz) = control.bandwidth_hz() {
            info!("Setting radio BW to {bw_hz}");
            if let Err(e) = self.set_bandwidth(bw_hz) {
                error!("Got error in setting bandwidth:\n{e}");
                bw_err = Some(e);
            }
        }
        if let Some(freq_hz) = control.rf_ref_freq_hz() {
            info!("Setting radio FC to {freq_hz}");
            if let Err(e) = self.set_freq(freq_hz) {
                error!("Got error in setting frequency\n{e}");
                freq_err = Some(e);
            }
        }

        // Create an ACK packet based on what the control packet
        // requested (or just return here if no ACK is requested).
        let mut reply = if command.cam().execution() {
            Vrt::new_exec_ack_packet()
        } else if command.cam().validation() {
            Vrt::new_validation_ack_packet()
        } else if command.cam().state() {
            Vrt::new_query_ack_packet()
        } else {
            return None;
        };

        // Mirror some of the header values from the command packet to make
        // sure the controller knows which command we're replying to.
        reply.set_stream_id(packet.stream_id()).unwrap();

        let ack = reply.payload_mut().command_mut().unwrap();
        let mut cam = command.cam();
        cam.unset_error();
        cam.unset_warning();
        if bw_err.is_some() || freq_err.is_some() {
            cam.set_error();
        } else {
            cam.set_action_scheduled_or_executed();
        }
        ack.set_cam(cam);
        ack.set_message_id(command.message_id());
        ack.set_controllee_id(command.controllee_id()).unwrap();
        ack.set_controllee_uuid(command.controllee_uuid()).unwrap();
        ack.set_controller_id(command.controller_id()).unwrap();
        ack.set_controller_uuid(command.controller_uuid()).unwrap();

        match ack.payload_mut() {
            // If we're sending a validation or exec ACK, fill in the fields with
            // any errors. If `bw_err` and `freq_err` are `None` here, the function
            // calls are effectively no-ops.
            CommandPayload::ValidationAck(a) | CommandPayload::ExecAck(a) => {
                a.set_bandwidth(AckLevel::Error, bw_err);
                a.set_rf_ref_freq(AckLevel::Error, freq_err);
            }
            // If we're sending a query ACK, just get the current values from our
            // "SDR" and send them back to the controller.
            CommandPayload::QueryAck(a) => {
                a.set_bandwidth_hz(Some(self.bandwidth_hz));
                a.set_rf_ref_freq_hz(Some(self.tune_freq_hz));
            }
            _ => unimplemented!(),
        }
        reply.update_packet_size().unwrap();
        Some(reply)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let nats_url = "nats://127.0.0.1:4222";
    let nats_subject = "vrt.command";
    info!("Connecting to NATS at {nats_url}...");
    let nats_client = async_nats::connect(&nats_url).await?;
    let mut sub = nats_client.subscribe(nats_subject).await.unwrap();

    let mut sdr = Sdr::default();

    info!("Awaiting commands");
    while let Some(message) = sub.next().await {
        // Parse the VRT packet out of the NATS message.
        let packet = Vrt::try_from(&message.payload.to_vec()[..])?;

        // Make sure it's a command packet.
        if !matches!(packet.header().packet_type(), PacketType::Command) {
            error!("Got a packet other than a command packet");
            return Err(Error::new(ErrorKind::InvalidInput, "invalid VRT packet").into());
        }

        // Process the command packet and get a constructed ACK back (if requested).
        if let (Some(re_subj), Some(ack)) = (message.reply, sdr.process_command(&packet)) {
            // If NATS provided a reply subject and the VRT packet requested an
            // ACK, send the ACK back through NATS.
            info!("Publishing ACK");
            debug!("{}", ack.payload().command().unwrap());
            nats_client.publish(re_subj, ack.to_bytes()?.into()).await?;
        }
    }
    Ok(())
}
