fn main() {
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=comdlg32");
    println!("cargo:rustc-link-lib=comctl32");
    println!("cargo:rustc-link-lib=shell32");

    let no_macros: &[&str] = &[];
    embed_resource::compile("retropad.rc", no_macros);
}
