# Q-JSON: High-Performance Quaternary JSON Library

Q-JSON is an advanced, high-performance JSON processing and compression library written in **Rust**. It implements **quaternary logic** by packing 2 bits into a single quaternary digit ("quat": `00`, `01`, `10`, `11`), compressing 4 quats into a single standard `u8` byte for optimal memory density. 

Designed for enterprise workloads, it features robust Abstract Syntax Tree (AST) parsing, comprehensive error management, and a safe C-compatible Foreign Function Interface (FFI) to seamlessly integrate with C, Python, Go, and TypeScript/JavaScript.

---

### Core Modules:
- **`quat.rs`**: Handles low-level bit operations, packing 4 quats (`Q0`, `Q1`, `Q2`, `Q3`) into an 8-bit byte with inline optimizations.
- **`ast.rs`**: Uses `serde_json` to traverse JSON syntax trees and map structural boundaries (objects, arrays, separators, values) into quaternary states.
- **`codec.rs`**: Manages high-level encoding and decoding pipelines alongside custom `std::error::Error` implementations.
- **`ffi.rs`**: Exposes a safe C ABI boundary protected by `std::panic::catch_unwind` to prevent undefined behavior across language borders.

---

## Licensing

This project is dual-licensed:

* **Open Source (AGPLv3):** Free to use for open-source projects. If you use or modify this library in your software or network service, your source code must also be open-source under AGPLv3.
* **Commercial License:** Required if you want to use `quat_json` in closed-source or proprietary commercial software without AGPLv3 restrictions.

For commercial licensing, contact: **strahinjastojanovic826@gmail.com**

---

To compile the release dynamic library for cross-language binding:
```bash
cargo build --release
```

---

## FFI Usage & Safety

The library exports standard C functions designed for external bindings:

- **`quat_json_advanced_encode(const char* json_ptr) -> AdvancedQJsonResult`**: Encodes a null-terminated JSON string into a packed quaternary buffer.
- **`quat_json_advanced_free(uint8_t* data, size_t size)`**: Safely deallocates memory allocated by Rust.

### Error Codes:
- `0`: Success
- `-1` / `-2`: Invalid pointer or string conversion error
- `-3`: Encoding or parsing failure
- `-99`: Catastrophic panic intercepted safely via FFI boundary

---

## Testing

The library includes an extensive test suite ranging from basic bit-level unit tests to deep nesting and massive payload memory stress tests. 

Run tests via Cargo:
```bash
cargo test
```

### Importing and Using in a Rust Project

Here is the complete example of how to import and use the library directly inside another **Rust** project, written in English.

#### 1. Configuration in `Cargo.toml`

If the library is located locally on your machine, add the path to the consumer's `Cargo.toml` file:

```toml
[dependencies]
quat_json = "0.3.0"
```

#### 2. Usage in Rust Code (`main.rs`)

```rust
use qjson::codec::encode_json;
use qjson::quat::unpack_quats;

fn main() {
    let json_input = r#"{
        "name": "qjson",
        "language": "rust",
        "supported": true
    }"#;

    // Encode JSON into quaternary buffer
    match encode_json(json_input) {
        Ok(buffer) => {
            println!("Successfully encoded JSON!");
            println!("Total quaternary digits: {}", buffer.total_quats);
            println!("Packed bytes length: {}", buffer.data.len());

            // Optional: Unpack quats back to verify
            let unpacked = unpack_quats(&buffer.data, buffer.total_quats);
            println!("Unpacked quats count: {}", unpacked.len());
        }
        Err(e) => {
            eprintln!("Failed to encode JSON: {}", e);
        }
    }
}
```

### 1. C Integration (`main.c`)

```c
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

typedef struct {
    uint8_t* data;
    size_t size;
    size_t total_quats;
    int32_t error_code;
} AdvancedQJsonResult;

extern AdvancedQJsonResult qjson_advanced_encode(const char* json_ptr);
extern void qjson_advanced_free(uint8_t* data, size_t size);

int main() {
    const char* json_data = "{\"name\": \"qjson\", \"version\": 2}";
    AdvancedQJsonResult res = qjson_advanced_encode(json_data);

    if (res.error_code != 0) {
        printf("Encoding failed with error code: %d\n", res.error_code);
        return 1;
    }

    printf("Encoded successfully! Total quats: %zu, Bytes size: %zu\n", res.total_quats, res.size);

    // Free memory allocated by Rust
    qjson_advanced_free(res.data, res.size);
    return 0;
}
```

