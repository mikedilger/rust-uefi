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
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::mem;

use console::SimpleTextOutput;
use guid::Guid;
use protocol::Protocol;
use void::CVoid;

#[repr(u8)]
pub enum DevicePathTypes {
    Hardware = 0x01,
    ACPI = 0x02,
    Messaging = 0x03,
    Media = 0x04,
    BIOSBootSpecification = 0x05,
    End = 0x7F
}

impl Into<u8> for DevicePathTypes {
    fn into(self) -> u8 {
        self as u8
    }
}

#[repr(u8)]
pub enum HardwareSubTypes {
    PCI = 0x01,
    PCCARD = 0x02,
    MemoryMapped = 0x03,
    Vendor = 0x04,
    Controller = 0x05,
    BMC = 0x06
}

impl Into<u8> for HardwareSubTypes {
    fn into(self) -> u8 {
        self as u8
    }
}

#[repr(u8)]
pub enum ACPISubTypes {
    ACPIDevicePath = 0x01,
    ExpandedACPIDevicePath = 0x02,
    _ADR = 0x03
}

impl Into<u8> for ACPISubTypes {
    fn into(self) -> u8 {
        self as u8
    }
}

#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum MessagingSubTypes {
    ATAPI = 0x01,
    SCSI = 0x02,
    FibreChannel = 0x03,
    // The EFI specification calls this type "1394", but that won't work as an enum variant for
    // Rust.
    FireWire = 0x4,
    USB = 0x5,
    I2O = 0x6,
    Infiniband = 0x9,
    Vendor = 0xA,
    MACAddress = 0xB,
    IPv4 = 0xC,
    IPv6 = 0xD,
    UART = 0xE,
    USBClass = 0xF,
    USBWWID = 0x10,
    DeviceLogicalUnit = 0x11,
    SATA = 0x12,
    iSCSI = 0x13,
    Vlan = 0x14,
    FibreChannelEx = 0x15,
    SASEx = 0x16,
    NVMExpressNamespace = 0x17,
    URI = 0x18,
    UFS = 0x19,
    SD = 0x1A,
    Bluetooth = 0x1B,
    WiFi = 0x1C,
    eMMC = 0x1D
}

impl Into<u8> for MessagingSubTypes {
    fn into(self) -> u8 {
        self as u8
    }
}

#[repr(u8)]
pub enum MediaSubTypes {
    HardDrive = 0x1,
    CDROM = 0x2,
    Vendor = 0x3,
    FilePath = 0x4,
    MediaProtocol = 0x5,
    PIWGFirmwareFile = 0x6,
    PIWGFirmwareVolume = 0x7,
    RelativeOffsetRange = 0x8,
    RAMDisk = 0x9
}

impl Into<u8> for MediaSubTypes {
    fn into(self) -> u8 {
        self as u8
    }
}

#[repr(u8)]
pub enum BIOSSubTypes {
    BIOSBootSpecification = 0x1
}

impl Into<u8> for BIOSSubTypes {
    fn into(self) -> u8 {
        self as u8
    }
}

#[repr(u8)]
pub enum EndPathSubTypes {
    EndInstance = 0x01,
    EndEntirePath = 0xFF
}

impl Into<u8> for EndPathSubTypes {
    fn into(self) -> u8 {
        self as u8
    }
}

/// GUID for UEFI protocol for device paths
pub static EFI_DEVICE_PATH_PROTOCOL_GUID: Guid = Guid(0x09576E91, 0x6D3F, 0x11D2, [0x8E,0x39,0x00,0xA0,0xC9,0x69,0x72,0x3B]);

/// GUID for UEFI protocol for converting a DevicePath to text
pub static EFI_DEVICE_PATH_TO_TEXT_PROTOCOL_GUID: Guid = Guid(0x8B843E20, 0x8132, 0x4852, [0x90,0xCC,0x55,0x1A,0x4E,0x4A,0x7F,0x1C]);

#[derive(Debug)]
#[repr(C)]
pub struct DevicePathProtocol {
    type_: u8,
    sub_type: u8,
    length: [u8; 2],
}

impl Protocol for DevicePathProtocol {
    fn guid() -> &'static Guid {
        &EFI_DEVICE_PATH_PROTOCOL_GUID
    }
}

impl DevicePathProtocol {
    fn data<T>(&self) -> *const T {
        unsafe {
            let self_u8: *const u8 = mem::transmute(self);
            mem::transmute(self_u8.offset(4))
        }
    }
}

#[repr(C)]
pub struct DevicePathToTextProtocol {
    device_path_node_to_text: unsafe extern "win64" fn(device_node: *const DevicePathProtocol, display_only: u8, allow_shortcuts: u8) -> *const u16,
    device_path_to_text: unsafe extern "win64" fn(device_path: *const DevicePathProtocol, display_only: u8, allow_shortcuts: u8) -> *const u16
}

impl Protocol for DevicePathToTextProtocol {
    fn guid() -> &'static Guid {
        &EFI_DEVICE_PATH_TO_TEXT_PROTOCOL_GUID
    }
}

impl DevicePathToTextProtocol {
    pub fn device_path_node_to_text(&self, device_node: *const DevicePathProtocol, display_only: bool, allow_shortcuts: bool) -> *const u16 {
        unsafe {
            (self.device_path_node_to_text)(device_node, display_only as u8, allow_shortcuts as u8)
        }
    }

    pub fn device_path_to_text(&self, device_node: *const DevicePathProtocol, display_only: bool, allow_shortcuts: bool) -> *const u16 {
        unsafe {
            (self.device_path_to_text)(device_node, display_only as u8, allow_shortcuts as u8)
        }
    }

    pub fn print_device_path_node(device_node: *const DevicePathProtocol, display_only: bool, allow_shortcuts: bool) {
        let system_table = ::get_system_table();
        let boot_services = system_table.boot_services();

        let this: &'static DevicePathToTextProtocol = boot_services.locate_protocol(0 as *const CVoid).unwrap();

        let ptr = this.device_path_node_to_text(device_node, display_only, allow_shortcuts);

        system_table.console().write_raw(ptr);
        system_table.boot_services().free_pool(ptr);
    }

    pub fn print_device_path(device_node: *const DevicePathProtocol, display_only: bool, allow_shortcuts: bool) {
        let system_table = ::get_system_table();
        let boot_services = system_table.boot_services();

        let this: &'static DevicePathToTextProtocol = boot_services.locate_protocol(0 as *const CVoid).unwrap();

        let ptr = this.device_path_to_text(device_node, display_only, allow_shortcuts);

        system_table.console().write_raw(ptr);
        system_table.boot_services().free_pool(ptr);
    }
}

