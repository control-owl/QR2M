/// Converts a slice of bytes representing binary data into a string of binary digits.
///
/// This function takes a slice of bytes `input_value` and converts each byte into a binary string
/// representation. Each byte is transformed into 8 binary digits (bits), and the resulting strings
/// are concatenated together to form the final output string.
///
/// # Arguments
///
/// * `input_value` - A slice of bytes (`&[u8]`) representing binary data to be converted.
///
/// # Returns
///
/// A `String` containing the binary representation of the input bytes.
///
/// # Examples
///
/// ```
/// let input = [208, 230, 220, 52];
/// let result = qr2m_converters::convert_binary_to_string(&input);
/// assert_eq!(result, "11010000111001101101110000110100");
/// ```
pub fn convert_binary_to_string(input_value: &[u8]) -> String {
    input_value
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect()
}

/// Converts a binary string representation into a vector of bytes.
///
/// This function takes a binary string (`input_value`) and converts it into a vector of bytes.
/// Each byte in the output vector corresponds to 8 bits of the input string.
///
/// # Arguments
///
/// * `input_value` - A reference to a string slice (`&str`) containing the binary representation.
///
/// # Returns
///
/// A vector (`Vec<u8>`) containing the binary representation converted into bytes.
///
/// # Examples
///
/// ```
/// let input = "01000111010110111001000110110101";
/// let result = qr2m_converters::convert_string_to_binary(input);
/// assert_eq!(result, [71, 91, 145, 181]);
/// ```
pub fn convert_string_to_binary(input_value: &str) -> Vec<u8> {
    input_value
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8)))
        .collect()
}
