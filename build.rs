//! A build script to detect if the current compiler is nightly. If it is, we
//! automatically enable the `nightly` feature.

use rustc_version::{version_meta, Channel};

fn main() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}
