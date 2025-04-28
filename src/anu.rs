// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

use rand::Rng;
use std::{
  fs::{self, File},
  io::{BufRead, BufReader, Read, Write},
  net::{TcpStream, ToSocketAddrs},
  path::Path,
  time::{Duration, SystemTime},
};

use crate::os::LOCAL_SETTINGS;
use crate::{AppError, FunctionOutput, d3bug};

const ANU_TIMESTAMP_FILE: &str = "anu.timestamp";
const ANU_RESPONSE_FILE: &str = "anu.api";
const ANU_API_URL: &str = "qrng.anu.edu.au:80";
const TCP_REQUEST_TIMEOUT_SECONDS: u64 = 60;
const ANU_REQUEST_INTERVAL_SECONDS: i64 = 120;

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

pub fn get_entropy_from_anu(
  entropy_length: usize,
  data_format: &str,
  array_length: u32,
  hex_block_size: u32,
  log: bool,
) -> FunctionOutput<String> {
  d3bug(">>> get_entropy_from_anu", "log");
  d3bug(&format!("entropy_length: {:?}", entropy_length), "log");
  d3bug(&format!("data_format: {:?}", data_format), "log");
  d3bug(&format!("array_length: {:?}", array_length), "log");
  d3bug(&format!("hex_block_size: {:?}", hex_block_size), "log");
  d3bug(&format!("log: {:?}", log), "log");

  let start_time = SystemTime::now();
  let (sender, receiver) = std::sync::mpsc::channel();

  match fetch_anu_qrng_data(data_format, array_length, hex_block_size, sender) {
    Ok(_) => {
      d3bug("<<< fetch_anu_qrng_data", "log");
    }
    Err(err) => d3bug(&format!("fetch_anu_qrng_data: \n{:?}", err), "error"),
  };

  let anu_data = receiver
    .recv()
    .map_err(|e| AppError::Custom(format!("anu_data not set {:?}", e)))?;

  if let Some(anu_data) = anu_data.as_ref() {
    if !anu_data.is_empty() {
      if log {
        match create_anu_timestamp(start_time) {
          Ok(_) => {
            d3bug("<<< create_anu_timestamp", "log");
          }
          Err(err) => d3bug(&format!("create_anu_timestamp: \n{:?}", err), "error"),
        };

        match write_api_response_to_log(&Some(anu_data.to_string())) {
          Ok(_) => {
            d3bug("<<< write_api_response_to_log", "log");
          }
          Err(err) => d3bug(&format!("write_api_response_to_log: \n{:?}", err), "error"),
        };
      }
    }
  } else {
    return Err(AppError::Custom(format!("Anu response was empty")));
  }

  let entropy = match data_format {
    "uint8" => {
      let uint8 = match extract_uint8_data(&anu_data) {
        Ok(data) => {
          d3bug("<<< extract_uint8_data", "log");
          data
        }
        Err(err) => {
          return Err(crate::AppError::Custom(format!(
            "Problem with extracting uint8 data from ANU {}",
            err
          )));
        }
      };

      match process_uint8_data(&uint8) {
        Ok(data) => {
          d3bug("<<< process_uint8_data", "log");
          data
        }
        Err(err) => {
          return Err(crate::AppError::Custom(format!(
            "Problem with processing uint8 data from ANU {}",
            err
          )));
        }
      }
    }
    "uint16" => todo!(),
    "hex16" => todo!(),
    _ => {
      return Err(AppError::Custom(t!("error.anu.format").to_string()));
    }
  };

  match entropy.len().cmp(&entropy_length) {
    std::cmp::Ordering::Greater => {
      let mut rng = rand::rng();
      let original_len = entropy.len();
      let start_index = rng.random_range(0..original_len);

      if start_index + entropy_length < original_len {
        Ok(entropy[start_index..start_index + entropy_length].to_string())
      } else {
        Ok(entropy[start_index..].to_string())
      }
    }
    std::cmp::Ordering::Equal => Ok(entropy),
    std::cmp::Ordering::Less => {
      return Err(AppError::Custom(t!("error.anu.short").to_string()));
    }
  }
}

