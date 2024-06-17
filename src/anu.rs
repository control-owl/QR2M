// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "ANU QRNG"
// copyright = "Copyright © 2023-2024 D3BUG"
// version = "2024-06-16"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use std::{
    fs::{self, File}, 
    io::{BufRead, BufReader, Read, Write}, 
    net::{TcpStream,ToSocketAddrs}, 
    path::Path, 
    time::{Duration, SystemTime}
};
use rand::Rng;

const ANU_TIMESTAMP_FILE: &str = "tmp/anu.timestamp";
const ANU_API_URL: &str = "qrng.anu.edu.au:80";
const TCP_REQUEST_TIMEOUT_SECONDS: u64 = 60;
const ANU_REQUEST_INTERVAL_SECONDS: i64 = 120;


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub fn get_entropy_from_anu(entropy_length: usize, data_format: &str, array_length: u32,hex_block_size: Option<u32>) -> String {
    let start_time = SystemTime::now();

    crate::create_message_window(
        "ANU QRNG API", 
        &t!("UI.main.anu.download"), 
        Some(true), 
        Some(5)
    );
    
    let (sender, receiver) = std::sync::mpsc::channel();

    fetch_anu_qrng_data(
        data_format, 
        array_length, 
        hex_block_size.unwrap(),
        sender
    );

    let anu_data = receiver.recv().unwrap(); // Blocking call to wait for the response

    if !anu_data.as_ref().unwrap().is_empty() {
        create_anu_timestamp(start_time);
        // TODO: Check if global log is enabled, then save
        write_api_response_to_log(&anu_data);
    } else {
        return String::new();
    }

    // anu_data.unwrap();

    let entropy = match data_format {
        "uint8" =>  {
            let uint8 = extract_uint8_data(&anu_data);

            process_uint8_data(&uint8)
        },
        "uint16" =>  {
            todo!() // Create uint16 ANU extraction
        },
        "hex16" =>  {
            todo!() // Create hex16 ANU extraction
            // let hex_strings = extract_hex_strings(
            //         &anu_data, 
            //         hex_block_size.unwrap().try_into().unwrap()
            //     );
            //     let mut anu_qrng_binary = String::new();
            //     for hex_string in hex_strings {
            //         // println!("Hex string: {}", hex_string);
            //         let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
            //         let binary_string: String = bytes.iter()
            //             .map(|byte| format!("{:08b}", byte))
            //             .collect();
            //         // println!("Binary string: {:?}", binary_string);
            //         anu_qrng_binary.push_str(&binary_string);
            //     }
            //     // Write all binary strings to a file
            //     let qrng_file = format!("{}.binary", ANU_QRNG_FILE);
            //     let mut file = File::create(&qrng_file).expect("Can not read file");
            //     file.write_all(anu_qrng_binary.as_bytes()).expect("can not write to file");
            //     if anu_qrng_binary.len() < entropy_length {
            //         return Err(format!(
            //             "Entropy string too short for requested entropy length: {}",
            //             entropy_length
            //         ).into());
            //     }
            //     let max_start = anu_qrng_binary.len() - entropy_length;
            //     let start_point = rand::thread_rng().gen_range(0..=max_start);
            //     entropy_raw_binary = anu_qrng_binary
            //         .chars()
            //         .skip(start_point)
            //         .take(entropy_length)
            //         .collect();
            //     println!("Final entropy string: {}", entropy_raw_binary);
        },
        _ => {
            eprintln!("{}", &t!("error.anu.format"));
            return String::new()
        }
    };

    if entropy.len() > entropy_length {
        let original_len = entropy.len();
        let mut rng = rand::thread_rng();
        let start_index = rng.gen_range(0..original_len);

        let trimmed_entropy = if start_index + entropy_length < original_len {
            entropy[start_index..start_index + entropy_length].to_string()
        } else {
            entropy[start_index..].to_string()
        };

        return trimmed_entropy;
    } else if entropy.len() == entropy_length {
        return entropy.to_string();
    } else {
        eprintln!("{}", &t!("error.anu.short"));
        return String::new();
    }
}

fn fetch_anu_qrng_data(
    data_format: &str,
    array_length: u32,
    block_size: u32,
    sender: std::sync::mpsc::Sender<Option<String>>
) {
    let data_format_owned = data_format.to_string();

    std::thread::spawn(move || {
        let current_time = SystemTime::now();
        let last_request_time = load_last_anu_request().unwrap();
        let elapsed = current_time.duration_since(last_request_time).unwrap_or(Duration::from_secs(0));
        let wait_duration = Duration::from_secs(ANU_REQUEST_INTERVAL_SECONDS as u64);

        if elapsed < wait_duration {
            let remaining_seconds = wait_duration.as_secs() - elapsed.as_secs();
            crate::create_message_window(
                "ANU API Timeout", 
                &t!("error.anu.timeout", value = remaining_seconds), 
                Some(true), 
                Some(remaining_seconds as u32)
            );
            eprintln!("{}", &t!("error.anu.timeout", value = remaining_seconds));
            sender.send(Some(String::new())).unwrap();
            return;
        }

        let mut socket_addr = ANU_API_URL
            .to_socket_addrs()
            .map_err(|e| format!("Socket address parsing error: {}", e))
            .unwrap();
        
        let socket_addr = socket_addr
            .next()
            .ok_or("No socket addresses found for ANU API URL")
            .unwrap();

        let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(TCP_REQUEST_TIMEOUT_SECONDS))
            .map_err(|e| format!("Connection error: {}", e))
            .unwrap();

        let anu_request = format!(
            "GET /API/jsonI.php?type={}&length={}&size={} HTTP/1.1\r\nHost: qrng.anu.edu.au\r\nConnection: close\r\n\r\n",
            data_format_owned, array_length, block_size
        )
        .into_bytes();

        stream.write_all(&anu_request)
            .map_err(|e| format!("Write error: {}", e))
            .unwrap();

        stream.flush()
            .map_err(|e| format!("Flush error: {}", e))
            .unwrap();

        let mut response = String::new();
        let mut buffer = [0; 256];
        let mut chunks = Vec::new();

        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
                    // print!("{}", chunk);
                    response.push_str(&chunk);
                    chunks.push(chunk.to_string());

                    if chunk.ends_with("\r\n\r\n") {
                        break;
                    }
                }
                Ok(_) | Err(_) => break,
            }
        }

        let combined_response = chunks.concat();
        sender.send(Some(combined_response)).unwrap();
    });
}

