// Copyright 2017 CoreOS, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied
// See the License for the specific language governing permissions and
// limitations under the License.

use core::slice;
use core::str;

use base::Status;
use guid::Guid;
use protocol::Protocol;
use void::CVoid;

#[repr(C)]
pub struct SerialIOMode {
    control_mask: u32,
    timeout: u32,
    baud_rate: u64,
    receive_fifo_depth: u32,
    data_bits: u32,
    parity: ParityType,
    stop_bits: StopBits,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum ParityType {
    DefaultParity = 0,
    NoParity = 1,
    EvenParity = 2,
    OddParity = 3,
    MarkParity = 4,
    SpaceParity = 5,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum StopBits {
    DefaultStopBits = 0,
    OneStopBit = 1,
    OneFiveStopBits = 2,
    TwoStopBits = 3,
}

bitflags! {
    pub struct ControlBits: u32 {
        const DATA_TERMINAL_READY = 0x0001;
        const REQUEST_TO_SEND = 0x0002;
        const CLEAR_TO_SEND = 0x0010;
        const DATA_SET_READY = 0x0020;
        const RING_INDICATE = 0x0040;
        const CARRIER_DETECT = 0x0080;
        const INPUT_BUFFER_EMPTY = 0x0100;
        const OUTPUT_BUFFER_EMPTY = 0x0200;
        const HARDWARE_LOOPBACK_ENABLE = 0x1000;
        const SOFTWARE_LOOPBACK_ENABLE = 0x2000;
        const HARDWARE_FLOW_CONTROL_ENABLE = 0x4000;
    }
}

/// GUID for Serial I/O protocol
pub static EFI_SERIAL_IO_PROTOCOL_GUID: Guid = Guid(0xBB25CF6F, 0xF1D4, 0x11D2, [0x9A, 0x0C, 0x00, 0x90, 0x27, 0x3F, 0xC1, 0xFD]);

#[repr(C)]
struct RawSerialIOProtocol {
    revision: u32,
    reset: unsafe extern "win64" fn(this: *const RawSerialIOProtocol) -> Status,
    set_attributes: unsafe extern "win64" fn(this: *const RawSerialIOProtocol,
                                             baud_rate: u64,
                                             receive_fifo_depth: u32,
                                             timeout: u32,
                                             parity: ParityType,
                                             data_bits: u8,
                                             stop_bits: StopBits)
                                             -> Status,
    set_control_bits:
        unsafe extern "win64" fn(this: *const RawSerialIOProtocol, control: u32) -> Status,
    get_control_bits:
        unsafe extern "win64" fn(this: *const RawSerialIOProtocol, control: *mut u32) -> Status,
    write: unsafe extern "win64" fn(this: *const RawSerialIOProtocol,
                                    buffer_size: *mut usize,
                                    buffer: *const CVoid)
                                    -> Status,
    read: unsafe extern "win64" fn(this: *const RawSerialIOProtocol,
                                   buffer_size: *mut usize,
                                   buffer: *mut CVoid)
                                   -> Status,
    mode: *const SerialIOMode,
}

impl Protocol for RawSerialIOProtocol {
    fn guid() -> &'static Guid {
        &EFI_SERIAL_IO_PROTOCOL_GUID
    }
}

impl RawSerialIOProtocol {
    /// Reset the serial device.
    pub fn reset(&self) -> Result<(), Status> {
        unsafe {
            match (self.reset)(self as *const RawSerialIOProtocol) {
                Status::Success => Ok(()),
                e => Err(e),
            }
        }
    }

    /// Set the serial device's attributes. Passing `None` to any argument will keep the device
    /// default. Note that each attribute is set each time this function is called, so setting one
    /// attribute in one call and another in the next will result in the first attribute being
    /// reset to the device default.
    pub fn set_attributes(
        &self,
        baud_rate: Option<u64>,
        receive_fifo_depth: Option<u32>,
        timeout: Option<u32>,
        parity: Option<ParityType>,
        data_bits: Option<u8>,
        stop_bits: Option<StopBits>,
    ) -> Status {
        unsafe {
            (self.set_attributes)(
                self,
                baud_rate.unwrap_or(0),
                receive_fifo_depth.unwrap_or(0),
                timeout.unwrap_or(0),
                parity.unwrap_or(ParityType::DefaultParity),
                data_bits.unwrap_or(0),
                stop_bits.unwrap_or(StopBits::DefaultStopBits),
            )
        }
    }

    /// Set the serial device's control bits. Per the UEFI specification, only five of the control
    /// bits can be set with SetControlBits:
    ///
    /// * `REQUEST_TO_SEND`
    /// * `DATA_TERMINAL_READY`
    /// * `HARDWARE_LOOPBACK_ENABLE`
    /// * `SOFTWARE_LOOPBACK_ENABLE`
    /// * `HARDWARE_FLOW_CONTROL_ENABLE`
    pub fn set_control_bits(&self, control: ControlBits) -> Result<(), Status> {
        unsafe {
            match (self.set_control_bits)(self, control.bits()) {
                Status::Success => Ok(()),
                e => Err(e),
            }
        }
    }

