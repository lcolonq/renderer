extern crate gl_generator;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let dest = env::var("OUT_DIR").unwrap();
    let path = Path::new(&dest).join("bindings.rs");
    let mut file = File::create(&path).unwrap();

    Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, ["GL_EXT_texture_filter_anisotropic"])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}