fn load_last_anu_request() -> Option<SystemTime> {
    let path = Path::new(ANU_TIMESTAMP_FILE);
    if path.exists() {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            if let Some(Ok(timestamp_str)) = reader.lines().next() {
                if let Ok(timestamp) = timestamp_str.trim().parse::<i64>() {
                    return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
                }
            }
        }
    }
    Some(SystemTime::UNIX_EPOCH)
}

fn create_anu_timestamp(time: SystemTime) {
    let timestamp = time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string();

    if let Some(parent) = Path::new(ANU_TIMESTAMP_FILE).parent() {
        fs::create_dir_all(parent).expect("Can not create log directory");
    }

    let mut file = File::create(ANU_TIMESTAMP_FILE).expect("Can not create ANU timestamp file");

    file.write_all(timestamp.as_bytes()).expect("Can not write to ANU timestamp file");
}

fn write_api_response_to_log(response: &Option<String>) {
    if let Some(parent) = Path::new(crate::get_log_file().as_str()).parent() {
        match fs::create_dir_all(parent) {
            Ok(_) => {
                let mut file = match File::create(&crate::get_log_file().as_str()) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("Error creating file: {}", e);
                        return;
                    }
                };

                if let Some(data) = &response {
                    let bytes = data.as_bytes();
                    if let Err(e) = file.write_all(bytes) {
                        eprintln!("Can not write ANU response to log file: {}", e);
                    }
                } else {
                    eprintln!("ANU response is empty");
                }
            }
            Err(err) => {
                eprintln!("Error creating directory {}: {}", parent.display(), err);
            }
        }
    }
}

fn extract_uint8_data(api_response: &Option<String>) -> Option<Vec<u8>> {
    let api_response = match api_response {
        Some(response) => response,
        None => {
            eprintln!("ANU response is None.");
            return None;
        }
    };

    let json_start_index = match api_response.find('{') {
        Some(index) => index,
        None => {
            eprintln!("JSON data not found in the response.");
            return None;
        }
    };

    let json_end_index = match api_response.rfind('}') {
        Some(index) => index,
        None => {
            eprintln!("JSON data end not found in the response.");
            return None;
        }
    };

    let json_str = &api_response[json_start_index..=json_end_index];
    let parsed_json: Result<serde_json::Value, _> = serde_json::from_str(json_str);
    let parsed_json = match parsed_json {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to parse JSON: {}", err);
            return None;
        }
    };

    let data_array = parsed_json["data"].as_array();
    let data_array = match data_array {
        Some(arr) => arr,
        None => {
            eprintln!("No data array found.");
            return None;
        }
    };

    let mut uint8_data = Vec::new();

    for data_item in data_array {
        if let Some(byte_val) = data_item.as_u64() {
            if byte_val <= u8::MAX as u64 {
                uint8_data.push(byte_val as u8);
            } else {
                eprintln!("Error parsing byte: number too large to fit in target type");
            }
        } else {
            eprintln!("Invalid byte value: {:?}", data_item);
        }
    }

    Some(uint8_data)
}

fn process_uint8_data(data: &Option<Vec<u8>>) -> String {
    let data = match data {
        Some(data) => data,
        None => {
            eprintln!("ANU response was empty.");
            return String::new();
        }
    };

    let binary_string = data
        .iter()
        .flat_map(|byte| {
            format!("{:08b}", byte)
                .chars()
                .collect::<Vec<_>>()
        })
        .collect::<String>();

    binary_string
}

// ANU extract hex16
// TODO: recheck if hex16 code is still working
// fn extract_hex_strings(response: &str, hex_block_size: usize) -> Vec<String> {
//     let hex_block_size = hex_block_size * 2; // Adjust for byte format for ANU
//     let mut hex_strings = Vec::new();
//     let mut current_string = String::new();
//     let mut in_hex_string = false;
//     for c in response.chars() {
//         if !in_hex_string {
//             if c == '"' {
//                 // Start of a potential hex string
//                 in_hex_string = true;
//                 current_string.clear();
//             }
//         } else {
//             if c == '"' {
//                 // End of hex string found, check if it's of expected length and contains valid hex characters
//                 if current_string.len() == hex_block_size && current_string.chars().all(|c| c.is_ascii_hexdigit()) {
//                     hex_strings.push(current_string.clone());
//                 }
//                 current_string.clear();
//                 in_hex_string = false;
//             } else if c == '\r' || c == '\n' || c == '\t' {
//                 // Ignore control characters within the hex string
//                 current_string.clear();
//                 in_hex_string = false;
//             } else {
//                 // Character is part of hex string, add to current string
//                 current_string.push(c);
//             }
//         }
//     }
//     hex_strings
// }


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
