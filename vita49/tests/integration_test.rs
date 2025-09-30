// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::{Error, ErrorKind};
use std::process::Command;

use semver_sort::semver::semver_compare;
use subprocess::Exec;
use tempfile::NamedTempFile;
use vita49::{prelude::*, ActionMode, ControlAckMode};
use vita49::{CommandPayload, Spectrum};
#[cfg(feature = "serde")]
use vita49::{Indicators, SignalDataIndicators, Tsf, Tsi};

fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

fn wireshark_parse(packet: &Vrt, check_strs: &[&str]) -> Result<(), Error> {
    if std::env::var("SKIP_WIRESHARK_TESTS").unwrap_or("false".to_string()) == "true" {
        eprintln!("Skipping Wireshark tests because SKIP_WIRESHARK_TESTS is set");
        return Ok(());
    }

    let od_path = std::env::var("OD_PATH").unwrap_or("od".to_string());
    let text2pcap_path = std::env::var("TEXT2PCAP_PATH").unwrap_or("text2pcap".to_string());
    let tshark_path = std::env::var("TSHARK_PATH").unwrap_or("tshark".to_string());
    // Minimum required version for added spectral decoding features.
    let min_tshark_version = "4.5.0";

    let mut err = false;
    match Command::new(&tshark_path).arg("--version").output() {
        Ok(cmd) => {
            // Expect the first line of output to match:
            // TShark (Wireshark) 4.5.0
            let stdout = String::from_utf8(cmd.stdout).unwrap();
            let first_line = stdout.lines().next().unwrap();
            let mut version_num = first_line.split_whitespace().nth(2).unwrap();
            if version_num.ends_with(".") {
                version_num = version_num.strip_suffix(".").unwrap();
            }
            if semver_compare(version_num, min_tshark_version, true) {
                eprintln!(
                    "TShark version ({version_num}) is too old - minimum required is {min_tshark_version}"
                );
                err = true;
            }
        }
        Err(e) => {
            err = true;
            eprintln!("tshark executable `{tshark_path}` failed: {e}");
        }
    }

    if let Err(e) = Command::new(&od_path).arg("--version").output() {
        err = true;
        eprintln!("od executable `{od_path}` failed: {e}");
    }

    if let Err(e) = Command::new(&text2pcap_path).arg("--version").output() {
        err = true;
        eprintln!("text2pcap executable `{text2pcap_path}` failed: {e}");
    }

    if err {
        let err_string = format!("missing prerequisites for Wireshark testing - install Tshark >={min_tshark_version} or set SKIP_WIRESHARK_TESTS=true in your env var to skip");
        eprintln!("{err_string}");
        return Err(Error::new(ErrorKind::NotFound, err_string));
    }

    // Write packet to a file, then wrap it in a dummy
    // pcap file and run it through tshark. It should
    // never say the packet is malformed.
    let tmp = NamedTempFile::new()?;
    packet.to_writer(&mut Writer::new(&tmp), ())?;
    let tmp_path = tmp.into_temp_path();

    // Take the raw binary of the VRT packet, wrap it in
    // a pcap file, then pass it into tshark.
    // od -Ax -tx1 -v ./tmp.vrt | text2pcap -u 4991,4991 - - | tshark -r - -V
    let tshark_out = {
        Exec::shell(format!(
            "{} -Ax -tx1 -v {}",
            od_path,
            tmp_path.to_str().unwrap()
        )) | Exec::shell(format!("{text2pcap_path} -u 4991,4991 - -"))
            | Exec::shell(format!("{tshark_path} -r - -V"))
    }
    .capture()
    .expect("failed to get capture");

    log::error!("STDERR:\n{}", tshark_out.stderr_str());
    log::error!("STDOUT:\n{}", tshark_out.stdout_str());

    if (!tshark_out.success())
        || tshark_out.stdout_str().contains("Malformed Packet")
        || tshark_out.stderr_str().contains("Malformed Packet")
    {
        log::error!("Wireshark couldn't parse this packet!");
        log::error!("STDERR:\n{}", tshark_out.stderr_str());
        log::error!("STDOUT:\n{}", tshark_out.stdout_str());
        return Err(Error::other("failed to parse packet"));
    }

    // Check for specific strings in the output
    for check_str in check_strs {
        if !tshark_out.stdout_str().contains(check_str) {
            let err = format!("output does not contain: \"{check_str}\"");
            log::error!("STDERR:\n{}", tshark_out.stderr_str());
            log::error!("STDOUT:\n{}", tshark_out.stdout_str());
            log::error!("{err}");
            return Err(Error::other(err));
        }
    }

    Ok(())
}

