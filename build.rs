use reqwest;
use std::fs;
use std::path::Path;

fn main() {
    let url_path = [
        (
            "https://github.com/saadeghi/daisyui/releases/latest/download/daisyui.mjs",
            Path::new("style").join("daisyui.mjs"),
        ),
        (
            "https://github.com/saadeghi/daisyui/releases/latest/download/daisyui-theme.mjs",
            Path::new("style").join("daisyui-theme.mjs"),
        ),
    ];

    for (url, file_path) in url_path {
        if !file_path.exists() {
            let response = reqwest::blocking::get(url).expect("Failed to download file");
            let bytes = response.bytes().expect("Failed to get bytes from response");
            fs::write(&file_path, bytes).expect("Failed to write downloaded file");
        }

        // Tell Cargo to re-run if this file changes or doesn't exist
        println!("cargo:rerun-if-changed={}", file_path.display());
    }
}
