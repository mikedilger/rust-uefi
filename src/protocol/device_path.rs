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

use base::Status;
use console::SimpleTextOutput;
use guid::Guid;
use protocol::Protocol;
use void::CVoid;
use util::{utf16_ptr_to_str, str_to_utf16_ptr};

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

/// GUID for UEFI protocol for converting text to a DevicePath
pub static EFI_DEVICE_PATH_FROM_TEXT_PROTOCOL_GUID: Guid = Guid(0x5C99A21, 0xC70F, 0x4AD2, [0x8A,0x5F,0x35,0xDF,0x33,0x43,0xF5,0x1E]);

/// GUID for UEFI protocol for device path utilities
pub static EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID: Guid = Guid(0x379BE4E, 0xD706, 0x437D, [0xB0,0x37,0xED,0xB8,0x2F,0xB7,0x72,0xA4]);

#[derive(Debug)]
#[repr(C, packed)]
pub struct DevicePathProtocol {
    pub type_: u8,
    pub sub_type: u8,
    pub length: [u8; 2],
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

    pub fn len(&self) -> u16 {
        (self.length[0] as u16) | ((self.length[1] as u16) << 8)
    }

    // Get a pointer to the DevicePathProtocol after this one, accounting for however big this one
    // might be. This is very unsafe.
    pub unsafe fn next(&self) -> &mut DevicePathProtocol {
        let this_u8 = self as *const DevicePathProtocol as *const u8;
        &mut *(this_u8.offset(self.len() as isize) as *mut DevicePathProtocol)
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
    pub fn device_path_node_to_text(&self, device_node: *const DevicePathProtocol, display_only: bool, allow_shortcuts: bool) -> Result<&str, Status> {
        let chars: *const u16 = unsafe { (self.device_path_node_to_text)(device_node, display_only as u8, allow_shortcuts as u8) };
        let out = utf16_ptr_to_str(chars);
        ::get_system_table().boot_services().free_pool(chars);
        out
    }

    pub fn device_path_to_text(&self, device_node: *const DevicePathProtocol, display_only: bool, allow_shortcuts: bool) -> Result<&str, Status> {
        let chars: *const u16 = unsafe { (self.device_path_to_text)(device_node, display_only as u8, allow_shortcuts as u8) };
        let out = utf16_ptr_to_str(chars);
        ::get_system_table().boot_services().free_pool(chars);
        out
    }

    pub fn print_device_path_node(device_node: *const DevicePathProtocol, display_only: bool, allow_shortcuts: bool) -> Result<(), Status> {
        let system_table = ::get_system_table();
        let boot_services = system_table.boot_services();

        boot_services
            .locate_protocol::<DevicePathToTextProtocol>(0 as *const CVoid)
            .and_then(|this| {
                          this.device_path_node_to_text(device_node, display_only, allow_shortcuts)
                              .map(|result| {
                                       system_table.console().write(result);
                                       ()
                                   })
                     })
    }

    pub fn print_device_path(device_node: *const DevicePathProtocol, display_only: bool, allow_shortcuts: bool) -> Result<(), Status> {
        let system_table = ::get_system_table();
        let boot_services = system_table.boot_services();

        boot_services
            .locate_protocol::<DevicePathToTextProtocol>(0 as *const CVoid)
            .and_then(|this| {
                          this.device_path_to_text(device_node, display_only, allow_shortcuts)
                              .map(|result| {
                                       system_table.console().write(result);
                                       ()
                                   })
                     })
    }
}

#[repr(C)]
pub struct DevicePathFromTextProtocol {
    text_to_device_path_node: unsafe extern "win64" fn(text: *const u16) -> *const DevicePathProtocol,
    text_to_device_path: unsafe extern "win64" fn(text: *const u16) -> *const DevicePathProtocol,
}

impl Protocol for DevicePathFromTextProtocol {
    fn guid() -> &'static Guid {
        &EFI_DEVICE_PATH_FROM_TEXT_PROTOCOL_GUID
    }
}

