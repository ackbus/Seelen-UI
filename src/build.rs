use std::{fs::create_dir, path::PathBuf};

use slu_utils::{checksums::CheckSums, signature::sign_minisign};

fn main() {
    let _ = create_dir("gen");

    let mut checksums = CheckSums::new();
    read_folder_recursive(PathBuf::from("static"), &mut |path| {
        checksums.add(&path).unwrap();
    });

    let target_dir = target_dir();
    let sums_path = target_dir.join("SHA256SUMS");
    checksums.write(&sums_path).unwrap();

    if !cfg!(debug_assertions) {
        sign_sha256sums(&sums_path);
    } else {
        std::fs::write(
            sums_path.with_extension("sig"),
            "NOT SIGNED NEEDED FOR DEBUG",
        )
        .unwrap();
    }

    tauri_build::build();
}

fn target_dir() -> PathBuf {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // see <https://github.com/rust-lang/cargo/issues/5457>
    out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read_folder_recursive<F>(path: PathBuf, cb: &mut F)
where
    F: FnMut(PathBuf),
{
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            read_folder_recursive(path, cb);
        } else {
            cb(path);
        }
    }
}

fn sign_sha256sums(path: &PathBuf) {
    let key_base64 = match std::env::var("TAURI_SIGNING_PRIVATE_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            std::fs::write(path.with_extension("sig"), "NOT SIGNED (fork)").unwrap();
            return;
        }
    };
    let password = match std::env::var("TAURI_SIGNING_PRIVATE_KEY_PASSWORD") {
        Ok(pwd) if !pwd.is_empty() => pwd,
        _ => {
            std::fs::write(path.with_extension("sig"), "NOT SIGNED (fork)").unwrap();
            return;
        }
    };
    let data = std::fs::read(path).unwrap();
    let signature = sign_minisign(&data, &key_base64, password).unwrap();
    std::fs::write(path.with_extension("sig"), signature).unwrap();
}
