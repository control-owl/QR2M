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

const QRNG_DEF_BLOCK_SIZE: u32 = 1024;
const QRNG_MIN_ARRAY: u32 = 24;

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
        .default_width(crate::WINDOW_SETTINGS_DEFAULT_WIDTH.try_into().unwrap())
        .default_height(crate::WINDOW_SETTINGS_DEFAULT_HEIGHT.try_into().unwrap())
        .resizable(true)
        .modal(true)
        .build();


    let lock_app_settings = crate::APP_SETTINGS.lock().unwrap();
    let anu_data_type = lock_app_settings.anu_data_format.clone();
    let anu_array_length = lock_app_settings.anu_array_length.clone();
    let anu_hex_block_size = lock_app_settings.anu_hex_block_size.clone();

    let main_grid_box = gtk::Box::builder()
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .orientation(gtk::Orientation::Vertical)
        .build();

    let scroll_window = gtk::ScrolledWindow::new();
    scroll_window.set_hexpand(true);
    scroll_window.set_vexpand(true);

    let main_container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let blocks = create_boxes(anu_array_length);
    
    let blocks_rc = std::rc::Rc::new(std::cell::RefCell::new(blocks));
    
    for block in blocks_rc.borrow().iter() {
        main_container.append(&block.container);
    }

    main_grid_box.append(&scroll_window);
    scroll_window.set_child(Some(&main_container));

    let main_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let ok_button = gtk::Button::with_label("OK");
    let cancel_button = gtk::Button::with_label("Cancel");
    let new_button = gtk::Button::with_label("New QRNG");

    main_button_box.append(&ok_button);
    main_button_box.append(&new_button);
    main_button_box.append(&cancel_button);

    main_button_box.set_margin_bottom(4);
    main_button_box.set_margin_top(4);
    main_button_box.set_margin_start(4);
    main_button_box.set_margin_end(4);
    main_button_box.set_halign(gtk::Align::Center);
    main_grid_box.append(&main_button_box);
    window.set_child(Some(&main_grid_box));

    // let (tx, rx): (std::sync::mpsc::Sender<Result<Vec<String>, Box<dyn std::error::Error>>>, std::sync::mpsc::Receiver<_>) = std::sync::mpsc::channel();

    let (tx, rx): (std::sync::mpsc::Sender<Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>>, std::sync::mpsc::Receiver<_>) = std::sync::mpsc::channel();
    let task_handle: std::rc::Rc<std::cell::RefCell<Option<tokio::task::JoinHandle<()>>>> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let blocks_rc_clone = blocks_rc.clone();
    
    
    let pulse_active = std::rc::Rc::new(std::cell::RefCell::new(false));
    let pulse_active_clone = pulse_active.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if *pulse_active.borrow() {
            for block in blocks_rc_clone.borrow().iter() {
                block.progress_bar.pulse();
            }
            glib::ControlFlow::Continue
        } else {
            glib::ControlFlow::Break
        }
    });
    
    
    let pulse_active = std::rc::Rc::new(std::cell::RefCell::new(false));
    let blocks_rc_clone = blocks_rc.clone();
    glib::idle_add_local(move || {
        if let Ok(result) = rx.try_recv() {
            *pulse_active_clone.borrow_mut() = false;
            match result {
                Ok(data) => {
                    let blocks = blocks_rc_clone.borrow();
                    for (i, block) in blocks.iter().enumerate() {
                        if i < data.len() {
                            block.entry.set_text(&data[i]);
                            block.progress_bar.set_fraction(1.0);
                        }
                    }
                }
                Err(e) => {
                    println!("Error receiving QRNG data: {:?}", e);
                    let blocks = blocks_rc_clone.borrow();
                    for block in blocks.iter() {
                        block.entry.set_text("Error occurred");
                        block.progress_bar.set_fraction(0.0);
                    }
                }
            }
        }
        glib::ControlFlow::Continue
    });

    new_button.connect_clicked(glib::clone!(
        #[strong] task_handle,
        #[strong] blocks_rc,
        #[strong] pulse_active,
        move |_| {
            let tx = tx.clone();
            let blocks = blocks_rc.borrow();
            
            for block in blocks.iter() {
                block.entry.set_text("Loading...");
                block.progress_bar.set_fraction(0.0);
            }
            *pulse_active.borrow_mut() = true;
            
            if let Some(handle) = task_handle.borrow_mut().take() {
                handle.abort();
                println!("Previous task aborted.");
            }
            
            let anu_data_type_clone = anu_data_type.clone();
            let new_handle = tokio::spawn(async move {
                let result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(60),
                    get_qrng(anu_data_type_clone, anu_array_length, anu_hex_block_size)
                ).await;
                
                let _ = tx.send(match result {
                    Ok(Ok(data)) => Ok(data),
                    Ok(Err(_)) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "stupid ANU error")) as Box<dyn std::error::Error + Send + Sync>),
                    Err(_) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "QRNG fetch timed out")) as Box<dyn std::error::Error + Send + Sync>),
                });
            });

            *task_handle.borrow_mut() = Some(new_handle);
        }
    ));
    
    cancel_button.connect_clicked(glib::clone!(
        #[strong] task_handle,
        #[strong] pulse_active,
        #[weak] window,
        move |_| {
            *pulse_active.borrow_mut() = false;
            if let Some(handle) = task_handle.borrow_mut().take() {
                println!("aborting async task before closing...");
                handle.abort();
            }
            window.close();
        }
    ));

    window.connect_close_request(glib::clone!(
        #[strong] task_handle,
        #[strong] pulse_active,
        move |_| {
            *pulse_active.borrow_mut() = false;
            if let Some(handle) = task_handle.borrow_mut().take() {
                println!("aborting async task on window close...");
                handle.abort();
            }
            glib::Propagation::Proceed
        }
    ));
    
    window
}

