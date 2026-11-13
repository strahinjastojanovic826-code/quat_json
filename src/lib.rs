pub mod quat;
pub mod ast;
pub mod codec;
pub mod ffi;
pub mod query;
pub mod stream;
pub mod logic;
pub mod compression;
pub mod safety;

#[cfg(test)]
mod tests {
    use super::quat::{Quat, pack_quats, unpack_quats};
    use super::codec::{encode_json, decode_quats, AdvancedCodecError};
    use std::time::Instant;
    use crate::query::QuatQueryEngine;
    use crate::stream::QuatStreamEncoder;
    use crate::logic::{QuatLogicState, QuatLogicState::{False, True, Unknown, Error}};
    use crate::compression::ScalarCompressor;
    use crate::safety::{RobustnessValidator, verify_thread_safety_contracts};
    use crate::quat::QuatBuffer;
    use crate::ast::QuatTokenType;


    // ==========================================
    // 1. NAJLAKŠI TESTOVI (Osnovne jedinice i bitovi)
    // ==========================================

    #[test]
    fn test_01_quat_enum_mapping() {
        let start = Instant::now();
        // Test basic Quat enum parsing from u8 values
        assert_eq!(Quat::from_u8(0b00), Some(Quat::Q0));
        assert_eq!(Quat::from_u8(0b01), Some(Quat::Q1));
        assert_eq!(Quat::from_u8(0b10), Some(Quat::Q2));
        assert_eq!(Quat::from_u8(0b11), Some(Quat::Q3));
        assert_eq!(Quat::from_u8(4), Some(Quat::Q0)); // Prilagođeno zbog & 0b11 maskiranja
        let duration = start.elapsed();
        println!("Benchmark [test_01_quat_enum_mapping] execution time: {:?}", duration);
    }

    #[test]
    fn test_02_basic_quat_packing() {
        let start = Instant::now();
        // Test packing exactly 4 quats into a single u8 byte
        let quats: Vec<Quat> = vec![Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3];
        let buffer: QuatBuffer = pack_quats(&quats);

        assert_eq!(buffer.total_quats, 4);
        assert_eq!(buffer.data.len(), 1);
        assert_eq!(buffer.data[0], 0xE4); // (2 << 6 | 1 << 4 | 2 << 2 | 0 = 0xC9 (201))
        let duration = start.elapsed();
        println!("Benchmark [test_02_basic_quat_packing] execution time: {:?}", duration);
    }

    #[test]
    fn test_03_basic_quat_unpacking() {
        let start = Instant::now();
        // Test unpacking bytes back to original quats
        let original: Vec<Quat> = vec![Quat::Q3, Quat::Q0, Quat::Q2, Quat::Q1];
        let buffer: QuatBuffer = pack_quats(&original);
        let unpacked: Vec<Quat> = unpack_quats(&buffer.data, buffer.total_quats);

        assert_eq!(original, unpacked);
        let duration = start.elapsed();
        println!("Benchmark [test_03_basic_quat_unpacking] execution time: {:?}", duration);
    }

    // ==========================================
    // 2. SREDNJI TESTOVI (Jednostavan JSON i validacija)
    // ==========================================

    #[test]
    fn test_04_empty_json_object() {
        let start = Instant::now();
        // Test encoding an empty JSON object
        let json: &str = "{}";
        let result: Result<QuatBuffer, AdvancedCodecError> = encode_json(json);
        assert!(result.is_ok());
        let buf: QuatBuffer = result.unwrap();
        assert!(buf.total_quats > 0);
        let duration = start.elapsed();
        println!("Benchmark [test_04_empty_json_object] execution time: {:?}", duration);
    }

    #[test]
    fn test_05_simple_json_array() {
        let start = Instant::now();
        // Test encoding a flat JSON array
        let json: &str = "[1, 2, 3, 4]";
        let result: Result<QuatBuffer, AdvancedCodecError> = encode_json(json);
        assert!(result.is_ok());
        let buf: QuatBuffer = result.unwrap();

        let decoded_tokens: Result<Vec<QuatTokenType>, _> = decode_quats(&buf.data, buf.total_quats);
        assert!(decoded_tokens.is_ok());
        assert_eq!(decoded_tokens.unwrap().len(), buf.total_quats);
        let duration = start.elapsed();
        println!("Benchmark [test_05_simple_json_array] execution time: {:?}", duration);
    }

