use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    // Run make in the src/c directory to compile the C code and generate libexercise.a
    let status = Command::new("make")
        .current_dir("src/c")
        .status()
        .expect("Failed to execute make");

    if !status.success() {
        panic!("Make command failed with status: {}", status);
    } else {
        println!("Make finished successfully! {}", status);
    }
    
    // Move libexercise.a from src/c to the root of the project
    let libexercise_path = Path::new("src/c/libexercise.a");
    let destination_path = Path::new("./libexercise.a");
    
    if libexercise_path.exists() {
        fs::rename(libexercise_path, destination_path)
            .expect("Failed to move libexercise.a to the main folder");
    } else {
        panic!("libexercise.a was not found in the src/c directory");
    }

    // Tell Cargo to link with the static libraries
    println!("cargo:rustc-link-lib=static=exercise");
    println!("cargo:rustc-link-lib=static=json");
    println!("cargo:rustc-link-lib=static=tokenise");

    if cfg!(target_os = "windows") {
        // Windows-specific linking
        println!("cargo:rustc-link-lib=static=msvcrt");
        println!("cargo:rustc-link-search=native=C:/ProgramData/mingw64/mingw64/x86_64-w64-mingw32/lib");
    } else {
        // Linux-specific linking
        println!("cargo:rustc-link-lib=dylib=c");  // Link with libc on Linux
        println!("cargo:rustc-link-search=native=/usr/local/lib"); // Adjust this path if necessary
    }

    // Specify the directory where the .a files are located
    println!("cargo:rustc-link-search=native=libs"); // Adjust path if necessary
    println!("cargo:rustc-link-search=native=./");
}