async fn get_qrng(
    anu_data_type: Option<String>, 
    anu_array_length: Option<u32>, 
    anu_hex_block_size: Option<u32>
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://qrng.anu.edu.au/API/jsonI.php?length={}&type={}&size={}",
        anu_array_length.unwrap_or(QRNG_MIN_ARRAY), 
        anu_data_type.unwrap_or("hex16".to_string()), 
        anu_hex_block_size.unwrap_or(QRNG_DEF_BLOCK_SIZE)
    );

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<AnuResponse>()
        .await?;

    println!("API Response: {:?}", response);

    if !response.success {
        return Err("API request failed".into());
    }

    Ok(response.data)
}






















// BOXES - too memory intensive
// pub fn anu_window() -> gtk::ApplicationWindow {
//     let app = gtk::ApplicationWindow::builder()
//         .title(t!("UI.anu").to_string())
//         .default_width(crate::WINDOW_SETTINGS_DEFAULT_WIDTH.try_into().unwrap())
//         .default_height(crate::WINDOW_SETTINGS_DEFAULT_HEIGHT.try_into().unwrap())
//         .resizable(true)
//         .modal(true)
//         .build();
// 
// 
//     let main_grid_box = gtk::Box::builder()
//         .margin_bottom(10)
//         .margin_end(10)
//         .margin_start(10)
//         .margin_top(10)
//         .orientation(gtk::Orientation::Vertical)
//         .build();
// 
//     let scroll_window = gtk::ScrolledWindow::new();
//     scroll_window.set_hexpand(true);
//     scroll_window.set_vexpand(true);
// 
//     let grid = gtk::Grid::builder()
//         .column_spacing(0)
//         .row_spacing(0)
//         .build();
// 
//     scroll_window.set_child(Some(&grid));
//     main_grid_box.append(&scroll_window);
// 
//     let main_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
//     let ok_button = gtk::Button::with_label("OK");
//     let cancel_button = gtk::Button::with_label("Cancel");
//     let new_button = gtk::Button::with_label("New QRNG");
// 
//     main_button_box.append(&ok_button);
//     main_button_box.append(&new_button);
//     main_button_box.append(&cancel_button);
// 
//     main_button_box.set_margin_bottom(4);
//     main_button_box.set_margin_top(4);
//     main_button_box.set_margin_start(4);
//     main_button_box.set_margin_end(4);
// 
//     main_grid_box.append(&main_button_box);
//     app.set_child(Some(&main_grid_box));
// 
// 
// 
// 
// 
// 
// 
//     let boxes = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
//     for _ in 0..QRNG_MAGIC_NUMBER {
//         let small_box = gtk::Box::builder()
//             .width_request(BOX_SIZE as i32)
//             .height_request(BOX_SIZE as i32)
//             .build();
//         small_box.set_css_classes(&["empty-box"]);
//         boxes.borrow_mut().push(small_box);
//     }
// 
//     let initial_boxes = boxes.borrow();
//     let initial_columns = ((QRNG_BLOCK_SIZE - MARGIN_TOTAL) / BOX_SIZE).max(1) as usize;
//     for (i, small_box) in initial_boxes.iter().enumerate() {
//         grid.attach(small_box, (i % initial_columns) as i32, (i / initial_columns) as i32, 1, 1);
//     }
// 
//     drop(initial_boxes);
//    
//     let reallocate_boxes = {
//         let grid = grid.clone();
//         let boxes = boxes.clone();
//         let mut last_width = app.width() - MARGIN_TOTAL as i32;
//         move |mut width: i32| {
//             if width <= 0 {
//                 width = crate::WINDOW_SETTINGS_DEFAULT_WIDTH as i32
//             }
//             let effective_width = width - MARGIN_TOTAL as i32;
//             if effective_width != last_width {
//                 let columns = (effective_width / BOX_SIZE as i32).max(1) as usize;
//                 let boxes = boxes.borrow();
//                 for small_box in boxes.iter() {
//                     if small_box.parent().map_or(false, |p| p == *grid.upcast_ref::<gtk::Widget>()) {
//                         grid.remove(small_box);
//                     }
//                 }
//                 for (i, small_box) in boxes.iter().enumerate() {
//                     grid.attach(small_box, (i % columns) as i32, (i / columns) as i32, 1, 1);
//                 }
//                 println!("width={}, effective_width={}, columns={}", width, effective_width, columns);
//                 last_width = effective_width;
//             }
//         }
//     };
// 
//     let mut reallocate_boxes_clone = reallocate_boxes.clone();
//     reallocate_boxes_clone(app.width());
//     
//     // glib::idle_add_local(glib::clone!(
//     //     #[strong] app,
//     //     // #[strong] reallocate_boxes,
//     //     move || {
//     //         if app.is_active() {
//     //             let mut width = app.width();
//     //             if width == 0 {
//     //                 width = crate::WINDOW_SETTINGS_DEFAULT_WIDTH as i32;
//     //             }
//     //             reallocate_boxes_clone(width);
//     //             glib::ControlFlow::Continue
//     //         } else {
//     //             println!("Stopping reallocate_boxes_clone loop because app is closed.");
//     //             glib::ControlFlow::Break
//     //         }
//     // }));
// 
//     app.connect_default_width_notify(glib::clone!(
//         // #[strong] app,
//         move |app| {
//             if app.is_visible() && app.is_mapped() {
//                 println!("--------------------------------------------------------------resize event");
//                 
//                 // last_resize_time.set(std::time::Instant::now());
//                 let mut reallocate_boxes_clone = reallocate_boxes.clone();
//                 
//                 
//                 
//                 let last_resize_width = std::rc::Rc::new(std::cell::Cell::new(app.width()));
//                 // let app_width = app.width();
// 
//                 glib::timeout_add_local(std::time::Duration::from_millis(500), glib::clone!(
//                     #[strong] app,
//                     #[strong] last_resize_width,
//                     move || {
// 
//                         if *last_resize_width == app.width().into() {
//                             println!("same width");
//                             return glib::ControlFlow::Break;
//                         } else {
//                             if app.is_visible() && app.is_mapped() {
//                                 // let elapsed = last_resize_time.get().elapsed();
//                                 // if elapsed >= std::time::Duration::from_millis(500) {
//                                     // let mut reallocate_boxes_clone = reallocate_boxes.clone();
//                                     println!("--------------------------------------------------------------resize executed");
//                                     reallocate_boxes_clone(app.width());
//                                     return glib::ControlFlow::Break;
//                                 // }
//                                 // glib::ControlFlow::Continue
//                             } else {
//                                 println!("Stopping timeout because app is closed.");
//                                 glib::ControlFlow::Break
//                             }
// 
//                         }
//                     }
//                 ));
//             }
//         }
//     ));
// 
//     let (tx, rx) = std::sync::mpsc::channel();
//     let rx = std::rc::Rc::new(std::cell::RefCell::new(rx));
//     let task_handle: std::rc::Rc<std::cell::RefCell<Option<tokio::task::JoinHandle<()>>>> = std::rc::Rc::new(std::cell::RefCell::new(None));
// 
//     
//     new_button.connect_clicked(glib::clone!(
//         #[strong] task_handle,
//         // #[weak] app_messages_state,
//         move |_| {
//             let tx = tx.clone();
// 
//             if let Some(handle) = task_handle.borrow_mut().take() {
//                 handle.abort();
//                 println!("Previous task aborted.");
//             }
// 
//             
//             // let new_handle = tokio::spawn(async move {
//             //     let qrng_string = get_qrng().await;
//             //     tx.send(qrng_string).expect("Failed to send QRNG result");
//             // });
// 
//             // IMPLEMENT: ANU API Timeout
//             let new_handle = tokio::spawn(async move {
//                 match tokio::time::timeout(tokio::time::Duration::from_secs(3), get_qrng()).await {
//                     Ok(qrng_string) => {
//                         let _ = tx.send(qrng_string);
//                     }
//                     Err(_) => println!("QRNG fetch timed out."),
//                 }
//             });
//     
// 
//             *task_handle.borrow_mut() = Some(new_handle);
//         }
//     ));
//     
// 
// 
//     let boxes_clone = boxes.clone();
//     let app_weak = app.downgrade();
//     let rx_clone = rx.clone();
// 
//     glib::idle_add_local(move || {
//         if let Some(_app) = app_weak.upgrade() {
//             match rx_clone.borrow().try_recv() {
//                 Ok(qrng_string) => {
//                     for (i, small_box) in boxes_clone.borrow().iter().enumerate() {
//                         if i < qrng_string.len() {
//                             small_box.set_css_classes(&["green-box"]);
//                         } else {
//                             small_box.set_css_classes(&["empty-box"]);
//                         }
//                     }
//                 }
//                 Err(_) => {}
//             }
//             glib::ControlFlow::Continue
//         } else {
//             println!("Stopping idle function because anu window is closed");
//             glib::ControlFlow::Break
//         }
//     });
// 
// 
//     cancel_button.connect_clicked(glib::clone!(
//         #[strong] task_handle,
//         #[weak] app,
//         move |_| {
//             if let Some(handle) = task_handle.borrow_mut().take() {
//                 println!("aborting async task before closing...");
//                 handle.abort();
//             }
//             app.close();
//         }
//     ));
// 
//     app.connect_close_request(glib::clone!(
//         #[strong] task_handle,
//         // #[strong] rx,
//         move |_| {
//             if let Some(handle) = task_handle.borrow_mut().take() {
//                 println!("aborting async task on window close...");
//                 handle.abort();
//             }
//             // rx.borrow_mut();
//             
//             glib::Propagation::Proceed
//         }
//     ));
// 
// 
//     app
// }
// 

