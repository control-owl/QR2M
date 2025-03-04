// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "Development playground"
// copyright = "Copyright © 2023-2025 Control Owl"
// version = "2025-01-17"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use sha2::{Digest, Sha256};
use ed25519_dalek::{SigningKey, VerifyingKey};
use gtk4 as gtk;
use libadwaita as adw;
use adw::prelude::*;

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// SOLANA

pub fn derive_from_path_ed25519(
    master_key: &[u8],
    master_chain_code: &[u8],
    path: &str,
) -> crate::keys::DerivationResult {
    println!("Deriving from path for ed25519: {}", path);

    println!("master_key: {:?}", &master_key);
    println!("master_chain_code: {:?}", &master_chain_code);
    println!("path: {:?}", &path);

    let private_key = master_key.to_vec();
    let chain_code = master_chain_code.to_vec();
    let mut public_key = Vec::new();
    let mut private_key_array = [0; 32];

    for part in path.split('/') {
        if part == "m" {
            continue;
        }

        let hardened = part.ends_with("'");
        let index: u32 = match part.trim_end_matches("'").parse() {
            Ok(index) => {
                index
            },
            Err(_) => {
                eprintln!("Error: Unable to parse index from path part: {}", part);
                return None;
            }
        };
        let derived = derive_child_key_ed25519(
            &private_key, 
            &chain_code, 
            index, 
            hardened
        ).unwrap_or_default();

        private_key.clone().copy_from_slice(&derived.0);
        private_key_array = derived.0.clone().try_into().expect("Incorrect private key length");

        chain_code.clone().copy_from_slice(&derived.1);
        public_key = derived.2.clone();
    }

    let chain_code_array: [u8; 32] = chain_code.try_into().expect("Slice with incorrect length");
    Some((private_key_array, chain_code_array, public_key))
}

fn derive_child_key_ed25519(
    parent_key: &[u8],
    parent_chain_code: &[u8],
    index: u32,
    hardened: bool,
) -> crate::keys::DerivationResult {
    println!("Deriving ed25519 child key");
    
    println!("parent_key: {:?}", &parent_key);
    println!("parent_chain_code: {:?}", &parent_chain_code);
    println!("index: {:?}", &index);
    println!("hardened: {:?}", &hardened);
    
    let mut hasher = Sha256::new();
    hasher.update(parent_key);
    hasher.update(&index.to_be_bytes());
    if hardened {
        hasher.update(&[1u8; 1]);
    }
    let result = hasher.finalize();
    
    if result.len() != 64 {
        eprintln!("len is not 64, it is: {}", result.len());
        return None;
    }
    
    let mut child_private_key_bytes: [u8; 32] = [0; 32];
    let mut child_chain_code_bytes: [u8; 32] = [0; 32];
    
    child_private_key_bytes.copy_from_slice(&result[..32]);
    child_chain_code_bytes.copy_from_slice(&result[32..]);

    let secret_key = SigningKey::from_bytes(&child_private_key_bytes).to_bytes();
    let public_key = VerifyingKey::from_bytes(&secret_key).unwrap_or_default().to_bytes().to_vec();
    
    println!("child_private_key_bytes: {:?}", &secret_key);
    println!("child_chain_code_bytes: {:?}", &child_chain_code_bytes);
    println!("child_public_key_bytes: {:?}", &public_key);

    Some((
        secret_key,
        child_chain_code_bytes,
        public_key,
    ))
}

