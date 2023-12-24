pub fn convert_binary_to_string(input_value: &[u8]) -> String {
    input_value
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect()
}

pub fn convert_string_to_binary(input_value: &str) -> Vec<u8> {
    input_value
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| {
            chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8))
        })
        .collect()
}

pub fn convert_hex_to_binary(input_value: &str) -> Vec<u8> {
    hex::decode(input_value).expect("Failed to decode seed hex")
}