    /// Retrieve the serial device's control bits.
    pub fn get_control_bits(&self) -> Result<ControlBits, Status> {
        let mut bits: u32 = 0;
        unsafe {
            match (self.get_control_bits)(self, &mut bits) {
                Status::Success => Ok(ControlBits::from_bits_truncate(bits)),
                e => Err(e),
            }
        }
    }

    /// Write some data to the serial device.
    pub fn write_raw(&self, data: *const u8, length: usize) -> Result<usize, Status> {
        let mut new_length = length;
        unsafe {
            match (self.write)(self, &mut new_length, data as *const CVoid) {
                Status::Success => Ok(new_length),
                e => Err(e),
            }
        }
    }

    /// Write a string to the serial device.
    pub fn write(&self, data: &str) -> Result<usize, Status> {
        self.write_raw(data.as_ptr(), data.len())
    }

    fn read_raw(&self, buf: *mut u8, length: usize) -> Result<Option<(*const u8, usize)>, Status> {
        let mut new_length = length;
        unsafe {
            match (self.read)(self, &mut new_length, buf as *mut CVoid) {
                Status::Success => Ok(Some((buf, new_length))),
                Status::Timeout => {
                    //::get_system_table().console().write("timeout");
                    Ok(None)
                }
                e => Err(e),
            }
        }
    }

    /// Read `length` bytes from the serial device.
    /// Note: The returned pointer is allocated with `allocate_pool`, and it is the caller's
    /// responsibility to free at some point.
    pub fn read_bytes(&self, length: usize) -> Result<Option<&[u8]>, Status> {
        let buf_ptr_result = ::get_system_table().boot_services().allocate_pool(length);
        match buf_ptr_result {
            Ok(buf_ptr) => {
                let result = self.read_raw(buf_ptr, length);
                result
                    .map(|option| {
                        option.map(|(ptr, len)| unsafe { slice::from_raw_parts(ptr, len) })
                    })
                    .map_err(|e| {
                        ::get_system_table().boot_services().free_pool(buf_ptr);
                        e
                    })
            }
            Err(e) => Err(e),
        }
    }
}

pub struct SerialIOProtocol {
    raw_protocol: &'static RawSerialIOProtocol,
    baud_rate: Option<u64>,
    receive_fifo_depth: Option<u32>,
    timeout: Option<u32>,
    parity: Option<ParityType>,
    data_bits: Option<u8>,
    stop_bits: Option<StopBits>,
}

impl SerialIOProtocol {
    /// Create and reset a new SerialIOProtocol struct.
    pub fn new() -> Result<SerialIOProtocol, Status> {
        let bs = ::get_system_table().boot_services();
        bs.locate_protocol::<RawSerialIOProtocol>(0 as *const CVoid)
            .and_then(|protocol| {
                if let Err(e) = protocol.reset() {
                    return Err(e);
                }

                Ok(SerialIOProtocol {
                    raw_protocol: protocol,
                    baud_rate: None,
                    receive_fifo_depth: None,
                    timeout: None,
                    parity: None,
                    data_bits: None,
                    stop_bits: None,
                })
            })
    }

    fn set_attributes(&self) -> Result<(), Status> {
        match self.raw_protocol.set_attributes(
            self.baud_rate,
            self.receive_fifo_depth,
            self.timeout,
            self.parity,
            self.data_bits,
            self.stop_bits,
        ) {
            Status::Success => Ok(()),
            e => Err(e),
        }
    }

    /// Update this serial device's attributes. Unlike the UEFI protocol, this function can be
    /// called incrementally - that is, passing `None` for any attribute will leave it unchanged
    /// from its current value, instead of resetting it to the device default.
    pub fn update_attributes(
        &mut self,
        baud_rate: Option<u64>,
        receive_fifo_depth: Option<u32>,
        timeout: Option<u32>,
        parity: Option<ParityType>,
        data_bits: Option<u8>,
        stop_bits: Option<StopBits>,
    ) -> Result<(), Status> {
        self.baud_rate = baud_rate.or(self.baud_rate);
        self.receive_fifo_depth = receive_fifo_depth.or(self.receive_fifo_depth);
        self.timeout = timeout.or(self.timeout);
        self.parity = parity.or(self.parity);
        self.data_bits = data_bits.or(self.data_bits);
        self.stop_bits = stop_bits.or(self.stop_bits);

        self.set_attributes()
    }

    /// Write a string to the serial device.
    pub fn write(&self, data: &str) -> Result<usize, Status> {
        // GRUB sets the attributes on the serial device on each read or write, so we will too.
        // This ensures that the cached attributes present in this wrapper structure will be
        // reflected by the serial device.
        self.set_attributes().and_then(|_| {
            self.raw_protocol.write(data)
        })
    }

    /// Read a slice of bytes from the serial device. The resulting slice is allocated with
    /// `allocate_pool` and is the caller's responsibility to free.
    pub fn read_bytes(&self, length: usize) -> Result<Option<&[u8]>, Status> {
        // GRUB sets the attributes on the serial device on each read or write, so we will too.
        // This ensures that the cached attributes present in this wrapper structure will be
        // reflected by the serial device.
        self.set_attributes().and_then(|_| {
            self.raw_protocol.read_bytes(length)
        })
    }
}