pub fn generate_ed25519_address(public_key: &crate::keys::CryptoPublicKey) -> String {
    let public_key_bytes = match public_key {
        crate::keys::CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
        _ => {
            eprintln!("generate_ed25519_address called with non-ed25519 key");
            Vec::new()
        }
    };
    
    let hash = Sha256::digest(&public_key_bytes);
    bs58::encode(hash).with_alphabet(bs58::Alphabet::BITCOIN).into_string()
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// NEW ANU LOGIC

struct BlockEntry {
    container: gtk::Box,
    entry: gtk::Entry,
    progress_bar: gtk::ProgressBar,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AnuResponse {
    success: bool,
    data: Vec<String>,
    #[serde(rename = "type")]
    data_type: String,
    length: u32,
    size: u32,
}

const QRNG_MIN_ARRAY: u32 = 2;
const TCP_REQUEST_TIMEOUT_SECONDS: u64 = 10;
const ANU_API_URL: &str = "qrng.anu.edu.au:443";


fn create_boxes(n: Option<u32>) -> Vec<BlockEntry> {
    let mut blocks = Vec::new();

    let array_size = match n {
        Some(value) => {if value < QRNG_MIN_ARRAY {QRNG_MIN_ARRAY} else {value}},
        None => QRNG_MIN_ARRAY
    };

    for i in 0..array_size {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let label = gtk::Label::new(Some(&format!("Block {}", i + 1)));
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
        let entry = gtk::Entry::new();
        let progress_bar = gtk::ProgressBar::new();

        entry.set_hexpand(true);
        entry.set_width_request(250);
        progress_bar.set_hexpand(true);
        progress_bar.set_pulse_step(0.1);

        info_box.append(&entry);
        info_box.append(&progress_bar);
        container.append(&label);
        container.append(&info_box);

        blocks.push(BlockEntry {
            container,
            entry,
            progress_bar,
        });
    }

    blocks
}

pub fn anu_window() -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .title(t!("UI.anu").to_string())
        .default_width(1024)
        .default_height(768)
        .resizable(true)
        .modal(true)
        .build();


    let lock_app_settings = crate::APP_SETTINGS.read().unwrap();
    let anu_data_type = lock_app_settings.anu_data_format.clone();
    let anu_array_length = lock_app_settings.anu_array_length.clone();
    let anu_hex_block_size = lock_app_settings.anu_hex_block_size.clone();

    let main_anu_window_box = gtk::Box::builder()
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .orientation(gtk::Orientation::Vertical)
        .build();

    let main_header_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let content_header_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let content_header_box_status = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let content_header_box_data_type = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let content_header_box_array_length = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let content_header_box_block_size = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let content_header_box_progress = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let scroll_window = gtk::ScrolledWindow::new();
    let anu_progress = gtk::ProgressBar::new();
    
    let anu_status_frame = gtk::Frame::new(Some("ANU status"));
    let anu_data_type_frame = gtk::Frame::new(Some("ANU data type"));
    let anu_array_length_frame = gtk::Frame::new(Some("ANU array length"));
    let anu_block_size_frame = gtk::Frame::new(Some("ANU block size"));
    // let anu_progress_frame = gtk::Frame::new(Some("ANU progress"));


    main_anu_window_box.append(&main_header_box);
    main_anu_window_box.append(&scroll_window);
    
    main_header_box.append(&content_header_box);
    main_header_box.append(&content_header_box_progress);
    main_header_box.set_hexpand(true);
    
    content_header_box.append(&content_header_box_status);
    content_header_box.append(&content_header_box_data_type);
    content_header_box.append(&content_header_box_array_length);
    content_header_box.append(&content_header_box_block_size);

    content_header_box_status.append(&anu_status_frame);
    content_header_box_status.append(&anu_data_type_frame);
    content_header_box_status.append(&anu_array_length_frame);
    content_header_box_status.append(&anu_block_size_frame);
    content_header_box_progress.append(&anu_progress);
    
    scroll_window.set_hexpand(true);
    scroll_window.set_vexpand(true);

    content_header_box.set_halign(gtk::Align::Center);
    content_header_box.set_hexpand(true);

    content_header_box_progress.set_margin_bottom(20);
    content_header_box_progress.set_margin_top(20);
    content_header_box_progress.set_hexpand(true);
    anu_progress.set_hexpand(true);

    let anu_status_entry = gtk::Entry::new();
    anu_status_entry.set_text("Inactive");
    anu_status_entry.set_editable(false);
    anu_status_frame.set_child(Some(&anu_status_entry));

    let anu_data_type_entry = gtk::Entry::new();
    anu_data_type_entry.set_text(&anu_data_type.clone().unwrap());
    anu_data_type_entry.set_editable(false);
    anu_data_type_frame.set_child(Some(&anu_data_type_entry));

    let anu_array_length_entry = gtk::Entry::new();
    anu_array_length_entry.set_text(&anu_array_length.clone().unwrap().to_string());
    anu_array_length_entry.set_editable(false);
    anu_array_length_frame.set_child(Some(&anu_array_length_entry));

    let anu_block_size_entry = gtk::Entry::new();
    anu_block_size_entry.set_text(&anu_hex_block_size.clone().unwrap().to_string());
    anu_block_size_entry.set_editable(false);
    anu_block_size_frame.set_child(Some(&anu_block_size_entry));
    
    let main_container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let blocks = create_boxes(anu_array_length);
    
    let blocks_rc = std::rc::Rc::new(std::cell::RefCell::new(blocks));
    
    for block in blocks_rc.borrow().iter() {
        main_container.append(&block.container);
    }

    scroll_window.set_child(Some(&main_container));

    let main_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let ok_button = gtk::Button::with_label("OK");
    let cancel_button = gtk::Button::with_label("Cancel");
    let new_button = gtk::Button::with_label("New QRNG");

    main_button_box.append(&ok_button);
    main_button_box.append(&new_button);
    main_button_box.append(&cancel_button);

    main_button_box.set_margin_bottom(10);
    main_button_box.set_margin_top(20);
    main_button_box.set_halign(gtk::Align::Center);
    main_anu_window_box.append(&main_button_box);
    window.set_child(Some(&main_anu_window_box));


    // Hocus - Pokus
    
    let fetch_handle = std::sync::Arc::new(std::sync::Mutex::new(None::<glib::JoinHandle<()>>));
    let parse_handler = std::sync::Arc::new(std::sync::Mutex::new(None::<glib::JoinHandle<()>>));

    let total_length = anu_array_length.clone().unwrap() as f64;
    let block_size = anu_hex_block_size.clone().unwrap();
    let total_hex_chars = total_length as f64 * block_size as f64 * 2.0;
    let received_chars = std::sync::Arc::new(std::sync::Mutex::new(0.0));
    let current_index = std::sync::Arc::new(std::sync::Mutex::new(0));
    let char_buffer = std::sync::Arc::new(std::sync::Mutex::new(String::new()));


    new_button.connect_clicked(glib::clone!(
        #[strong] fetch_handle,
        #[strong] parse_handler,
        #[strong] blocks_rc,
        #[strong] current_index,
        #[strong] received_chars,
        #[strong] anu_progress,
        #[strong] anu_status_entry,
        move |_| {
            let (tx, mut rx): (tokio::sync::mpsc::Sender<String>, tokio::sync::mpsc::Receiver<String>) = tokio::sync::mpsc::channel(100);
            anu_progress.set_show_text(true);

            let current_index_clone = current_index.clone();           
            let blocks = blocks_rc.borrow();

            for block in blocks.iter() {
                block.entry.set_text("Loading...");
                block.progress_bar.set_fraction(0.0);
            }

            *current_index_clone.lock().unwrap() = 0;
            *received_chars.lock().unwrap() = 0.0;
            anu_progress.set_fraction(0.0);
            anu_status_entry.set_text("Starting...");
    
            if let Some(handle) = fetch_handle.lock().unwrap().take() {
                handle.abort();
                println!("Previous fetch aborted.");
            }
    
            if let Some(handle) = parse_handler.lock().unwrap().take() {
                handle.abort();
                println!("Previous parsing aborted.");
            }
            let main_context = glib::MainContext::default();
    
            let anu_loop = main_context.spawn_local(async move {
                fetch_anu_qrng_data("hex16", total_length as u32, block_size, tx);
            });

            *fetch_handle.lock().unwrap() = Some(anu_loop);

            let blocks_rc_clone = blocks_rc.clone();
            let anu_status_entry_clone = anu_status_entry.clone();
            let char_buffer_clone = char_buffer.clone();
            let anu_progress_clone = anu_progress.clone();
            let received_chars_clone = received_chars.clone();

            let parsing_loop = main_context.spawn_local(async move {
                while let Some(chunk) = rx.recv().await {
                    let blocks = blocks_rc_clone.borrow();
                    let mut index = current_index_clone.lock().unwrap();
                    let mut chars = received_chars_clone.lock().unwrap();
                    let mut buffer = char_buffer_clone.lock().unwrap();
                    
                    if chunk.starts_with("FINAL:") {
                        anu_status_entry_clone.set_text("Reconstructing quantum entropy ...");

                        let json_data = &chunk[6..];
                        
                        match serde_json::from_str::<AnuResponse>(json_data) {
                            Ok(anu_response) => {
                                if anu_response.success {
                                    for (i, value) in anu_response.data.iter().enumerate() {
                                        let pos = i;
                                        if pos < blocks.len() {
                                            blocks[pos].entry.set_text(value.as_str());
                                            blocks[pos].progress_bar.set_fraction(1.0);
                                            *chars = (pos + 1) as f64 * value.len() as f64;
                                        }
                                    }
                                    *index = anu_response.data.len();
                                    anu_status_entry_clone.set_text("Complete");
                                    anu_progress_clone.set_fraction(1.0);
                                    *buffer = String::new();
                                    break;
                                } else {
                                    anu_status_entry_clone.set_text("ANU response unsuccessful");
                                    break;
                                }
                            }
                            Err(e) => {
                                anu_status_entry_clone.set_text(&format!("Parsing error: {}", e));
                                break;
                            }
                        }
                    } else {
                        buffer.push_str(&chunk);
                        let block_size_chars = anu_hex_block_size.clone().unwrap() as usize * 2;
                        while buffer.len() >= block_size_chars {
                            let segment = buffer.drain(..block_size_chars).collect::<String>();
                            let pos = *index;
                            if pos < blocks.len() {
                                blocks[pos].entry.set_text(&segment);
                                let chars_received = segment.len() as f64 / 2.0;
                                let target_chars = anu_hex_block_size.clone().unwrap() as f64;
                                let entry_progress = (chars_received / target_chars).min(1.0);
                                blocks[pos].progress_bar.set_fraction(entry_progress);
                                *chars += chars_received;
                                *index += 1;
                            }
                        }
                        let progress = *chars / total_hex_chars * 2.0;
                        anu_progress_clone.set_fraction(progress.min(1.0));
                        anu_status_entry_clone.set_text("Receiving raw json data ...");
                    }
                }
            });
    
            *parse_handler.lock().unwrap() = Some(parsing_loop);
        }
    ));

    cancel_button.connect_clicked(glib::clone!(
        #[strong] fetch_handle,
        #[strong] parse_handler,
        move |_| {
            if let Some(handle) = fetch_handle.lock().unwrap().take() {
                println!("canceling async task...");
                handle.abort();
            }

            if let Some(handle) = parse_handler.lock().unwrap().take() {
                println!("canceling parsing async task...");
                handle.abort();
            }
        }
    ));  

    window.connect_close_request(glib::clone!(
        #[strong] fetch_handle,
        #[strong] parse_handler,
        move |_| {
            if let Some(handle) = fetch_handle.lock().unwrap().take() {
                println!("aborting async task on window close...");
                handle.abort();
            }

            if let Some(handle) = parse_handler.lock().unwrap().take() {
                println!("canceling parsing async task...");
                handle.abort();
            }

            glib::Propagation::Proceed
        }
    ));
    
    window
}

use std::{
    io::{Read, Write}, 
    net::ToSocketAddrs, 
};


fn filter_chunked_body(chunk: &str) -> String {
    let mut filtered = String::new();
    let mut lines = chunk.lines();
    let mut skip_next = false;

    while let Some(line) = lines.next() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if let Ok(size) = usize::from_str_radix(line.trim(), 16) {
            if size == 0 {
                break;
            }
            skip_next = true;
            if let Some(data) = lines.next() {
                filtered.push_str(data);
            }
        } else {
            filtered.push_str(line);
        }
    }
    filtered
}