fn fetch_anu_qrng_data(
  data_format: &str,
  array_length: u32,
  block_size: u32,
  sender: std::sync::mpsc::Sender<Option<String>>,
) -> FunctionOutput<()> {
  d3bug(">>> fetch_anu_qrng_data", "log");
  d3bug(&format!("fetch_anu_qrng_data: {:?}", data_format), "log");
  d3bug(&format!("array_length: {:?}", array_length), "log");
  d3bug(&format!("block_size: {:?}", block_size), "log");

  let data_format_owned = data_format.to_string();
  let current_time = SystemTime::now();
  let last_request_time = load_last_anu_request()
    .map_err(|e| AppError::Custom(format!("Can not load system time: {:?}", e)))?;

  let elapsed = current_time
    .duration_since(last_request_time)
    .unwrap_or(Duration::from_secs(0));

  let wait_duration = Duration::from_secs(ANU_REQUEST_INTERVAL_SECONDS as u64);

  if elapsed < wait_duration {
    let remaining_seconds = wait_duration.as_secs() - elapsed.as_secs();

    sender
      .send(Some(String::new()))
      .map_err(|e| AppError::Custom(format!("Can not send data: {:?}", e)))?;

    return Err(AppError::Custom(
      t!("error.anu.timeout", value = remaining_seconds).to_string(),
    ));
  }

  let mut socket_addr = ANU_API_URL
    .to_socket_addrs()
    .map_err(|e| crate::AppError::Io(e))?;

  let socket_addr = socket_addr.next().ok_or(AppError::Custom(
    "No socket addresses found for ANU API URL".to_string(),
  ))?;

  let mut stream = TcpStream::connect_timeout(
    &socket_addr,
    Duration::from_secs(TCP_REQUEST_TIMEOUT_SECONDS),
  )
  .map_err(|e| crate::AppError::Io(e))?;

  let anu_request = format!(
    "GET /API/jsonI.php?type={}&length={}&size={} HTTP/1.1\r\nHost: qrng.anu.edu.au\r\nConnection: close\r\n\r\n",
    data_format_owned, array_length, block_size
  )
    .into_bytes();

  stream
    .write_all(&anu_request)
    .map_err(|e| crate::AppError::Io(e))?;

  stream.flush().map_err(|e| crate::AppError::Io(e))?;

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
  sender
    .send(Some(combined_response))
    .map_err(|e| AppError::Custom(format!("Can not send data: {:?}", e)))?;

  Ok(())
}

fn load_last_anu_request() -> FunctionOutput<SystemTime> {
  d3bug(">>> load_last_anu_request", "log");

  let local_settings = LOCAL_SETTINGS
    .lock()
    .map_err(|e| crate::AppError::Custom(format!("Failed to lock LOCAL_SETTINGS: {}", e)))?;

  let local_temp_dir = local_settings
    .local_temp_dir
    .clone()
    .ok_or_else(|| crate::AppError::Custom("local_temp_dir not set".to_string()))?;

  let path = Path::new(&local_temp_dir);
  if path.exists() {
    if let Ok(file) = File::open(path) {
      let reader = BufReader::new(file);
      if let Some(Ok(timestamp_str)) = reader.lines().next() {
        if let Ok(timestamp) = timestamp_str.trim().parse::<i64>() {
          return Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
        }
      }
    }
  } else {
    return Err(crate::AppError::Custom(
      "local_temp_dir directory does not exists".to_string(),
    ));
  }

  Ok(SystemTime::UNIX_EPOCH)
}

fn create_anu_timestamp(time: SystemTime) -> FunctionOutput<()> {
  d3bug(">>> create_anu_timestamp", "log");

  let local_settings = LOCAL_SETTINGS
    .lock()
    .map_err(|e| crate::AppError::Custom(format!("Failed to lock LOCAL_SETTINGS: {}", e)))?;

  let local_temp_dir = local_settings
    .local_temp_dir
    .clone()
    .ok_or_else(|| crate::AppError::Custom("local_temp_dir not set".to_string()))?;

  let local_anu_timestamp_file = Path::new(&local_temp_dir).join(ANU_TIMESTAMP_FILE);

  d3bug(
    &format!("local_anu_timestamp_file: {:?}", local_anu_timestamp_file),
    "log",
  );

  let timestamp = time
    .duration_since(SystemTime::UNIX_EPOCH)
    .map_err(|e| crate::AppError::Custom(format!("Failed to get system time: {}", e)))?
    .as_secs()
    .to_string();

  if let Some(parent) = Path::new(&local_anu_timestamp_file).parent() {
    fs::create_dir_all(parent).map_err(|e| {
      crate::AppError::Custom(format!(
        "Failed to create directory {:?}: {}",
        local_anu_timestamp_file, e
      ))
    })?;
  }

  let mut file = File::create(local_anu_timestamp_file).map_err(|e| crate::AppError::Io(e))?;

  file
    .write_all(timestamp.as_bytes())
    .map_err(|e| crate::AppError::Io(e))?;

  Ok(())
}

