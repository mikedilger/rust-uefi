use base::{Handle, MemoryType};
use console::SimpleTextOutput;
use guid::Guid;
use void::{CVoid, NotYetDef};

pub trait Protocol {
    fn guid() -> &'static Guid;
}

/// GUID for UEFI protocol for loaded images
pub static EFI_LOADED_IMAGE_PROTOCOL_GUID: Guid = Guid(0x5B1B31A1, 0x9562, 0x11d2, [0x8E,0x3F,0x00,0xA0,0xC9,0x69,0x72,0x3B]);

#[derive(Debug)]
#[repr(C)]
pub struct LoadedImageProtocol {
    revision: u32,
    parent_handle: Handle,
    system_table: *const NotYetDef,
    device_handle: Handle,
    file_path: *const NotYetDef,
    __reserved: *const NotYetDef,
    load_options_size: u32,
    load_options: *const NotYetDef,
    pub image_base: usize,
    pub image_size: u64,
    image_code_type: MemoryType,
    image_data_type: MemoryType,

    //unload: unsafe extern "win64" fn(handle: ::base::Handle),
    unload: *const NotYetDef,
}

impl Protocol for LoadedImageProtocol {
    fn guid() -> &'static Guid {
        return &EFI_LOADED_IMAGE_PROTOCOL_GUID;
    }
}

