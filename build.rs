extern crate bindgen;
#[macro_use]
extern crate serde;
extern crate serde_json;

use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug)]
struct Layout {
    #[serde(rename = "BINDIR")]
    bin_dir: String,

    #[serde(rename = "INCLUDEDIR")]
    include_dir: String,

    #[serde(rename = "PLUGINDIR")]
    plugin_dir: String,
}

// Exec `traffic_layout` and extract the Layout from the JSON result.
fn get_layout() -> Layout {
    let result = Command::new("traffic_layout")
        .args(&["--json"])
        .stdin(Stdio::null())
        .output()
        .expect("failed to spawn \"traffic_layout\"");

    if !result.status.success() {
        panic!(
            "traffic_layout --json failed: {}",
            String::from_utf8(result.stdout).unwrap()
        );
    }

    let s = String::from_utf8(result.stdout).unwrap();

    let l: Layout = serde_json::from_str(&s.as_str()).expect("invalid layout");
    return l;
}

fn main() {
    if env::var("BUILD_TS_BINDINGS").is_err() {
        return;
    }

    let include_dir = env::var("TS_INCLUDE").unwrap_or_else(|_e| get_layout().include_dir);

    let bindings = bindgen::Builder::default()
        .header("trafficserver.h")
        .clang_arg(format!("-I{}", include_dir))
        .whitelist_function("TS.*")
        .whitelist_type("TS.*")
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src/bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
