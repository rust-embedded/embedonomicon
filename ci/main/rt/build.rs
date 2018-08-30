use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<Error>> {
    // build directory for this crate
    let mut out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // extend the library search path
    println!("cargo:rustc-link-search={}", out.display());

    // put `link.x` in the build directory
    out.push("link.x");
    File::create(out)?.write_all(include_bytes!("link.x"))?;

    Ok(())
}
