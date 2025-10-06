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
  d3bug(">>> get_entropy_from_anu", "debug");
  d3bug(&format!("entropy_length: {entropy_length:?}"), "debug");
  d3bug(&format!("data_format: {data_format:?}"), "debug");
  d3bug(&format!("array_length: {array_length:?}"), "debug");
  d3bug(&format!("hex_block_size: {hex_block_size:?}"), "debug");
  d3bug(&format!("log: {log:?}"), "debug");

  let start_time = SystemTime::now();
  let (sender, receiver) = std::sync::mpsc::channel();

  match fetch_anu_qrng_data(data_format, array_length, hex_block_size, sender) {
    Ok(_) => {
      d3bug("<<< fetch_anu_qrng_data", "debug");
    }
    Err(err) => d3bug(&format!("fetch_anu_qrng_data: \n{err:?}"), "error"),
  };

  let anu_data = receiver
    .recv()
    .map_err(|err| AppError::Custom(format!("anu_data not set {err:?}")))?;

  if let Some(anu_data) = anu_data.as_ref() {
    if !anu_data.is_empty() {
      if log {
        match create_anu_timestamp(start_time) {
          Ok(_) => {
            d3bug("<<< create_anu_timestamp", "debug");
          }
          Err(err) => d3bug(&format!("create_anu_timestamp: \n{err:?}"), "error"),
        };

        match write_api_response_to_log(&Some(anu_data.to_string())) {
          Ok(_) => {
            d3bug("<<< write_api_response_to_log", "debug");
          }
          Err(err) => d3bug(&format!("write_api_response_to_log: \n{err:?}"), "error"),
        };
      }
    } else {
      return Err(AppError::Custom("Anu response was empty".to_string()));
    }
  } else {
    return Err(AppError::Custom("Anu response was empty".to_string()));
  }

  let entropy = match data_format {
    "uint8" => {
      let uint8 = match extract_uint8_data(&anu_data) {
        Ok(data) => {
          d3bug("<<< extract_uint8_data", "debug");
          data
        }
        Err(err) => {
          return Err(AppError::Custom(format!(
            "Problem with extracting uint8 data from ANU {err}"
          )));
        }
      };

      match process_uint8_data(&uint8) {
        Ok(data) => {
          d3bug("<<< process_uint8_data", "debug");
          data
        }
        Err(err) => {
          return Err(AppError::Custom(format!(
            "Problem with processing uint8 data from ANU {err}"
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
    std::cmp::Ordering::Less => Err(AppError::Custom(t!("error.anu.short").to_string())),
  }
}

fn fetch_anu_qrng_data(
  data_format: &str,
  array_length: u32,
  block_size: u32,
  sender: std::sync::mpsc::Sender<Option<String>>,
) -> FunctionOutput<()> {
  d3bug(">>> fetch_anu_qrng_data", "debug");
  d3bug(&format!("fetch_anu_qrng_data: {data_format:?}"), "debug");
  d3bug(&format!("array_length: {array_length:?}"), "debug");
  d3bug(&format!("block_size: {block_size:?}"), "debug");

  let data_format_owned = data_format.to_string();
  let current_time = SystemTime::now();
  let last_request_time = load_last_anu_request()
    .map_err(|err| AppError::Custom(format!("Can not load system time: {err:?}")))?;

  let elapsed = current_time
    .duration_since(last_request_time)
    .unwrap_or(Duration::from_secs(0));

  let wait_duration = Duration::from_secs(ANU_REQUEST_INTERVAL_SECONDS as u64);

  if elapsed < wait_duration {
    let remaining_seconds = wait_duration.as_secs() - elapsed.as_secs();

    sender
      .send(Some(String::new()))
      .map_err(|err| AppError::Custom(format!("Can not send data: {err:?}")))?;

    return Err(AppError::Custom(
      t!("error.anu.timeout", value = remaining_seconds).to_string(),
    ));
  }

  let mut socket_addr = ANU_API_URL.to_socket_addrs().map_err(AppError::Io)?;

  let socket_addr = socket_addr.next().ok_or(AppError::Custom(
    "No socket addresses found for ANU API URL".to_string(),
  ))?;

  let mut stream = TcpStream::connect_timeout(
    &socket_addr,
    Duration::from_secs(TCP_REQUEST_TIMEOUT_SECONDS),
  )
  .map_err(AppError::Io)?;

  let anu_request = format!(
    "GET /API/jsonI.php?type={data_format_owned}&length={array_length}&size={block_size} HTTP/1.1\r\nHost: qrng.anu.edu.au\r\nConnection: close\r\n\r\n"
  )
    .into_bytes();

  stream.write_all(&anu_request).map_err(AppError::Io)?;

  stream.flush().map_err(AppError::Io)?;

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
    .map_err(|err| AppError::Custom(format!("Can not send data: {err:?}")))?;

  Ok(())
}

fn load_last_anu_request() -> FunctionOutput<SystemTime> {
  d3bug(">>> load_last_anu_request", "debug");

  let local_settings = LOCAL_SETTINGS
    .lock()
    .map_err(|err| AppError::Custom(format!("Failed to lock LOCAL_SETTINGS: {err}")))?;

  let local_temp_dir = local_settings
    .local_temp_dir
    .clone()
    .ok_or_else(|| AppError::Custom("local_temp_dir not set".to_string()))?;

  let path = Path::new(&local_temp_dir);

  if path.exists()
    && let Ok(file) = File::open(path)
  {
    let reader = BufReader::new(file);
    if let Some(Ok(timestamp_str)) = reader.lines().next()
      && let Ok(timestamp) = timestamp_str.trim().parse::<i64>()
    {
      return Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
    }
  }

  Ok(SystemTime::UNIX_EPOCH)
}

fn create_anu_timestamp(time: SystemTime) -> FunctionOutput<()> {
  d3bug(">>> create_anu_timestamp", "debug");

  let local_settings = LOCAL_SETTINGS
    .lock()
    .map_err(|err| AppError::Custom(format!("Failed to lock LOCAL_SETTINGS: {err}")))?;

  let local_temp_dir = local_settings
    .local_temp_dir
    .clone()
    .ok_or_else(|| AppError::Custom("local_temp_dir not set".to_string()))?;

  let local_anu_timestamp_file = Path::new(&local_temp_dir).join(ANU_TIMESTAMP_FILE);

  d3bug(
    &format!("local_anu_timestamp_file: {local_anu_timestamp_file:?}"),
    "log",
  );

  let timestamp = time
    .duration_since(SystemTime::UNIX_EPOCH)
    .map_err(|err| AppError::Custom(format!("Failed to get system time: {err}")))?
    .as_secs()
    .to_string();

  if let Some(parent) = Path::new(&local_anu_timestamp_file).parent() {
    fs::create_dir_all(parent).map_err(|err| {
      AppError::Custom(format!(
        "Failed to create directory {local_anu_timestamp_file:?}: {err}"
      ))
    })?;
  }

  let mut file = File::create(local_anu_timestamp_file).map_err(AppError::Io)?;

  file.write_all(timestamp.as_bytes()).map_err(AppError::Io)?;

  Ok(())
}

fn write_api_response_to_log(response: &Option<String>) -> FunctionOutput<()> {
  d3bug(">>> write_api_response_to_log", "debug");

  let local_settings = LOCAL_SETTINGS
    .lock()
    .map_err(|err| AppError::Custom(format!("Failed to lock LOCAL_SETTINGS: {err}")))?;

  let local_temp_dir = local_settings
    .local_temp_dir
    .clone()
    .ok_or_else(|| AppError::Custom("local_temp_dir not set".to_string()))?;

  let local_anu_response_file = Path::new(&local_temp_dir).join(ANU_RESPONSE_FILE);
  d3bug(
    &format!("local_anu_response_file: {local_anu_response_file:?}"),
    "log",
  );

  if let Some(parent) = Path::new(&local_anu_response_file).parent() {
    match fs::create_dir_all(parent) {
      Ok(_) => {
        let mut file = match File::create(&local_anu_response_file) {
          Ok(file) => file,
          Err(e) => {
            return Err(AppError::Io(e));
          }
        };

        if let Some(data) = &response {
          let bytes = data.as_bytes();

          if let Err(err) = file.write_all(bytes) {
            return Err(AppError::Custom(format!(
              "Can not write ANU response to log file: {err}"
            )));
          }
        } else {
          return Err(AppError::Custom("ANU response is empty".to_string()));
        }
      }
      Err(err) => {
        return Err(AppError::Io(err));
      }
    }
  }

  Ok(())
}

fn extract_uint8_data(api_response: &Option<String>) -> FunctionOutput<Vec<u8>> {
  d3bug(">>> extract_uint8_data", "debug");

  let api_response = match api_response {
    Some(response) => response,
    None => {
      return Err(AppError::Custom("ANU response is None".to_string()));
    }
  };

  let json_start_index = match api_response.find('{') {
    Some(index) => index,
    None => {
      return Err(AppError::Custom(
        "JSON data not found in the response".to_string(),
      ));
    }
  };

  let json_end_index = match api_response.rfind('}') {
    Some(index) => index,
    None => {
      return Err(AppError::Custom(
        "JSON data end not found in the response".to_string(),
      ));
    }
  };

  let json_str = &api_response[json_start_index..=json_end_index];
  let parsed_json: Result<serde_json::Value, _> = serde_json::from_str(json_str);
  let parsed_json = match parsed_json {
    Ok(value) => value,
    Err(err) => {
      return Err(AppError::Custom(format!("Failed to parse JSON: {err}")));
    }
  };

  let data_array = parsed_json["data"].as_array();
  let data_array = match data_array {
    Some(arr) => arr,
    None => {
      return Err(AppError::Custom("No data array found".to_string()));
    }
  };

  let mut uint8_data = Vec::new();

  for data_item in data_array {
    if let Some(byte_val) = data_item.as_u64() {
      if byte_val <= u8::MAX as u64 {
        uint8_data.push(byte_val as u8);
      } else {
        return Err(AppError::Custom(
          "Error parsing byte: number too large to fit in target type".to_string(),
        ));
      }
    } else {
      return Err(AppError::Custom(format!(
        "Invalid byte value: {data_item:?}"
      )));
    }
  }

  Ok(uint8_data)
}

fn process_uint8_data(data: &[u8]) -> FunctionOutput<String> {
  d3bug(">>> process_uint8_data", "debug");

  let binary_string = data
    .iter()
    .flat_map(|byte| format!("{byte:08b}").chars().collect::<Vec<_>>())
    .collect::<String>();

  Ok(binary_string)
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
