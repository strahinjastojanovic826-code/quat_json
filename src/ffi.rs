//! C ABI and FFI boundary for `quat_json`.
//!
//! # Safety & Ownership Invariants
//! - All `extern "C"` entry points wrap execution in `catch_unwind` to prevent Rust panics
//!   from unwinding across foreign stack frames.
//! - Input C string pointers (`json_ptr`) must be non-null when encoding and point to valid,
//!   null-terminated UTF-8 data for the duration of the call.
//! - Output buffers returned in `AdvancedQJsonResult` (`data`, `size`, `capacity`) are allocated
//!   by Rust (`Vec<u8>`) and must only be freed once via `qjson_advanced_free` or
//!   `qjson_advanced_free_result` using the matching `size` and `capacity`.
//! - Result struct and free function pointers passed from callers must be properly aligned,
//!   valid pointers, and must not be freed more than once.

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
    pub capacity: usize,
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
                capacity: 0,
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
                capacity: 0,
            },
        };

        match encode_json(json_str) {
            Ok(mut buf) => {
                let size = buf.data.len();
                let total_quats = buf.total_quats;
                let capacity = buf.data.capacity();
                let ptr = buf.data.as_mut_ptr();
                std::mem::forget(buf.data);

                AdvancedQJsonResult {
                    data: ptr,
                    size,
                    total_quats,
                    error_code: 0,
                    capacity,
                }
            }
            Err(_) => AdvancedQJsonResult {
                data: std::ptr::null_mut(),
                size: 0,
                total_quats: 0,
                error_code: -3,
                capacity: 0,
            },
        }
    });

    match result {
        Ok(res) => res,
        Err(_) => AdvancedQJsonResult {
            data: std::ptr::null_mut(),
            size: 0,
            total_quats: 0,
            capacity: 0,
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

#[unsafe(no_mangle)]
pub extern "C" fn qjson_advanced_free_result(result: *mut AdvancedQJsonResult) {
    if result.is_null() {
        return;
    }
    let _ = catch_unwind(|| unsafe {
        let res = &mut *result;
        if !res.data.is_null() && res.capacity > 0 {
            let _ = Vec::from_raw_parts(res.data, res.size, res.capacity);
        }
        res.data = std::ptr::null_mut();
        res.size = 0;
        res.capacity = 0;
        res.total_quats = 0;
        res.capacity = 0;
    });
}