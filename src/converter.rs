pub fn convert_binary_to_string(input_binary: &[u8]) -> String {
    input_binary
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect()
}

pub fn convert_string_to_binary(input_string: &str) -> Vec<u8> {
    input_string
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| {
            chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8))
        })
        .collect()
}