#[cfg(feature = "serde")]
#[test]
fn read_spectral_data() {
    log_init();
    let json = include_str!("spectral_data_packet.json5");
    let packet: Vrt = serde_json5::from_str(json).unwrap();
    assert_eq!(packet.header().packet_type(), PacketType::SignalData);
    assert_eq!(
        packet.header().indicators(),
        Indicators::SignalData(SignalDataIndicators {
            trailer_included: true,
            not_a_vita490_packet: false,
            signal_spectral_data: true,
        })
    );
    assert!(packet.header().stream_id_included());
    assert!(packet.header().class_id_included());
    assert!(packet.header().integer_timestamp_included());
    assert!(packet.header().fractional_timestamp_included());
    assert!(packet.header().trailer_included());
    assert_eq!(packet.header().tsi(), Tsi::Utc);
    assert_eq!(packet.header().tsf(), Tsf::RealTimePs);
    assert_eq!(packet.stream_id(), Some(1));
    assert_eq!(packet.class_id().unwrap().oui(), 0xff5654);
    assert_eq!(
        packet.payload().signal_data().unwrap().size_words(),
        1280 / 4
    );
    assert_eq!(
        packet.payload().signal_data().unwrap().payload_size_bytes(),
        1280
    );
    assert!(packet.trailer().is_some());
}

#[cfg(feature = "serde")]
#[test]
fn read_context() {
    log_init();
    let json = include_str!("context_packet.json5");
    let packet: Vrt = serde_json5::from_str(json).unwrap();
    assert_eq!(packet.header().packet_type(), PacketType::Context);
    assert!(packet.header().stream_id_included());
    assert!(!packet.header().class_id_included());
    assert!(packet.header().integer_timestamp_included());
    assert!(packet.header().fractional_timestamp_included());
    assert!(!packet.header().trailer_included());
    assert_eq!(packet.header().tsi(), Tsi::Utc);
    assert_eq!(packet.header().tsf(), Tsf::RealTimePs);
    assert_eq!(packet.stream_id(), Some(1));

    let context = packet.payload().context().unwrap();
    log::info!("\nParsed context packet:\n{}", context);
    assert!(!context.context_changed());
    assert_eq!(context.bandwidth_hz(), Some(6e6));
    assert_eq!(context.rf_ref_freq_hz(), Some(100e6));
    assert_eq!(context.sample_rate_sps(), Some(8e6));
    assert_eq!(context.spectrum().unwrap().spectrum_type_as_u32(), 0x101);
    assert_eq!(context.spectrum().unwrap().num_transform_points(), 1280);
    assert_eq!(context.spectrum().unwrap().f1_index(), -640);
}

#[cfg(feature = "serde")]
#[test]
fn modify_context() {
    log_init();
    let json = include_str!("context_packet.json5");
    let mut packet: Vrt = serde_json5::from_str(json).unwrap();
    assert_eq!(packet.header().packet_type(), PacketType::Context);
    let context = packet.payload_mut().context_mut().unwrap();
    assert_eq!(context.bandwidth_hz(), Some(6e6));
    context.set_bandwidth_hz(Some(8e6));
    assert_eq!(context.bandwidth_hz(), Some(8e6));
}

#[cfg(feature = "serde")]
#[test]
fn read_command() {
    log_init();
    let json = include_str!("command_packet.json5");
    let packet: Vrt = serde_json5::from_str(json).unwrap();
    assert_eq!(packet.header().packet_type(), PacketType::Command);
    let command = packet.payload().command().unwrap();
    log::info!("\nConstructed command packet:\n{}", command);
}

#[test]
fn construct_signal_data_packet() {
    log_init();
    let mut packet = Vrt::new_signal_data_packet();
    packet.set_stream_id(Some(0xDEADBEEF));
    packet
        .set_signal_payload(&[1, 2, 3, 4, 5, 6, 7, 8])
        .unwrap();
    packet.update_packet_size();
    assert!(wireshark_parse(
        &packet,
        &[
            "Packet type: IF data packet with stream ID (1)",
            "Data: 0102030405060708"
        ]
    )
    .is_ok());
    log::info!("\nConstructed signal data packet:\n{packet:#?}");
}

#[test]
fn construct_context_packet() {
    log_init();
    let mut packet = Vrt::new_context_packet();
    let context = packet.payload_mut().context_mut().unwrap();
    context.set_bandwidth_hz(Some(8e6));
    let mut spectrum = Spectrum::default();
    spectrum.set_num_transform_points(1280);
    spectrum.set_num_window_points(1280);
    spectrum.set_resolution_hz(6.25e3);
    spectrum.set_span_hz(8e6);
    spectrum.set_f1_index(-1280);
    spectrum.set_f2_index(1279);
    context.set_spectrum(Some(spectrum));
    packet.set_stream_id(Some(0xDEADBEEF));
    packet.update_packet_size();
    assert!(wireshark_parse(
        &packet,
        &[
            "Packet type: IF context packet (4)",
            "F1 index: -1280",
            "Resolution: 6.250000 kHz"
        ],
    )
    .is_ok());
    log::info!("\nConstructed context packet:\n{packet:#?}");
}