    #[test]
    fn test_06_key_value_json_object() {
        let start = Instant::now();
        // Test encoding a simple key-value object
        let json: &str = "{\"id\": 100}";
        let result: Result<QuatBuffer, AdvancedCodecError> = encode_json(json);
        assert!(result.is_ok());

        let buf: QuatBuffer = result.unwrap();
        let decoded: Vec<QuatTokenType> = decode_quats(&buf.data, buf.total_quats).unwrap();
        assert!(!decoded.is_empty());
        let duration = start.elapsed();
        println!("Benchmark [test_06_key_value_json_object] execution time: {:?}", duration);
    }

    // ==========================================
    // 3. NAPREDNI TESTOVI (Greške i ugnježđeni objekti)
    // ==========================================

    #[test]
    fn test_07_malformed_json_error() {
        let start = Instant::now();
        // Test handling of malformed JSON strings
        let bad_json: &str = "{\"id\": 100, missing_quote}";
        let result: Result<QuatBuffer, AdvancedCodecError> = encode_json(bad_json);

        assert!(result.is_err());
        match result.unwrap_err() {
            AdvancedCodecError::JsonParseError(_) => assert!(true),
            _ => panic!("Expected JsonParseError"),
        }
        let duration = start.elapsed();
        println!("Benchmark [test_07_malformed_json_error] execution time: {:?}", duration);
    }

    #[test]
    fn test_08_empty_input_error() {
        let start = Instant::now();
        // Test handling of empty string input
        let empty_input: &str = "";
        let result: Result<QuatBuffer, AdvancedCodecError> = encode_json(empty_input);

        assert!(result.is_err());
        match result.unwrap_err() {
            AdvancedCodecError::EmptyInput => assert!(true),
            _ => panic!("Expected EmptyInput error"),
        }
        let duration = start.elapsed();
        println!("Benchmark [test_08_empty_input_error] execution time: {:?}", duration);
    }

    #[test]
    fn test_09_nested_objects_and_arrays() {
        let start = Instant::now();
        // Test complex nested JSON structures
        let complex_json: &str = r#"{
            "user": {
                "name": "Rust",
                "active": true,
                "scores": [95, 88, 92]
            }
        }"#;

        let result: Result<QuatBuffer, AdvancedCodecError> = encode_json(complex_json);
        assert!(result.is_ok());
        let buf: QuatBuffer = result.unwrap();

        let decoded: Result<Vec<QuatTokenType>, _> = decode_quats(&buf.data, buf.total_quats);
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().len(), buf.total_quats);
        let duration = start.elapsed();
        println!("Benchmark [test_09_nested_objects_and_arrays] execution time: {:?}", duration);
    }

    // ==========================================
    // 4. NAJSTRESNIJI TESTOVI (Duboka ugnježđenja i masivni payload)
    // ==========================================

    #[test]
    fn test_10_deeply_nested_json_stress() {
        let start = Instant::now();
        // Stress Test: Creating a deeply nested JSON structure (50 levels deep)
        let mut nested_json: String = String::new();
        let depth: i32 = 50;

        for _ in 0..depth {
            nested_json.push_str("{\"nested\":");
        }
        nested_json.push_str("1");
        for _ in 0..depth {
            nested_json.push_str("}");
        }

        let result: Result<QuatBuffer, AdvancedCodecError> = encode_json(&nested_json);
        assert!(result.is_ok(), "Failed to encode deep nesting stress test");

        let buf: QuatBuffer = result.unwrap();
        let decoded: Result<Vec<QuatTokenType>, _> = decode_quats(&buf.data, buf.total_quats);
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().len(), buf.total_quats);
        let duration = start.elapsed();
        println!("Benchmark [test_10_deeply_nested_json_stress] execution time: {:?}", duration);
    }

    #[test]
    fn test_11_massive_payload_memory_stress() {
        let start = Instant::now();
        // Extreme Stress Test: Generating a massive JSON payload with thousands of elements
        let mut massive_json: String = String::from("{\"items\": [");
        for i in 0..5000 {
            massive_json.push_str(&format!("{{\"id\": {}, \"status\": {}}},", i, i % 2 == 0));
        }
        massive_json.push_str("{\"id\": 9999, \"status\": false}]}");

        let result: Result<QuatBuffer, AdvancedCodecError> = encode_json(&massive_json);
        assert!(result.is_ok(), "Failed to encode massive scale payload");

        let buf: QuatBuffer = result.unwrap();
        assert!(buf.total_quats > 10000);

        // Verify unpack stability under heavy load
        let unpacked_quats: Vec<Quat> = unpack_quats(&buf.data, buf.total_quats);
        assert_eq!(unpacked_quats.len(), buf.total_quats);
        let duration = start.elapsed();
        println!("Benchmark [test_11_massive_payload_memory_stress] execution time: {:?}", duration);
    }

    #[test]
    fn test_roundtrip_json() {
        let start = Instant::now();
        let json_data: &str = r#"{"key": [1, 2, 3]}"#;
        let buffer: QuatBuffer = encode_json(json_data).unwrap();

        let decoded: Result<Vec<QuatTokenType>, _> = decode_quats(&buffer.data, buffer.total_quats);
        assert!(decoded.is_ok());
        let duration = start.elapsed();
        println!("Benchmark [test_roundtrip_json] execution time: {:?}", duration);
    }

    #[test]
    fn test_corrupted_buffer_handling() {
        let start = Instant::now();
        // Send fewer bytes than required (10 quats require 3 bytes, we send only 1)
        let corrupted_bytes: Vec<u8> = vec![0xFF];
        let result: Result<Vec<QuatTokenType>, _> = decode_quats(&corrupted_bytes, 10);
        assert!(result.is_err());
        let duration = start.elapsed();
        println!("Benchmark [test_corrupted_buffer_handling] execution time: {:?}", duration);
    }

    #[test]