impl DevicePathFromTextProtocol {
    pub fn text_to_device_path_node(&self, path: &str) -> Result<&DevicePathProtocol, Status> {
        str_to_utf16_ptr(path)
            .map(|utf16_str| {
                let out = unsafe { &*((self.text_to_device_path_node)(utf16_str)) };
                // FIXME(csssuf)
                // Ideally, at this point, we'd free utf16_str. However, free_pool(utf16_str) seems
                // to hang here for unknown reasons. So we leak it.
                out
            })
    }

    pub fn text_to_device_path(&self, path: &str) -> Result<&DevicePathProtocol, Status> {
        str_to_utf16_ptr(path)
            .map(|utf16_str| {
                let out = unsafe { &*((self.text_to_device_path)(utf16_str)) };
                // FIXME(csssuf)
                // Ideally, at this point, we'd free utf16_str. However, free_pool(utf16_str) seems
                // to hang here for unknown reasons. So we leak it.
                out
            })
    }
}

#[repr(C)]
pub struct DevicePathUtilitiesProtocol {
    get_device_path_size: *const CVoid,
    duplicate_device_path:
        unsafe extern "win64" fn(src: *const DevicePathProtocol) -> *mut DevicePathProtocol,
    append_device_path: unsafe extern "win64" fn(src1: *const DevicePathProtocol, src2: *const DevicePathProtocol) -> *const DevicePathProtocol,
    append_device_node: unsafe extern "win64" fn(path: *const DevicePathProtocol, node: *const DevicePathProtocol) -> *const DevicePathProtocol,
    append_device_path_instance: *const CVoid,
    get_next_device_path_instance: *const CVoid,
    is_device_path_multi_instance: *const CVoid,
    create_device_node: unsafe extern "win64" fn(node_type: u8, node_subtype: u8, node_length: u16) -> *const DevicePathProtocol
}

impl Protocol for DevicePathUtilitiesProtocol {
    fn guid() -> &'static Guid {
        &EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID
    }
}

impl DevicePathUtilitiesProtocol {
    pub fn duplicate_device_path(&self, src: &DevicePathProtocol) -> Result<&mut DevicePathProtocol, Status> {
        unsafe {
            let out = (self.duplicate_device_path)(src);
            if out.is_null() {
                return Err(Status::OutOfResources);
            }
            Ok(&mut *out)
        }
    }

    pub fn append_device_path(&self, src1: *const DevicePathProtocol, src2: *const DevicePathProtocol) -> Result<*const DevicePathProtocol, Status> {
        unsafe {
            let out = (self.append_device_path)(src1, src2);
            if out == 0 as *const DevicePathProtocol {
                // `out` being a null pointer indicates, according to the spec, that "memory could
                // not be allocate[sic]." Whether that's due to memory conditions, bad parameters
                // being passed in, or another reason is unspecified. Unless the caller passes in
                // a massive DevicePathProtocol, it's unlikely that it's due to the actual
                // parameters, so error here is represented as OutOfResources.
                return Err(Status::OutOfResources);
            }
            Ok(out)
        }
    }

    pub fn append_device_node(&self, path: *const DevicePathProtocol, node: *const DevicePathProtocol) -> Result<*const DevicePathProtocol, Status> {
        unsafe {
            let out = (self.append_device_node)(path, node);
            if out == 0 as *const DevicePathProtocol {
                // See comment in append_device_path.
                return Err(Status::OutOfResources);
            }
            Ok(out)
        }
    }

    pub fn create_device_node<T: Into<u8>, U: Into<u8>>(&self, node_type: T, node_subtype: U, node_length: u16) -> Result<*const DevicePathProtocol, Status> {
        unsafe {
            let out = (self.create_device_node)(node_type.into(), node_subtype.into(), node_length);
            if out == 0 as *const DevicePathProtocol {
                return Err(Status::InvalidParameter);
            }
            Ok(out)
        }
    }
}