#[test]
fn construct_control_packet() {
    log_init();
    let mut packet = Vrt::new_control_packet();
    packet.set_stream_id(Some(0xDEADBEEF));
    let command = packet.payload_mut().command_mut().unwrap();
    let control = command.payload_mut().control_mut().unwrap();
    control.set_controllee_id(Some(0));
    control.set_controller_uuid(Some(0));
    control.set_rf_ref_freq_hz(Some(100e6));
    control.set_sample_rate_sps(Some(128e6));
    control.set_bandwidth_hz(Some(100e6));
    let mut cam = ControlAckMode::default();
    cam.set_action_mode(ActionMode::Execute);
    cam.set_partial_packet_impl_permitted();
    cam.set_warnings_permitted();
    cam.set_validation();
    cam.set_warning();
    cam.set_error();
    command.set_cam(cam);
    command.set_message_id(123);
    command.set_controllee_id(Some(0)).unwrap();
    command.set_controller_uuid(Some(0)).unwrap();

    packet.update_packet_size();
    assert!(wireshark_parse(&packet, &["Packet type: Unknown (6)"]).is_ok());
    log::info!("\nConstructed command packet:\n{packet:#?}");
    log::info!("\nPacket size (words): {}", packet.header().packet_size());
}

#[test]
fn exec_ack_parsing() {
    log_init();
    let packet = Vrt::new_exec_ack_packet();
    assert!(packet.header().is_ack_packet().is_ok());
    assert!(matches!(
        packet.payload().command().unwrap().payload(),
        CommandPayload::ExecAck(_)
    ));

    let bytes = packet.to_bytes().unwrap();
    let parsed_packet = Vrt::try_from(bytes.as_ref()).unwrap();

    assert!(parsed_packet.header().is_ack_packet().is_ok());
    assert!(matches!(
        parsed_packet.payload().command().unwrap().payload(),
        CommandPayload::ExecAck(_)
    ));
}

#[test]
fn validation_ack_parsing() {
    log_init();
    let packet = Vrt::new_validation_ack_packet();
    assert!(packet.header().is_ack_packet().is_ok());
    assert!(matches!(
        packet.payload().command().unwrap().payload(),
        CommandPayload::ValidationAck(_)
    ));

    let bytes = packet.to_bytes().unwrap();
    let parsed_packet = Vrt::try_from(bytes.as_ref()).unwrap();

    assert!(parsed_packet.header().is_ack_packet().is_ok());
    assert!(matches!(
        parsed_packet.payload().command().unwrap().payload(),
        CommandPayload::ValidationAck(_)
    ));
}

#[test]
fn query_ack_parsing() {
    log_init();
    let packet = Vrt::new_query_ack_packet();
    assert!(packet.header().is_ack_packet().is_ok());
    assert!(matches!(
        packet.payload().command().unwrap().payload(),
        CommandPayload::QueryAck(_)
    ));

    let bytes = packet.to_bytes().unwrap();
    let parsed_packet = Vrt::try_from(bytes.as_ref()).unwrap();

    assert!(parsed_packet.header().is_ack_packet().is_ok());
    assert!(matches!(
        parsed_packet.payload().command().unwrap().payload(),
        CommandPayload::QueryAck(_)
    ));
}

#[cfg(feature = "serde")]
#[test]
fn parse_ack_packet() {
    log_init();
    let json = include_str!("ack_packet.json5");
    let packet: Vrt = serde_json5::from_str(json).unwrap();
    assert_eq!(packet.header().packet_type(), PacketType::Command);
}

#[test]
#[cfg(feature = "cif7")]
fn construct_cif7_packet() {
    log_init();
    let mut packet = Vrt::new_context_packet();
    let context = packet.payload_mut().context_mut().unwrap();
    let mut cif7 = Cif7::default();
    cif7.set_current();
    cif7.set_average();
    cif7.set_median();
    context.cif7 = Some(cif7);
    context.set_bandwidth_hz(Some(8e6));
    context.set_bandwidth_hz_attributes(Some(vec![8.0, 7.0]));
    context.set_sample_rate_sps(Some(10e6));
    context.set_sample_rate_sps_attributes(Some(vec![11.0, 9.0]));
    packet.update_packet_size();
    assert_eq!(packet.header().packet_size(), 16);
    assert!(wireshark_parse(
        &packet,
        &["CIF7: True", "CIF7: 0xe0000000", "Bandwidth: 8.000000 MHz"]
    )
    .is_ok());
}

#[cfg(feature = "serde")]
#[test]
fn serde_json() {
    log_init();
    let json = include_str!("context_packet.json5");
    let packet: Vrt = serde_json5::from_str(json).unwrap();
    println!("{}", serde_json::to_string_pretty(&packet).unwrap())
}