---

### 2. Python Integration (`main.py`)

```python
import ctypes
import os

# Load compiled Rust library
lib_path = os.path.abspath("./target/release/libqjson.so") # Use .dylib on macOS, .dll on Windows
lib = ctypes.CDLL(lib_path)

class AdvancedQJsonResult(ctypes.Structure):
    _fields_ = [
        ("data", ctypes.POINTER(ctypes.c_ubyte)),
        ("size", ctypes.c_size_t),
        ("total_quats", ctypes.c_size_t),
        ("error_code", ctypes.c_int32)
    ]

lib.qjson_advanced_encode.argtypes = [ctypes.c_char_p]
lib.qjson_advanced_encode.restype = AdvancedQJsonResult

lib.qjson_advanced_free.argtypes = [ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t]
lib.qjson_advanced_free.restype = None

def encode_json(json_str: str):
    c_str = json_str.encode('utf-8')
    res = lib.qjson_advanced_encode(c_str)
    
    if res.error_code != 0:
        raise RuntimeError(f"Q-JSON encoding failed with code {res.error_code}")
        
    buffer_data = ctypes.string_at(res.data, res.size)
    lib.qjson_advanced_free(res.data, res.size)
    return buffer_data

if __name__ == "__main__":
    packed = encode_json('{"python": true, "code": 42}')
    print("Python Packed Hex:", packed.hex())
```

---

### 3. Go Integration (`main.go`)

```go
package main

/*
#cgo LDFLAGS: -L./target/release -lqjson
#include <stdlib.h>
#include <stdint.h>

typedef struct {
    uint8_t* data;
    size_t size;
    size_t total_quats;
    int32_t error_code;
} AdvancedQJsonResult;

extern AdvancedQJsonResult qjson_advanced_encode(const char* json_str);
extern void qjson_advanced_free(uint8_t* data, size_t size);
*/
import "C"
import (
	"fmt"
	"unsafe"
)

func EncodeJSON(jsonStr string) ([]byte, error) {
	cStr := C.CString(jsonStr)
	defer C.free(unsafe.Pointer(cStr))

	res := C.qjson_advanced_encode(cStr)
	if res.error_code != 0 {
		return nil, fmt.Errorf("encoding error code: %d", res.error_code)
	}
	defer C.qjson_advanced_free(res.data, res.size)

	return C.GoBytes(unsafe.Pointer(res.data), C.int(res.size)), nil
}

func main() {
	packed, err := EncodeJSON(`{"language": "go", "active": true}`)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Go Packed Hex: %x\n", packed)
}
```

---

### 4. TypeScript / JavaScript Integration (`index.ts`)

```typescript
import koffi from 'koffi';

// Load shared library
const lib = koffi.load('./target/release/libqjson.so');

// Define C structure matching AdvancedQJsonResult
const AdvancedQJsonResult = koffi.struct('AdvancedQJsonResult', {
    data: 'uint8_t *',
    size: 'size_t',
    total_quats: 'size_t',
    error_code: 'int32_t'
});

// Bind FFI functions
const qjson_advanced_encode = lib.func('AdvancedQJsonResult qjson_advanced_encode(const char* json_str)');
const qjson_advanced_free = lib.func('void qjson_advanced_free(uint8_t* data, size_t size)');

export function encodeJsonQuaternary(jsonStr: string): Buffer {
    const res = qjson_advanced_encode(jsonStr);

    if (res.error_code !== 0) {
        throw new Error(`Q-JSON error code: ${res.error_code}`);
    }

    const buffer = Buffer.from(koffi.decode(res.data, 'uint8_t', res.size));
    qjson_advanced_free(res.data, res.size);

    return buffer;
}

// Usage
const packedResult = encodeJsonQuaternary('{"framework": "typescript", "quat": true}');
console.log("TS Packed Hex:", packedResult.toString('hex'));
```
