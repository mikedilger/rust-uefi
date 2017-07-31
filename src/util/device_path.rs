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

use base::Status;
use protocol::{DevicePathProtocol, DevicePathUtilitiesProtocol, DevicePathTypes, EndPathSubTypes,
               MediaSubTypes};
use void::CVoid;
use util::*;

pub fn create_file_device_node(filename: &str) -> Result<&DevicePathProtocol, Status> {
    str_to_utf16_ptr(filename).and_then(|filename_ptr| {
        let filename_len = utf16_strlen(filename_ptr);
        let node_size_bytes = 4 + (filename_len + 1) * 2;

        ::get_system_table()
            .boot_services()
            .locate_protocol::<DevicePathUtilitiesProtocol>(0 as *const CVoid)
            .and_then(|utilities| {
                utilities.create_device_node(DevicePathTypes::Media, MediaSubTypes::FilePath, node_size_bytes as u16)
                    .map(|node_ptr| {
                        let node_filename_ptr: *mut u16 = unsafe { (node_ptr as *const u8).offset(4) as *mut u16 };

                        for i in 0..filename_len as isize{
                            unsafe {
                                *node_filename_ptr.offset(i) = *filename_ptr.offset(i);
                            }
                        }
                        unsafe { *node_filename_ptr.offset(filename_len as isize) = 0 };

                        unsafe { &*node_ptr }
                    })
            })
    })
}

/// Get the "parent" of a given device path - i.e., take all but the last DevicePathProtocol
/// instance in the entire device path. This function allocates memory with `allocate_pool`, and it
/// is the caller's responsibility to free it.
pub fn parent_device_path(
    src_device_path: &DevicePathProtocol,
) -> Result<&mut DevicePathProtocol, Status> {
    // If the device path we're given is already an end, there's nothing we can do.
    if src_device_path.type_ ==  DevicePathTypes::End.into() {
        return Err(Status::InvalidParameter);
    }

    ::get_system_table()
        .boot_services()
        .locate_protocol::<DevicePathUtilitiesProtocol>(0 as *const CVoid)
        .and_then(|utilities| {
            utilities.duplicate_device_path(src_device_path).map(
                |device_path| {
                    let mut this_device_path_ptr = device_path as *mut DevicePathProtocol;
                    loop {
                        let next_device_path_ptr = unsafe { (&*this_device_path_ptr).next() };

                        unsafe {
                            if (*next_device_path_ptr).type_ == DevicePathTypes::End.into() {
                                (*this_device_path_ptr).type_ = DevicePathTypes::End.into();
                                (*this_device_path_ptr).sub_type = EndPathSubTypes::EndEntirePath
                                    .into();
                                (*this_device_path_ptr).length = [4, 0];
                                return device_path;
                            }
                        }

                        this_device_path_ptr = next_device_path_ptr;
                    }
                },
            )
        })
}
