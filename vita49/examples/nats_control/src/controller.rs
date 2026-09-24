// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Example program for a VITA 49.2 controller. This app takes in command
line parameters for bandwidth and frequency and sends a control packet
via NATS.
*/

use clap::Parser;
use env_logger::Env;
use jiff::Timestamp;
use log::{debug, error, info};
use std::io::{Error, ErrorKind};
use vita49::command_prelude::*;
use vita49::prelude::*;

#[derive(Parser, Debug)]
struct Args {
    /// NATS URL
    #[arg(short, long, default_value = "nats://127.0.0.1:4222")]
    url: String,
    /// NATS command subject
    #[arg(short, long, default_value = "vrt.command")]
    subject: String,
    /// Query current device settings
    #[arg(short, long)]
    query: bool,
    /// Bandwidth (Hz)
    #[arg(short, long)]
    bandwidth_hz: Option<f64>,
    /// Tune frequency (Hz)
    #[arg(short, long)]
    freq_hz: Option<f64>,
}

/// Create a new VRT control packet based on input bandwidth and frequency.
fn create_control_message(bandwidth_hz: Option<f64>, tune_freq_hz: Option<f64>) -> Vrt {
    // Get current timestamp to fill into the VRT packet.
    let secs_since_epoch: u32 = Timestamp::now().as_second().try_into().unwrap();
    let psecs_since_last_epoch_second =
        Timestamp::now().as_nanosecond() - (secs_since_epoch as f64 * 1e12) as i128;

    let mut control_packet = Vrt::new_control_packet();
    control_packet.set_stream_id(Some(0x1)).unwrap();
    control_packet
        .set_integer_timestamp(Some(secs_since_epoch), Tsi::Utc)
        .unwrap();
    control_packet
        .set_fractional_timestamp(Some(psecs_since_last_epoch_second as u64), Tsf::RealTimePs)
        .unwrap();

    // Set up the CAM field to execute the request and request ACKs.
    let mut cam = ControlAckMode::default();
    cam.set_action_mode(ActionMode::Execute);
    cam.set_warnings_permitted();
    cam.set_warning();
    cam.set_error();
    cam.set_partial_packet_impl_permitted();
    cam.set_execution();

    let command = control_packet.payload_mut().command_mut().unwrap();
    command.set_cam(cam);

    // Set controllee/controller identifiers (using 32-bit ID and 128-bit UUID respectively)
    command.set_controllee_id(Some(1234)).unwrap();
    command
        .set_controller_uuid(Some(0x7F3CEF62E56848F18B88A7576FA634DF))
        .unwrap();

    // Set the data fields.
    let control = command.payload_mut().control_mut().unwrap();
    control.set_rf_ref_freq_hz(tune_freq_hz);
    control.set_bandwidth_hz(bandwidth_hz);

    control_packet.update_packet_size().unwrap();
    control_packet
}

/// Create a new VRT query packet.
fn create_query_message() -> Vrt {
    let mut control_packet = Vrt::new_control_packet();
    control_packet.set_stream_id(Some(0x1)).unwrap();

    let mut cam = ControlAckMode::default();
    cam.set_action_mode(ActionMode::NoAction);
    // This asks the controllee for current state.
    cam.set_state();

    let command = control_packet.payload_mut().command_mut().unwrap();
    command.set_cam(cam);
    command.set_controllee_id(Some(1234)).unwrap();
    command
        .set_controller_uuid(Some(0x7F3CEF62E56848F18B88A7576FA634DF))
        .unwrap();

    control_packet.update_packet_size().unwrap();
    control_packet
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    info!("Connecting to NATS at {}...", args.url);
    let nats_client = async_nats::connect(&args.url).await?;

    if !args.query && (args.bandwidth_hz.is_none() && args.freq_hz.is_none()) {
        error!("error - pass frequency and/or bandwidth");
        return Err(Error::new(ErrorKind::InvalidInput, "invalid parameters").into());
    }

    // Get a packet to send via NATS.
    let packet = if args.query {
        create_query_message()
    } else {
        create_control_message(args.bandwidth_hz, args.freq_hz)
    };

    // Send the request and wait for a response.
    let response = nats_client
        .request(args.subject, packet.to_bytes()?.into())
        .await?;
    // Try to parse a VRT packet out of the response.
    let ack_packet = Vrt::try_from(response.payload.as_ref())?;
    debug!("Got ACK:\n{ack_packet:#?}");
    let command = ack_packet.payload().command().unwrap();
    match command.payload() {
        CommandPayload::ExecAck(ack) => {
            // If there are errors in the ACK, report them back to the user.
            if command.cam().error() {
                if let Some((_level, bw_ack)) = ack.bandwidth() {
                    error!("Bandwidth error:\n{bw_ack}");
                }
                if let Some((_level, freq_ack)) = ack.rf_ref_freq() {
                    error!("Frequency error:\n{freq_ack}");
                }
                Err(Error::new(ErrorKind::InvalidData, "ACK errors received").into())
            } else {
                info!("No errors found in ACK!");
                Ok(())
            }
        }
        CommandPayload::QueryAck(ack) => {
            info!("Current state:\n{ack}");
            Ok(())
        }
        _ => Err(Error::other("invalid ack type").into()),
    }
}
