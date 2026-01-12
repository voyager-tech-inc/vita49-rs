// SPDX-FileCopyrightText: 2026 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::time::Duration;

use jiff::Timestamp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use vita49::command_prelude::*;
use vita49::prelude::*;

#[pyclass]
struct VrtClient {
    dest: String,
    socket: std::net::UdpSocket,
    stream_id: Option<u32>,
}

#[pyclass]
struct VrtAck {
    #[pyo3(get, set)]
    ok: bool,
}

/// Create a new VRT control packet based on input bandwidth and frequency.
fn create_control_message(
    stream_id: Option<u32>,
    bandwidth_hz: Option<f64>,
    tune_freq_hz: Option<f64>,
) -> Vrt {
    // Get current timestamp to fill into the VRT packet.
    let secs_since_epoch: u32 = Timestamp::now().as_second().try_into().unwrap();
    let psecs_since_last_epoch_second =
        Timestamp::now().as_nanosecond() - (secs_since_epoch as f64 * 1e12) as i128;

    let mut control_packet = Vrt::new_control_packet();
    control_packet.set_stream_id(stream_id);
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

    // Set the data fields.
    let control = command.payload_mut().control_mut().unwrap();
    control.set_rf_ref_freq_hz(tune_freq_hz);
    control.set_bandwidth_hz(bandwidth_hz);

    control_packet.update_packet_size();
    control_packet
}

#[pymethods]
impl VrtClient {
    #[new]
    fn new(dest: String, stream_id: Option<u32>) -> PyResult<Self> {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| PyValueError::new_err(format!("failed to bind to UDP socket: {e}")))?;

        socket.set_read_timeout(Some(Duration::from_secs(2)))?;
        Ok(VrtClient {
            dest,
            socket,
            stream_id,
        })
    }

    fn send_cmd(&self, rf_ref_freq_hz: Option<f64>, bandwidth_hz: Option<f64>) -> PyResult<VrtAck> {
        let command_packet = create_control_message(self.stream_id, rf_ref_freq_hz, bandwidth_hz);
        self.socket
            .send_to(&command_packet.to_bytes().unwrap(), &self.dest)
            .map_err(|e| PyValueError::new_err(format!("failed to send packet: {e}")))?;
        let mut response_buf = [0; 4096];
        match self.socket.recv_from(&mut response_buf) {
            Ok((bytes_read, _src)) => {
                let ack_packet =
                    Vrt::try_from(&response_buf[..bytes_read]).expect("failed to parse ACK");
                let ack_command = ack_packet.payload().command().unwrap();
                match ack_command.payload() {
                    CommandPayload::ExecAck(_ack) => Ok(VrtAck {
                        ok: !ack_command.cam().error(),
                    }),
                    _ => Err(PyValueError::new_err("invalid ack type")),
                }
            }
            Err(e) => Err(PyValueError::new_err(format!("error: {e}"))),
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
mod pyo3_demo {
    #[pymodule_export]
    use super::VrtAck;
    #[pymodule_export]
    use super::VrtClient;
}