fn test_query_engine_with_benchmark() {
    let json_str = r#"{"a": 1, "b": 2}"#;
    let buf = encode_json(json_str).unwrap();
    let decoded_tokens = crate::codec::decode_quats(&buf.data, buf.total_quats).unwrap();

    // Benchmark execution time of query engine operations
    let start = Instant::now();
    let engine = QuatQueryEngine::new(&decoded_tokens);
    let value_node_count = engine.count_token_type(crate::ast::QuatTokenType::ValueNode);
    let keys = engine.find_keys();
    let duration = start.elapsed();

    assert!(value_node_count > 0);
    assert!(!keys.is_empty());
    println!("[BENCHMARK] Query Engine execution time: {:?}", duration);
}

#[test]
fn test_stream_encoder_with_benchmark() {
    let start = Instant::now();
    let mut encoder = QuatStreamEncoder::new();
    encoder.push_quat(Quat::Q0);
    encoder.push_quat(Quat::Q1);
    encoder.push_quat(Quat::Q2);
    encoder.push_quat(Quat::Q3);
    let (data, total) = encoder.finalize();
    let duration = start.elapsed();

    assert_eq!(total, 4);
    assert_eq!(data.len(), 1);
    println!("[BENCHMARK] Stream Encoder execution time: {:?}", duration);
}

#[test]
fn test_fast_unpack_with_benchmark() {
    let quats = vec![Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3];
    let buf = pack_quats(&quats);

    let start = Instant::now();
    let unpacked = buf.fast_unpack();
    let duration = start.elapsed();

    assert_eq!(unpacked, quats);
    println!("[BENCHMARK] Fast Unpack execution time: {:?}", duration);
}

#[test]
fn test_quaternary_logic_with_benchmark() {
    let start = Instant::now();
    let t1 = QuatLogicState::True;
    let t2 = QuatLogicState::False;
    let res_and = QuatLogicState::quat_and(t1, t2);
    let res_or = QuatLogicState::quat_or(t1, t2);
    let res_not = t1.quat_not();
    let duration = start.elapsed();

    assert_eq!(res_and, False);
    assert_eq!(res_or, True);
    assert_eq!(res_not, False);
    println!("[BENCHMARK] Quaternary Logic execution time: {:?}", duration);
}

#[test]
fn test_scalar_compression_with_benchmark() {
    let scalars = vec![0, 1, 2, 3, 1, 0, 2, 3];

    let start = Instant::now();
    let compressed = ScalarCompressor::compress_scalars(&scalars).unwrap();
    let decompressed = ScalarCompressor::decompress_scalars(&compressed);
    let duration = start.elapsed();

    assert_eq!(scalars, decompressed);
    println!("[BENCHMARK] Scalar Compression & Decompression execution time: {:?}", duration);
}

#[test]
fn test_safety_and_robustness_with_benchmark() {
    verify_thread_safety_contracts();
    let quats = vec![Quat::Q0, Quat::Q1];
    let buf = pack_quats(&quats);

    let start = Instant::now();
    let validation_result = RobustnessValidator::validate_buffer_integrity(&buf);
    let duration = start.elapsed();

    assert!(validation_result.is_ok());
    println!("[BENCHMARK] Safety & Robustness Validation execution time: {:?}", duration);
}
    
}