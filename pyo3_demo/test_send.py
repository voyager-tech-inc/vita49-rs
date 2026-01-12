#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 The vita49-rs Authors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

import sys
import argparse
import pyo3_demo


def main():
    parser = argparse.ArgumentParser(
        prog="py-vrt-command",
        description="Python VITA 49.2 command script",
    )
    parser.add_argument(
        "-d",
        "--destination",
        metavar="VITA_ENDPOINT",
        required=True,
        help="IP address/hostname of VITA 49.2 endpoint to control",
    )
    parser.add_argument(
        "-s",
        "--stream-id",
        metavar="STREAM_ID",
        type=int,
        help="VITA 49.2 stream ID",
    )
    parser.add_argument(
        "-b",
        "--bandwidth-hz",
        metavar="BANDWIDTH_HZ",
        type=float,
        help="Bandwidth to set (Hz)",
    )
    parser.add_argument(
        "-f",
        "--frequency-hz",
        metavar="FREQUENCY_HZ",
        type=float,
        help="Center tune frequency (Hz)",
    )

    args = parser.parse_args(sys.argv[1:])

    # Create a new VRT client (implemented in Rust)
    vrt_client = pyo3_demo.VrtClient(args.destination, args.stream_id)

    if not args.bandwidth_hz and not args.frequency_hz:
        print("error - please pass at least one of -b or -f")
        sys.exit(1)

    # Take our input parameters and send them off to the Rust impl to
    # actually construct and send the VITA 49.2 packet and receive an
    # ACK.
    ack = vrt_client.send_cmd(args.bandwidth_hz, args.frequency_hz)

    if ack.ok:
        print("Got ACK back with no errors")
    else:
        print("Got ACK back with errors")


if __name__ == "__main__":
    main()