fn write_api_response_to_log(response: &Option<String>) -> FunctionOutput<()> {
  d3bug(">>> write_api_response_to_log", "log");

  let local_settings = LOCAL_SETTINGS
    .lock()
    .map_err(|e| crate::AppError::Custom(format!("Failed to lock LOCAL_SETTINGS: {}", e)))?;

  let local_temp_dir = local_settings
    .local_temp_dir
    .clone()
    .ok_or_else(|| crate::AppError::Custom("local_temp_dir not set".to_string()))?;

  let local_anu_response_file = Path::new(&local_temp_dir).join(ANU_RESPONSE_FILE);
  d3bug(
    &format!("local_anu_response_file: {:?}", local_anu_response_file),
    "log",
  );

  if let Some(parent) = Path::new(&local_anu_response_file).parent() {
    match fs::create_dir_all(parent) {
      Ok(_) => {
        let mut file = match File::create(&local_anu_response_file) {
          Ok(file) => file,
          Err(e) => {
            return Err(crate::AppError::Io(e));
          }
        };

        if let Some(data) = &response {
          let bytes = data.as_bytes();

          if let Err(e) = file.write_all(bytes) {
            return Err(crate::AppError::Custom(format!(
              "Can not write ANU response to log file: {}",
              e
            )));
          }
        } else {
          return Err(crate::AppError::Custom("ANU response is empty".to_string()));
        }
      }
      Err(err) => {
        return Err(crate::AppError::Io(err));
      }
    }
  }

  Ok(())
}

fn extract_uint8_data(api_response: &Option<String>) -> FunctionOutput<Vec<u8>> {
  d3bug(">>> extract_uint8_data", "log");

  let api_response = match api_response {
    Some(response) => response,
    None => {
      return Err(crate::AppError::Custom("ANU response is None".to_string()));
    }
  };

  let json_start_index = match api_response.find('{') {
    Some(index) => index,
    None => {
      return Err(crate::AppError::Custom(
        "JSON data not found in the response".to_string(),
      ));
    }
  };

  let json_end_index = match api_response.rfind('}') {
    Some(index) => index,
    None => {
      return Err(crate::AppError::Custom(
        "JSON data end not found in the response".to_string(),
      ));
    }
  };

  let json_str = &api_response[json_start_index..=json_end_index];
  let parsed_json: Result<serde_json::Value, _> = serde_json::from_str(json_str);
  let parsed_json = match parsed_json {
    Ok(value) => value,
    Err(err) => {
      return Err(crate::AppError::Custom(format!(
        "Failed to parse JSON: {}",
        err
      )));
    }
  };

  let data_array = parsed_json["data"].as_array();
  let data_array = match data_array {
    Some(arr) => arr,
    None => {
      return Err(crate::AppError::Custom("No data array found".to_string()));
    }
  };

  let mut uint8_data = Vec::new();

  for data_item in data_array {
    if let Some(byte_val) = data_item.as_u64() {
      if byte_val <= u8::MAX as u64 {
        uint8_data.push(byte_val as u8);
      } else {
        return Err(crate::AppError::Custom(
          "Error parsing byte: number too large to fit in target type".to_string(),
        ));
      }
    } else {
      return Err(crate::AppError::Custom(format!(
        "Invalid byte value: {:?}",
        data_item
      )));
    }
  }

  Ok(uint8_data)
}

fn process_uint8_data(data: &Vec<u8>) -> FunctionOutput<String> {
  d3bug(">>> process_uint8_data", "log");

  let binary_string = data
    .iter()
    .flat_map(|byte| format!("{:08b}", byte).chars().collect::<Vec<_>>())
    .collect::<String>();

  Ok(binary_string)
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
