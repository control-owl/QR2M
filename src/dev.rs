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

const QRNG_BLOCK_SIZE: u32 = 512;
const QRNG_KEY_LEVEL: u32 = 24;
const QRNG_MAGIC_NUMBER: u32 = QRNG_BLOCK_SIZE * QRNG_KEY_LEVEL;
const BOX_SIZE: u32 = 5;
const MARGIN_TOTAL: u32 = 20;



async fn get_qrng() -> String {
    use rand::{Rng, rng};

    let mut rng = rng();
    let length = rng.random_range(2..=QRNG_MAGIC_NUMBER);

    let hex_chars: String = (0..length)
        .map(|_| {
            let random_char = rng.random_range(0..16);
            match random_char {
                0..=9 => (b'0' + random_char as u8) as char,
                10..=15 => (b'A' + (random_char - 10) as u8) as char, // A-F
                _ => unreachable!(),
            }
        })
        .collect();

    println!("Generated String: {}", hex_chars);
    hex_chars
}

pub fn anu_window() -> gtk::ApplicationWindow {
    let app = gtk::ApplicationWindow::builder()
        .title(t!("UI.anu").to_string())
        .default_width(crate::WINDOW_SETTINGS_DEFAULT_WIDTH.try_into().unwrap())
        .default_height(crate::WINDOW_SETTINGS_DEFAULT_HEIGHT.try_into().unwrap())
        .resizable(true)
        .modal(true)
        .build();


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

    let grid = gtk::Grid::builder()
        .column_spacing(0)
        .row_spacing(0)
        .build();

    scroll_window.set_child(Some(&grid));
    main_grid_box.append(&scroll_window);

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

    main_grid_box.append(&main_button_box);
    app.set_child(Some(&main_grid_box));







    let boxes = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    for _ in 0..QRNG_MAGIC_NUMBER {
        let small_box = gtk::Box::builder()
            .width_request(BOX_SIZE as i32)
            .height_request(BOX_SIZE as i32)
            .build();
        small_box.set_css_classes(&["empty-box"]);
        boxes.borrow_mut().push(small_box);
    }

    let initial_boxes = boxes.borrow();
    let initial_columns = ((QRNG_BLOCK_SIZE - MARGIN_TOTAL) / BOX_SIZE).max(1) as usize;
    for (i, small_box) in initial_boxes.iter().enumerate() {
        grid.attach(small_box, (i % initial_columns) as i32, (i / initial_columns) as i32, 1, 1);
    }

    drop(initial_boxes);
   
    let reallocate_boxes = {
        let grid = grid.clone();
        let boxes = boxes.clone();
        let mut last_width = app.width() - MARGIN_TOTAL as i32;
        move |mut width: i32| {
            if width <= 0 {
                width = crate::WINDOW_SETTINGS_DEFAULT_WIDTH as i32
            }
            let effective_width = width - MARGIN_TOTAL as i32;
            if effective_width != last_width {
                let columns = (effective_width / BOX_SIZE as i32).max(1) as usize;
                let boxes = boxes.borrow();
                for small_box in boxes.iter() {
                    if small_box.parent().map_or(false, |p| p == *grid.upcast_ref::<gtk::Widget>()) {
                        grid.remove(small_box);
                    }
                }
                for (i, small_box) in boxes.iter().enumerate() {
                    grid.attach(small_box, (i % columns) as i32, (i / columns) as i32, 1, 1);
                }
                println!("width={}, effective_width={}, columns={}", width, effective_width, columns);
                last_width = effective_width;
            }
        }
    };

    let mut reallocate_boxes_clone = reallocate_boxes.clone();
    reallocate_boxes_clone(app.width());
    
    // glib::idle_add_local(glib::clone!(
    //     #[strong] app,
    //     // #[strong] reallocate_boxes,
    //     move || {
    //         if app.is_active() {
    //             let mut width = app.width();
    //             if width == 0 {
    //                 width = crate::WINDOW_SETTINGS_DEFAULT_WIDTH as i32;
    //             }
    //             reallocate_boxes_clone(width);
    //             glib::ControlFlow::Continue
    //         } else {
    //             println!("Stopping reallocate_boxes_clone loop because app is closed.");
    //             glib::ControlFlow::Break
    //         }
    // }));

    app.connect_default_width_notify(glib::clone!(
        // #[strong] app,
        move |app| {
            if app.is_visible() && app.is_mapped() {
                println!("--------------------------------------------------------------resize event");
                let last_resize_time = std::rc::Rc::new(std::cell::Cell::new(std::time::Instant::now()));
                
                last_resize_time.set(std::time::Instant::now());
                let mut reallocate_boxes_clone = reallocate_boxes.clone();



                let app_width = app.width();

                glib::timeout_add_local(std::time::Duration::from_millis(500), glib::clone!(
                    #[strong] app,
                    #[strong] last_resize_time,
                    move || {

                        if app_width == app.width() {
                            println!("same width");
                            return glib::ControlFlow::Break;
                        } else {
                            if app.is_visible() && app.is_mapped() {
                                let elapsed = last_resize_time.get().elapsed();
                                if elapsed >= std::time::Duration::from_millis(1000) {
                                    // let mut reallocate_boxes_clone = reallocate_boxes.clone();
                                    println!("--------------------------------------------------------------resize executed");
                                    reallocate_boxes_clone(app.width());
                                    return glib::ControlFlow::Break;
                                }
                                glib::ControlFlow::Continue
                            } else {
                                println!("Stopping timeout because app is closed.");
                                glib::ControlFlow::Break
                            }

                        }
                    }
                ));
            }
        }
    ));

    let (tx, rx) = std::sync::mpsc::channel();
    let rx = std::rc::Rc::new(std::cell::RefCell::new(rx));
    let task_handle: std::rc::Rc<std::cell::RefCell<Option<tokio::task::JoinHandle<()>>>> = std::rc::Rc::new(std::cell::RefCell::new(None));

    
    new_button.connect_clicked(glib::clone!(
        #[strong] task_handle,
        // #[weak] app_messages_state,
        move |_| {
            let tx = tx.clone();

            if let Some(handle) = task_handle.borrow_mut().take() {
                handle.abort();
                println!("Previous task aborted.");
            }

            
            // let new_handle = tokio::spawn(async move {
            //     let qrng_string = get_qrng().await;
            //     tx.send(qrng_string).expect("Failed to send QRNG result");
            // });

            // IMPLEMENT: ANU API Timeout
            let new_handle = tokio::spawn(async move {
                match tokio::time::timeout(tokio::time::Duration::from_secs(3), get_qrng()).await {
                    Ok(qrng_string) => {
                        let _ = tx.send(qrng_string);
                    }
                    Err(_) => println!("QRNG fetch timed out."),
                }
            });
    

            *task_handle.borrow_mut() = Some(new_handle);
        }
    ));
    


    let boxes_clone = boxes.clone();
    let app_weak = app.downgrade();
    let rx_clone = rx.clone();

    glib::idle_add_local(move || {
        if let Some(_app) = app_weak.upgrade() {
            match rx_clone.borrow().try_recv() {
                Ok(qrng_string) => {
                    for (i, small_box) in boxes_clone.borrow().iter().enumerate() {
                        if i < qrng_string.len() {
                            small_box.set_css_classes(&["green-box"]);
                        } else {
                            small_box.set_css_classes(&["empty-box"]);
                        }
                    }
                }
                Err(_) => {}
            }
            glib::ControlFlow::Continue
        } else {
            println!("Stopping idle function because anu window is closed");
            glib::ControlFlow::Break
        }
    });


    cancel_button.connect_clicked(glib::clone!(
        #[strong] task_handle,
        #[weak] app,
        move |_| {
            if let Some(handle) = task_handle.borrow_mut().take() {
                println!("aborting async task before closing...");
                handle.abort();
            }
            app.close();
        }
    ));

    app.connect_close_request(glib::clone!(
        #[strong] task_handle,
        // #[strong] rx,
        move |_| {
            if let Some(handle) = task_handle.borrow_mut().take() {
                println!("aborting async task on window close...");
                handle.abort();
            }
            // rx.borrow_mut();
            
            glib::Propagation::Proceed
        }
    ));


    app
}
