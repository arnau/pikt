use bindgen::Builder;
use std::env;
use std::path::PathBuf;

fn main() {
    let lib_name = "pikchr";
    let lib_path = "pikchr/pikchr.c";
    let header_path = "pikchr/pikchr.h";

    println!("cargo:rerun-if-changed={}", lib_path);
    println!("cargo:rerun-if-changed={}", header_path);

    cc::Build::new().file(lib_path).compile(lib_name);
    println!("cargo:rustc-link-lib={}", lib_name);

    let bindings = Builder::default()
        .header(header_path)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
