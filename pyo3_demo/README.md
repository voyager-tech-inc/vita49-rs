# Using the `vita49` Crate from a Python Application
<!--
SPDX-FileCopyrightText: 2026 The vita49-rs Authors

SPDX-License-Identifier: MIT OR Apache-2.0
-->

Python has a huge ecosystem of AI/ML libraries and useful math libraries
(e.g. numpy, pandas, etc.). It's also handy for quick CLI applications
for hardware control. To that end, it can be useful to have Python apps
use the `vita49` crate for the binary marshalling of VITA 49.2 packets
while the main business logic is handled in Python.

To achieve this, let's use [PyO3](https://pyo3.rs/v0.27.2/index.html)
to handle FFI.

## Defining the Foreign Function Interface (FFI)

PyO3 has [excellent documentation](https://pyo3.rs/v0.27.2/getting-started.html),
but in brief, you define an FFI layer in a Rust file with annotations
showing how to access things from Python. For example:

```rust
#[pyclass]
struct VrtClient {
    dest: String,
    socket: std::net::UdpSocket,
    stream_id: Option<u32>,
}

#[pymethods]
impl VrtClient {
    #[new]
    fn new(dest: String, stream_id: Option<u32>) -> PyResult<Self> {
        unimplemented!()
    }

    fn send_cmd(&self, rf_ref_freq_hz: Option<f64>, bandwidth_hz: Option<f64>) -> PyResult<VrtAck> {
        unimplemented!()
    }
}

#[pymodule]
mod pyo3_demo {
    #[pymodule_export]
    use super::VrtClient;
}
```

Then, from the Python side, you do:

```python
import pyo3_demo
vrt_client = pyo3_demo.VrtClient("127.0.0.1:4991", 1234)
ack = vrt_client.send_cmd(10000, 100000000)
```

For details, see the [`src/lib.rs`](src/lib.rs) and
[`test_send.py`](test_send.py) files.

## Try it out

```bash
# Set up a new virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install maturin (used to generate the FFI)
python3 -m pip install maturin
maturin develop
```

From here, you should be able to run Python and import the demo:

```python3
import pyo3_demo
help(pyo3_demo)
```

```
Help on package pyo3_demo:

NAME
    pyo3_demo - A Python module implemented in Rust.

PACKAGE CONTENTS
    pyo3_demo

CLASSES
    builtins.object
        builtins.VrtAck
        builtins.VrtClient

    class VrtAck(object)
     |  Data descriptors defined here:
     |
     |  ok

    class VrtClient(object)
     |  VrtClient(dest, stream_id)
     |
     |  Methods defined here:
     |
     |  send_cmd(self, /, rf_ref_freq_hz, bandwidth_hz)
     |
     |  ----------------------------------------------------------------------
     |  Static methods defined here:
     |
     |  __new__(*args, **kwargs)
     |      Create and return a new object.  See help(type) for accurate signature.

DATA
    __all__ = ['VrtAck', 'VrtClient']
```

A test program ([`test_send.py`](test_send.py)) shows how to do a basic C2 flow using
Python and the FFI.

You can launch the example UDP recv application at the top-level of this repo:

```bash
cargo run --example udp_recv
```

And then in another window, launch the `test_send.py` command:

```bash
./test_send.py -d 127.0.0.1:4991 -s 1 -b 40000 -f 100000000
```

This will send a command packet to the `udp_recv` app which will
receive it and send an ACK back to the sender.

```
Entering receive loop...
Got command packet:
CAM:
  Controllee enabled: false
  Controllee ID format: Id32bit
  Controller enabled: false
  Controller ID format: Id32bit
  Partial packet impl permitted: true
  Warnings permitted: true
  Errors permitted: false
  Action mode: Execute
  NACK only: false
  Validation: false
  Execution: true
  State: false
  Warning: true
  Error: true
  Timing control: IgnoreTimestamp
Message ID: 0
Control:
  Bandwidth: 40000 Hz
  RF reference frequency: 100000000 Hz
```
