use std::env;

fn main() {
    let sdl_include = env::var("LUMEN_SDL_INCLUDE").unwrap_or_else(|_| "/usr/include/SDL2".into());
    let webos = env::var_os("CARGO_FEATURE_WEBOS").is_some();
    let mut sources = vec!["src/abi_check.c"];
    let mut c = cc::Build::new();
    c.include(&sdl_include).define("_REENTRANT", None).cargo_metadata(false);
    if webos {
        let sysroot = env::var("LUMEN_SYSROOT").expect("LUMEN_SYSROOT is the webOS SDK's sysroot");
        c.define("LUMEN_WEBOS", None).include(format!("{sysroot}/usr/include/glib-2.0")).include(format!("{sysroot}/usr/lib/glib-2.0/include")).include(format!("{sysroot}/usr/include"));
        sources.extend(["src/webos_shell.c", "src/getauxval_missing_from_sdk_libc.c", "src/luna.c"]);
    }
    c.files(&sources).compile("lumen_c");
    println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").expect("OUT_DIR"));
    println!("cargo:rustc-link-lib=static:+whole-archive=lumen_c");
    if webos {
        println!("cargo:rustc-link-lib=wayland-webos-client");
        println!("cargo:rustc-link-lib=wayland-client");
        println!("cargo:rustc-link-lib=luna-service2");
        println!("cargo:rustc-link-lib=glib-2.0");
    }
    for source in sources {
        println!("cargo:rerun-if-changed={source}");
    }
    println!("cargo:rerun-if-env-changed=LUMEN_SDL_INCLUDE");
}
