use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::catch_unwind;
use crate::codec::encode_json;

#[repr(C)]
pub struct AdvancedQJsonResult {
    pub data: *mut u8,
    pub size: usize,
    pub total_quats: usize,
    pub error_code: i32, // 0 = Success, Negative = Error Code, -99 = Panic caught
}

#[unsafe(no_mangle)]
pub extern "C" fn qjson_advanced_encode(json_ptr: *const c_char) -> AdvancedQJsonResult {
    let result = catch_unwind(|| {
        if json_ptr.is_null() {
            return AdvancedQJsonResult {
                data: std::ptr::null_mut(),
                size: 0,
                total_quats: 0,
                error_code: -1,
            };
        }

        let c_str = unsafe { CStr::from_ptr(json_ptr) };
        let json_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return AdvancedQJsonResult {
                data: std::ptr::null_mut(),
                size: 0,
                total_quats: 0,
                error_code: -2,
            },
        };

        match encode_json(json_str) {
            Ok(mut buf) => {
                let size = buf.data.len();
                let total_quats = buf.total_quats;
                let ptr = buf.data.as_mut_ptr();
                std::mem::forget(buf.data);

                AdvancedQJsonResult {
                    data: ptr,
                    size,
                    total_quats,
                    error_code: 0,
                }
            }
            Err(_) => AdvancedQJsonResult {
                data: std::ptr::null_mut(),
                size: 0,
                total_quats: 0,
                error_code: -3,
            },
        }
    });

    match result {
        Ok(res) => res,
        Err(_) => AdvancedQJsonResult {
            data: std::ptr::null_mut(),
            size: 0,
            total_quats: 0,
            error_code: -99, // Intercepted catastrophic Rust panic
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn qjson_advanced_free(data: *mut u8, size: usize, capacity: usize) {
    if data.is_null() || capacity == 0 {
        return;
    }
    unsafe {
        // Safely reconstruct the vector and let Rust drop it out of scope
        let _ = Vec::from_raw_parts(data, size, capacity);
    }
}