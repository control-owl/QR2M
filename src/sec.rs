// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

use crate::{FunctionOutput, d3bug};
use adw::prelude::*;
use gtk::glib::clone;
use gtk4 as gtk;
use libadwaita as adw;
use std::io::{self, Write};
use std::{fs, process::Command};

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

const CONTROL_OWL_KEY_ID: &str = "2524C8FEB60EFCB0";
const CONTROL_OWL_FINGERPRINT: &str = "C88E 6F25 736A D83D A1C7 57B2 2524 C8FE B60E FCB0";
const QR2M_KEY_ID: &str = "99204764AC6B6A44";
const QR2M_FINGERPRINT: &str = "DE39 6887 555C 656B 991D 768E 9920 4764 AC6B 6A44";

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

lazy_static::lazy_static! {
    pub static ref SECURITY_STATUS: std::sync::Arc<std::sync::RwLock<SecurityStatus>> = std::sync::Arc::new(std::sync::RwLock::new(SecurityStatus::new()));
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[derive(Clone)]
pub struct SecurityStatus {
  pub commit_hash: String,
  pub commit_date: String,
  pub commit_key: String,
  pub build_target: String,
  pub author_key: bool,
  pub app_key: bool,
  pub code_modified: bool,
}

impl SecurityStatus {
  fn new() -> Self {
    SecurityStatus {
      commit_hash: env!("COMMIT_HASH").to_string(),
      commit_date: env!("COMMIT_DATE").to_string(),
      commit_key: env!("COMMIT_KEY").to_string(),
      build_target: env!("BUILD_TARGET").to_string(),
      author_key: false,
      app_key: false,
      code_modified: false,
    }
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// fn hash_me_baby() -> String {
//   use sha1::{Digest, Sha1};
//   // let mut hasher = std::collections::hash_map::DefaultHasher::new();
//   let mut hasher = Sha1::new();
//   let mut paths = Vec::new();
//   get_project_files(std::path::Path::new("."), &mut paths);
//   paths.sort();
//
//   for path in paths {
//     if let Ok(contents) = fs::read(&path) {
//       // let rel_path = path.strip_prefix(".").unwrap().to_string_lossy();
//       let rel_path = path
//         .strip_prefix(".")
//         .unwrap()
//         .to_string_lossy()
//         .replace("\\", "/");
//       hasher.update(rel_path.as_bytes());
//       hasher.update(&contents);
//     }
//   }
//
//   format!("{:x}", hasher.finalize())
// }
//
// fn get_project_files(dir: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
//   if let Ok(entries) = fs::read_dir(dir) {
//     for entry in entries.filter_map(|e| e.ok()) {
//       let path = entry.path();
//       if path.is_file() {
//         let path_str = path.to_string_lossy();
//         if !path_str.contains(".git") && !path_str.contains("target") {
//           paths.push(path);
//         }
//       } else if path.is_dir() {
//         get_project_files(&path, paths);
//       }
//     }
//   }
// }

fn check_if_code_modified() -> FunctionOutput<bool> {
  d3bug(">>> check_if_code_modified", "debug");
  let output = Command::new("git")
    .arg("status")
    .arg("--porcelain")
    .output()
    .expect("Failed to execute git status");

  if output.stdout.is_empty() {
    println!("No changes since the last commit.");
    Ok(false)
  } else {
    println!("There are changes since the last commit:");
    io::stdout().write_all(&output.stdout).unwrap();
    Ok(true)
  }
}

pub fn check_security_level() -> FunctionOutput<()> {
  d3bug(">>> check_security_level", "debug");

  let mut security = SECURITY_STATUS.write().unwrap();

  // if security.commit_key == CONTROL_OWL_KEY_ID {
  //   security.author_key = true;
  // } else {
  //   security.author_key = false;
  // }
  security.author_key = security.commit_key == CONTROL_OWL_KEY_ID;

  match check_if_code_modified() {
    Ok(value) => {
      d3bug("<<< check_if_code_modified", "debug");
      security.code_modified = value;
    }
    Err(err) => {
      d3bug(&format!("check_if_code_modified: \n{:?}", err), "error");
      security.code_modified = true;
    }
  };

  let feature = qr2m_lib::get_active_app_feature();

  let sig_name = format!("{}-{}.sig", crate::APP_NAME.unwrap(), feature);
  let app_executable = std::env::current_exe().expect("Failed to get current executable path");
  let executable_dir = app_executable
    .parent()
    .expect("Failed to extract executable directory");
  let sig_full_path = format!("{}/{}", &executable_dir.to_string_lossy(), sig_name);

  // #[cfg(debug_assertions)]
  // println!("Signature file path: {}", sig_full_path);

  if std::path::Path::new(&sig_full_path).exists() {
    let output = Command::new("gpg")
      .args([
        "--verify",
        &sig_full_path,
        &app_executable.to_string_lossy(),
      ])
      .output()
      .expect("Failed to execute GPG verification");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let verification_succeeded = output.status.success()
      && (stdout.contains("Good signature") || !stderr.contains("BAD signature"));

    if !verification_succeeded {
      #[cfg(debug_assertions)]
      eprintln!(
        "\t- App signature verification failed! \n\tStdout: {}\n\tStderr: {}",
        stdout, stderr
      );

      let _ = fs::remove_file(&sig_full_path);
      let status = generate_new_app_signature(&app_executable, &sig_full_path);

      if status {
        #[cfg(debug_assertions)]
        println!(
          "\t- App signature created successfully. Status:\n{}",
          &status
        );
        security.app_key = true;
      } else {
        #[cfg(debug_assertions)]
        eprintln!("\t- App signing failed. No secret key found",);
        security.app_key = false;
      }
    } else {
      #[cfg(debug_assertions)]
      println!("\t- App signature verification succeeded");
      security.app_key = true;
    }
  } else {
    #[cfg(debug_assertions)]
    eprintln!("\t- App signature file not found. Try to generate new a one...");

    let status = generate_new_app_signature(&app_executable, &sig_full_path);

    if status {
      #[cfg(debug_assertions)]
      println!("\t- App signature created successfully");
      security.app_key = true;
    } else {
      #[cfg(debug_assertions)]
      eprintln!("\t- GPG signing failed for. No secret key found",);
      security.app_key = false;
    }
  }

  Ok(())
}

fn generate_new_app_signature(app_executable: &std::path::Path, sig_full_path: &str) -> bool {
  d3bug(">>> generate_new_app_signature", "debug");

  let key_check = Command::new("gpg")
    .args(["--list-secret-keys", QR2M_KEY_ID])
    .output();

  if let Err(_err) = key_check {
    d3bug(
      &format!("Failed to check GPG key {}: {}", QR2M_KEY_ID, _err),
      "error",
    );
    return false;
  }

  if !key_check.unwrap().status.success() {
    d3bug(
      &format!("GPG secret key {} not found", QR2M_KEY_ID),
      "error",
    );
    return false;
  }

  let output = Command::new("gpg")
    .args([
      "--detach-sign",
      "--armor",
      "-u",
      QR2M_KEY_ID,
      "-o",
      sig_full_path,
      app_executable.to_str().expect("Invalid executable path"),
    ])
    .output();

  match output {
    Ok(output) => {
      let _stderr = String::from_utf8_lossy(&output.stderr);
      let _stdout = String::from_utf8_lossy(&output.stdout);

      if !output.status.success() {
        d3bug(
          &format!(
            "GPG signing failed. \n\tStdout: {}\n\tStderr: {}",
            _stdout, _stderr
          ),
          "error",
        );

        return false;
      }

      if !std::path::Path::new(&sig_full_path).exists() {
        d3bug(
          &format!(
            "Signature file {} was not created despite successful GPG command",
            sig_full_path
          ),
          "error",
        );
        return false;
      }

      d3bug(
        &format!("Signature created successfully at {}", sig_full_path),
        "info",
      );
      true
    }
    Err(_err) => {
      d3bug(&format!("Failed to execute GPG signing: {}", _err), "error");
      false
    }
  }
}

pub fn create_security_window() -> gtk::ApplicationWindow {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.create_security_window").to_string());

  let security_window = gtk::ApplicationWindow::builder()
    .title(t!("UI.security").to_string())
    .default_width(450)
    .default_height(500)
    .resizable(false)
    .modal(true)
    .build();

  let main_sec_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  main_sec_box.set_halign(gtk::Align::Center);
  main_sec_box.set_valign(gtk::Align::Center);
  main_sec_box.set_margin_top(10);
  main_sec_box.set_margin_bottom(10);
  main_sec_box.set_margin_start(10);
  main_sec_box.set_margin_end(10);

  let scrolled_window = gtk::ScrolledWindow::new();
  scrolled_window.set_child(Some(&main_sec_box));
  scrolled_window.set_vexpand(true);

  let security_icon_path = std::path::Path::new("theme").join("color");
  let security = SECURITY_STATUS.read().unwrap();

  let security_image = if security.app_key && security.author_key && !security.code_modified {
    qr2m_lib::get_picture_from_resources(
      security_icon_path
        .join(format!("sec-good-128.{}", crate::GUI_IMAGE_EXTENSION))
        .to_str()
        .unwrap_or(&format!(
          "theme/color/sec-good-128.{}",
          crate::GUI_IMAGE_EXTENSION
        )),
    )
  } else if security.app_key && security.author_key {
    qr2m_lib::get_picture_from_resources(
      security_icon_path
        .join(format!("sec-warn-128.{}", crate::GUI_IMAGE_EXTENSION))
        .to_str()
        .unwrap_or(&format!(
          "theme/color/sec-warn-128.{}",
          crate::GUI_IMAGE_EXTENSION
        )),
    )
  } else {
    qr2m_lib::get_picture_from_resources(
      security_icon_path
        .join(format!("sec-error-128.{}", crate::GUI_IMAGE_EXTENSION))
        .to_str()
        .unwrap_or(&format!(
          "theme/color/sec-error-128.{}",
          crate::GUI_IMAGE_EXTENSION
        )),
    )
  };

  let image = gtk::Image::from_paintable(security_image.paintable().as_ref());
  image.set_pixel_size(128);
  main_sec_box.append(&image);

  let status_title = if security.app_key && security.author_key && !security.code_modified {
    &t!("UI.security.status.verified")
  } else if security.app_key && security.author_key {
    &t!("UI.security.status.modified")
  } else {
    &t!("UI.security.status.error")
  };

  let status_label = gtk::Label::new(Some(status_title));
  status_label.set_css_classes(&["h1"]);
  status_label.set_margin_top(10);
  status_label.set_justify(gtk::Justification::Center);
  main_sec_box.append(&status_label);

  if security.commit_key != "None" {
    if security.author_key {
      if security.code_modified {
        let changed_message = gtk::Label::new(Some(&t!("UI.security.keys.author.modified")));
        // verified_message.set_margin_top(10);
        changed_message.set_wrap(true);
        changed_message.set_css_classes(&["security-modified"]);
        changed_message.set_justify(gtk::Justification::Left);
        main_sec_box.append(&changed_message);
      } else {
        let verified_message = gtk::Label::new(Some(&t!("UI.security.keys.author.verified")));
        // verified_message.set_margin_top(10);
        verified_message.set_wrap(true);
        verified_message.set_css_classes(&["security-verified"]);
        verified_message.set_justify(gtk::Justification::Left);
        main_sec_box.append(&verified_message);
      }
    } else {
      let error_message = gtk::Label::new(Some(&t!("UI.security.keys.author.error")));
      error_message.set_wrap(true);
      error_message.set_css_classes(&["security-error"]);
      error_message.set_justify(gtk::Justification::Left);
      main_sec_box.append(&error_message);
    }
  } else {
    let not_signed_label = gtk::Label::new(Some(&t!("UI.security.keys.author.no_sign")));
    // not_signed_label.set_margin_top(10);
    not_signed_label.set_wrap(true);
    not_signed_label.set_css_classes(&["security-error"]);
    not_signed_label.set_justify(gtk::Justification::Left);
    main_sec_box.append(&not_signed_label);
  }

  if security.app_key {
    let verified_message = gtk::Label::new(Some(&t!("UI.security.keys.app.present")));
    // verified_message.set_margin_top(10);
    verified_message.set_wrap(true);
    verified_message.set_css_classes(&["security-verified"]);
    verified_message.set_justify(gtk::Justification::Left);
    main_sec_box.append(&verified_message);
  } else {
    let error_message = gtk::Label::new(Some(&t!("UI.security.keys.app.missing")));
    error_message.set_wrap(true);
    error_message.set_css_classes(&["security-error"]);
    error_message.set_justify(gtk::Justification::Left);
    main_sec_box.append(&error_message);
  }

  let build_info_label = gtk::Label::new(Some(&t!("UI.security.info.build")));
  build_info_label.set_css_classes(&["h2"]);
  build_info_label.set_margin_top(20);
  main_sec_box.append(&build_info_label);

  let build_details = gtk::Label::new(Some(&format!(
    "• {}: {}\n\
     • {}: {}\n\
     • {}: {}\n\
     • {}: {}",
    t!("UI.security.details.hash"),
    security.commit_hash,
    t!("UI.security.details.date"),
    security.commit_date,
    t!("UI.security.details.key"),
    if security.commit_key == "None" {
      t!("UI.security.details.no_sign").to_string()
    } else {
      security.commit_key.to_string()
    },
    t!("UI.security.details.platform"),
    security.build_target,
  )));

  build_details.set_margin_top(5);
  build_details.set_justify(gtk::Justification::Left);
  main_sec_box.append(&build_details);

  let our_key_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  let our_label = gtk::Label::new(Some(&t!("UI.security.details.developer")));
  our_label.set_margin_top(10);
  our_label.set_css_classes(&["h2"]);

  let control_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  let qr2m_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  our_key_box.append(&our_label);
  our_key_box.append(&control_box);
  our_key_box.append(&qr2m_box);

  let control_owl_name = gtk::Label::new(Some("Control Owl"));
  control_box.append(&control_owl_name);

  let control_online_checker = format!("https://keys.openpgp.org/search?q={}", CONTROL_OWL_KEY_ID);

  let control_link = gtk::LinkButton::builder()
    .uri(control_online_checker)
    .label(CONTROL_OWL_KEY_ID)
    .build();

  control_link.set_halign(gtk::Align::Center);
  control_box.append(&control_link);

  let control_fingerprint = gtk::Label::new(Some(CONTROL_OWL_FINGERPRINT));
  control_box.append(&control_fingerprint);

  let qr2m_label = gtk::Label::new(Some("QR2M:"));
  qr2m_label.set_margin_top(10);
  qr2m_box.append(&qr2m_label);

  let qr2m_online_checker = format!("https://keys.openpgp.org/search?q={}", QR2M_KEY_ID);

  let qr2m_link = gtk::LinkButton::builder()
    .uri(qr2m_online_checker)
    .label(QR2M_KEY_ID)
    .build();

  qr2m_link.set_halign(gtk::Align::Center);
  qr2m_box.append(&qr2m_link);

  let qr2m_fingerprint_label = gtk::Label::new(Some(QR2M_FINGERPRINT));
  qr2m_box.append(&qr2m_fingerprint_label);

  main_sec_box.append(&our_key_box);

  let close_button = gtk::Button::with_label(&t!("UI.button.close"));

  close_button.connect_clicked(clone!(
    #[strong]
    security_window,
    move |_| {
      security_window.close();
    }
  ));

  let security_layout_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  security_layout_box.set_margin_bottom(10);
  security_layout_box.set_margin_start(10);
  security_layout_box.set_margin_end(10);
  security_layout_box.append(&scrolled_window);
  security_layout_box.append(&close_button);

  security_window.set_child(Some(&security_layout_box));

  security_window
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