fn fetch_anu_qrng_data(
    data_format: &str,
    array_length: u32,
    block_size: u32,
    sender: tokio::sync::mpsc::Sender<String>,
) {
    let data_format_owned = data_format.to_string();

    println!("Starting fetch_anu_qrng_data: format={}, length={}, size={}", data_format, array_length, block_size);

    tokio::spawn(async move {
        match std::net::TcpStream::connect_timeout(
            &ANU_API_URL.to_socket_addrs().unwrap().next().unwrap(),
            std::time::Duration::from_secs(TCP_REQUEST_TIMEOUT_SECONDS),
        ) {
            Ok(stream) => {
                match native_tls::TlsConnector::new()
                    .unwrap()
                    .connect("qrng.anu.edu.au", stream)
                {
                    Ok(mut stream) => {
                        let anu_request = format!(
                            "GET /API/jsonI.php?type={}&length={}&size={} HTTP/1.1\r\nHost: qrng.anu.edu.au\r\nConnection: close\r\n\r\n",
                            data_format_owned, array_length, block_size
                        ).into_bytes();

                        println!("Sending request: {:?}", String::from_utf8_lossy(&anu_request));
                        
                        if stream.write_all(&anu_request).is_ok() && stream.flush().is_ok() {
                            let mut buffer = [0; 2048];
                            let mut response = Vec::new();
                            let mut headers_done = false;
                            let mut json_buffer = String::new();

                            loop {
                                match stream.read(&mut buffer) {
                                    Ok(bytes_read) if bytes_read > 0 => {
                                        let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
                                        response.extend_from_slice(&buffer[..bytes_read]);
                                        println!("Received chunk: {}", chunk);

                                        if !headers_done {
                                            if chunk.contains("\r\n\r\n") {
                                                headers_done = true;
                                                let header_end = response.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
                                                let body_start = String::from_utf8_lossy(&response[header_end..]).to_string();
                                                json_buffer = filter_chunked_body(&body_start);
                                                if sender.send(body_start).await.is_err() {
                                                    eprintln!("Failed to send body_start");
                                                    break;
                                                }
                                            }
                                        } else {
                                            let filtered_chunk = filter_chunked_body(&chunk);
                                            json_buffer.push_str(&filtered_chunk);
                                            if sender.send(chunk.to_string()).await.is_err() {
                                                println!("Failed to send chunk");
                                                break;
                                            }
                                        }

                                        if headers_done && json_buffer.contains('}') {
                                            println!("Full JSON assembled: {}", json_buffer);
                                            match serde_json::from_str::<AnuResponse>(&json_buffer) {
                                                Ok(anu_response) => {
                                                    if anu_response.success {
                                                        println!("Parsed JSON: {:?}", anu_response);
                                                        if sender.send(format!("FINAL:{}", json_buffer)).await.is_err() {
                                                            println!("Failed to send final response");
                                                        }
                                                        break;
                                                    } else {
                                                        println!("API returned success: false: {:?}", anu_response);
                                                        break;
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("JSON parsing failed: {}. Buffer: {}", e, json_buffer);
                                                }
                                            }
                                        }
                                    }
                                    Ok(0) => {
                                        if sender.send(format!("ERROR: Stream closed by server. \nLast chunk: {}", json_buffer)).await.is_err() {
                                            println!("Stream closed by server");
                                        }
                                        break;
                                    }
                                    Ok(_) => {
                                        eprintln!("Stream ?????");
                                        break;
                                    }
                                    Err(e) => {
                                        eprintln!("Read error: {}", e);
                                        break;
                                    }
                                }
                            }
                        } else {
                            eprintln!("Failed to write request or flush stream");
                        }
                    }
                    Err(e) => eprintln!("TLS connection error: {}", e),
                }
            }
            Err(e) => println!("TCP connection error: {}", e),
        }

        println!("fetch_anu_qrng_data completed");
    });
}











// 
// async fn get_qrng(
//     anu_data_type: Option<String>, 
//     anu_array_length: Option<u32>, 
//     anu_hex_block_size: Option<u32>
// ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
//     println!("function get_qrng");
//     let mut client_builder = reqwest::Client::builder();
//     
//     let lock_app_settings = crate::APP_SETTINGS.read();
//     let cccc = lock_app_settings.unwrap().clone();
//     let proxy_status = cccc.proxy_status.clone().unwrap();
//     let proxy_use_ssl = cccc.proxy_use_ssl.clone().unwrap();
//     let proxy_server_address = cccc.proxy_server_address.clone().unwrap();
//     let proxy_server_port = cccc.proxy_server_port.clone().unwrap();
//     let proxy_login_credentials = cccc.proxy_login_credentials.clone().unwrap();
//     let proxy_login_username = cccc.proxy_login_username.clone().unwrap();
//     let proxy_login_password = cccc.proxy_login_password.clone().unwrap();
//     let proxy_use_pac = cccc.proxy_use_pac.clone().unwrap();
//     let proxy_ssl_certificate = cccc.proxy_ssl_certificate.clone().unwrap();
// 
//     if proxy_status {
//         let proxy_address = format!(
//             "{}://{}:{}",
//             if proxy_use_ssl { "https" } else { "http" },
//             proxy_server_address,
//             proxy_server_port,
//         );
//         
//         let mut proxy = reqwest::Proxy::all(proxy_address)?;
//         
//         if proxy_login_credentials {
//             proxy = proxy.basic_auth(
//                 &proxy_login_username,
//                 &proxy_login_password,
//             );
//         }
//         
//         client_builder = client_builder.proxy(proxy);
//     }
//     
//     if proxy_use_pac {
//         // reqwest does not support PAC files - fuck
//         println!("Warning: PAC support is limited - using direct connection");
//     }
//     
//     if proxy_use_ssl && !proxy_ssl_certificate.is_empty() {
//         let cert = reqwest::Certificate::from_pem(
//             proxy_ssl_certificate.as_bytes()
//         )?;
//         client_builder = client_builder.add_root_certificate(cert);
//     }
//     
//     let client = client_builder.build()?;
//     println!("Client: {:?}", client);
//     
//     let url = format!(
//         "https://qrng.anu.edu.au/API/jsonI.php?length={}&type={}&size={}",
//         anu_array_length.unwrap_or(QRNG_MIN_ARRAY), 
//         anu_data_type.unwrap_or("hex16".to_string()), 
//         anu_hex_block_size.unwrap_or(anu_hex_block_size.clone().unwrap())
//     );
// 
//     println!("ANU URL: {:?}", url);
// 
//     let response = client
//         .get(&url)
//         .send()
//         .await?
//         .json::<AnuResponse>()
//         .await?;
// 
//     println!("API Response: {:?}", response);
// 
// 
//     if !response.success {
//         return Err("API request failed".into());
//     }
// 
//     Ok(response.data)
// }
