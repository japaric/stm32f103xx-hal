use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[cfg(all(feature = "stm32f103x4", not(any(feature = "stm32f103x6", feature = "stm32f103x8", feature = "stm32f103xB", feature = "stm32f103xC", feature = "stm32f103xD", feature = "stm32f103xE", feature = "stm32f103xF", feature = "stm32f103xG"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103x4.x")
    }
}

#[cfg(all(feature = "stm32f103x6", not(any(feature = "stm32f103x4", feature = "stm32f103x8", feature = "stm32f103xB", feature = "stm32f103xC", feature = "stm32f103xD", feature = "stm32f103xE", feature = "stm32f103xF", feature = "stm32f103xG"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103x6.x")
    }
}

#[cfg(all(feature = "stm32f103x8", not(any(feature = "stm32f103x4", feature = "stm32f103x6", feature = "stm32f103xB", feature = "stm32f103xC", feature = "stm32f103xD", feature = "stm32f103xE", feature = "stm32f103xF", feature = "stm32f103xG"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103x8.x")
    }
}

#[cfg(all(feature = "stm32f103xB", not(any(feature = "stm32f103x4", feature = "stm32f103x6", feature = "stm32f103x8", feature = "stm32f103xC", feature = "stm32f103xD", feature = "stm32f103xE", feature = "stm32f103xF", feature = "stm32f103xG"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103xB.x")
    }
}

#[cfg(all(feature = "stm32f103xC", not(any(feature = "stm32f103x4", feature = "stm32f103x6", feature = "stm32f103x8", feature = "stm32f103xB", feature = "stm32f103xD", feature = "stm32f103xE", feature = "stm32f103xF", feature = "stm32f103xG"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103xC.x")
    }
}

#[cfg(all(feature = "stm32f103xD", not(any(feature = "stm32f103x4", feature = "stm32f103x6", feature = "stm32f103x8", feature = "stm32f103xB", feature = "stm32f103xC", feature = "stm32f103xE", feature = "stm32f103xF", feature = "stm32f103xG"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103xD.x")
    }
}

#[cfg(all(feature = "stm32f103xE", not(any(feature = "stm32f103x4", feature = "stm32f103x6", feature = "stm32f103x8", feature = "stm32f103xB", feature = "stm32f103xC", feature = "stm32f103xD", feature = "stm32f103xF", feature = "stm32f103xG"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103xE.x")
    }
}

#[cfg(all(feature = "stm32f103xF", not(any(feature = "stm32f103x4", feature = "stm32f103x6", feature = "stm32f103x8", feature = "stm32f103xB", feature = "stm32f103xC", feature = "stm32f103xD", feature = "stm32f103xE", feature = "stm32f103xG"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103xF.x")
    }
}

#[cfg(all(feature = "stm32f103xG", not(any(feature = "stm32f103x4", feature = "stm32f103x6", feature = "stm32f103x8", feature = "stm32f103xB", feature = "stm32f103xC", feature = "stm32f103xD", feature = "stm32f103xE", feature = "stm32f103xF"))))]
macro_rules! memory_file {
    () => {
        include_bytes!("memory/stm32f103xG.x")
    }
}

pub fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(memory_file!())
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=memory.x");
}
