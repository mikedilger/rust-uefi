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
use protocol::{DevicePathProtocol, DevicePathUtilitiesProtocol, DevicePathTypes, MediaSubTypes};
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
