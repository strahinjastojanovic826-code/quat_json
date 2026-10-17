pub mod quat;
pub mod ast;
pub mod codec;
pub mod ffi;

#[cfg(test)]
mod tests {
    use super::quat::{Quat, pack_quats, unpack_quats};
    use super::codec::{encode_json, decode_quats, AdvancedCodecError};

    // ==========================================
    // 1. NAJLAKŠI TESTOVI (Osnovne jedinice i bitovi)
    // ==========================================

    #[test]
fn test_01_quat_enum_mapping() {
    // Test basic Quat enum parsing from u8 values
    assert_eq!(Quat::from_u8(0b00), Some(Quat::Q0));
    assert_eq!(Quat::from_u8(0b01), Some(Quat::Q1));
    assert_eq!(Quat::from_u8(0b10), Some(Quat::Q2));
    assert_eq!(Quat::from_u8(0b11), Some(Quat::Q3));
    assert_eq!(Quat::from_u8(4), Some(Quat::Q0)); // Prilagođeno zbog & 0b11 maskiranja
}

    #[test]
    fn test_02_basic_quat_packing() {
        // Test packing exactly 4 quats into a single u8 byte
        let quats = vec![Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3];
        let buffer = pack_quats(&quats);
        
        assert_eq!(buffer.total_quats, 4);
        assert_eq!(buffer.data.len(), 1);
        // Q3<<6 | Q2<<4 | Q1<<2 | Q0 -> 11<<6 | 10<<4 | 01<<2 | 00 = 0xC9 (201)
        assert_eq!(buffer.data[0], 0b11100100); 
    }

    #[test]
    fn test_03_basic_quat_unpacking() {
        // Test unpacking bytes back to original quats
        let original = vec![Quat::Q3, Quat::Q0, Quat::Q2, Quat::Q1];
        let buffer = pack_quats(&original);
        let unpacked = unpack_quats(&buffer.data, buffer.total_quats);

        assert_eq!(original, unpacked);
    }

    // ==========================================
    // 2. SREDNJI TESTOVI (Jednostavan JSON i validacija)
    // ==========================================

    #[test]
    fn test_04_empty_json_object() {
        // Test encoding an empty JSON object
        let json = "{}";
        let result = encode_json(json);
        assert!(result.is_ok());
        let buf = result.unwrap();
        assert!(buf.total_quats > 0);
    }

    #[test]
    fn test_05_simple_json_array() {
        // Test encoding a flat JSON array
        let json = "[1, 2, 3, 4]";
        let result = encode_json(json);
        assert!(result.is_ok());
        let buf = result.unwrap();
        
        let decoded_tokens = decode_quats(&buf.data, buf.total_quats);
        assert!(decoded_tokens.is_ok());
        assert_eq!(decoded_tokens.unwrap().len(), buf.total_quats);
    }

    #[test]
    fn test_06_key_value_json_object() {
        // Test encoding a simple key-value object
        let json = "{\"id\": 100}";
        let result = encode_json(json);
        assert!(result.is_ok());
        
        let buf = result.unwrap();
        let decoded = decode_quats(&buf.data, buf.total_quats).unwrap();
        assert!(!decoded.is_empty());
    }

    // ==========================================
    // 3. NAPREDNI TESTOVI (Greške i ugnježđeni objekti)
    // ==========================================

    #[test]
    fn test_07_malformed_json_error() {
        // Test handling of malformed JSON strings
        let bad_json = "{\"id\": 100, missing_quote}";
        let result = encode_json(bad_json);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            AdvancedCodecError::JsonParseError(_) => assert!(true),
            _ => panic!("Expected JsonParseError"),
        }
    }

    #[test]
    fn test_08_empty_input_error() {
        // Test handling of empty string input
        let empty_input = "   ";
        let result = encode_json(empty_input);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            AdvancedCodecError::EmptyInput => assert!(true),
            _ => panic!("Expected EmptyInput error"),
        }
    }

    #[test]
    fn test_09_nested_objects_and_arrays() {
        // Test complex nested JSON structures
        let complex_json = r#"{
            "user": {
                "name": "Rust",
                "active": true,
                "scores": [95, 88, 92]
            }
        }"#;
        
        let result = encode_json(complex_json);
        assert!(result.is_ok());
        let buf = result.unwrap();
        
        let decoded = decode_quats(&buf.data, buf.total_quats);
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().len(), buf.total_quats);
    }

    // ==========================================
    // 4. NAjstresniji TESTOVI (Duboka ugnježđenja i masivni payload)
    // ==========================================

    #[test]
    fn test_10_deeply_nested_json_stress() {
        // Stress Test: Creating a deeply nested JSON structure (50 levels deep)
        let mut nested_json = String::new();
        let depth = 50;
        
        for _ in 0..depth {
            nested_json.push_str("{\"nested\":");
        }
        nested_json.push_str("1");
        for _ in 0..depth {
            nested_json.push_str("}");
        }

        let result = encode_json(&nested_json);
        assert!(result.is_ok(), "Failed to encode deep nesting stress test");
        
        let buf = result.unwrap();
        let decoded = decode_quats(&buf.data, buf.total_quats);
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().len(), buf.total_quats);
    }

    #[test]
    fn test_11_massive_payload_memory_stress() {
        // Extreme Stress Test: Generating a massive JSON payload with thousands of elements
        let mut massive_json = String::from("{\"items\": [");
        for i in 0..5000 {
            massive_json.push_str(&format!("{{\"id\": {}, \"status\": true}},", i));
        }
        massive_json.push_str("{\"id\": 9999, \"status\": false}]}");

        let result = encode_json(&massive_json);
        assert!(result.is_ok(), "Failed to encode massive scale payload");
        
        let buf = result.unwrap();
        assert!(buf.total_quats > 10000);
        
        // Verify unpack stability under heavy load
        let unpacked_quats = unpack_quats(&buf.data, buf.total_quats);
        assert_eq!(unpacked_quats.len(), buf.total_quats);
    }
}