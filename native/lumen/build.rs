use std::env;
use std::path::PathBuf;

fn main() {
    let sdl_include = env::var("LUMEN_SDL_INCLUDE").unwrap_or_else(|_| "/usr/include/SDL2".into());
    let webos = env::var_os("CARGO_FEATURE_WEBOS").is_some();
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let mut sources = vec!["src/abi_check.c", "src/text.c", "src/http.c", "src/local_time.c"];
    let mut c = cc::Build::new();
    c.include(&sdl_include).define("_REENTRANT", None).cargo_metadata(false);
    if webos {
        let sysroot = env::var("LUMEN_SYSROOT").expect("LUMEN_SYSROOT is the webOS SDK's sysroot");
        c.define("LUMEN_WEBOS", None);
        for include in ["usr/include/glib-2.0", "usr/lib/glib-2.0/include", "usr/include", "usr/include/freetype2"] {
            c.include(format!("{sysroot}/{include}"));
        }
        sources.extend(["src/webos_shell.c", "src/getauxval_missing_from_sdk_libc.c", "src/luna.c"]);
        let stood_in = cc::Build::new()
            .get_compiler()
            .to_command()
            .args(["-shared", "-nostdlib", "-Wl,-soname,libharfbuzz.so.0", "src/harfbuzz_missing_from_sdk.c", "-o"])
            .arg(out.join("libharfbuzz.so"))
            .status()
            .expect("the SDK compiler runs");
        assert!(stood_in.success(), "src/harfbuzz_missing_from_sdk.c did not link");
    } else {
        c.include("/usr/include/freetype2");
    }
    c.files(&sources).compile("lumen_c");
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static:+whole-archive=lumen_c");
    println!("cargo:rustc-link-lib=harfbuzz");
    println!("cargo:rustc-link-lib=freetype");
    println!("cargo:rustc-link-lib=curl");
    if webos {
        for library in ["wayland-webos-client", "wayland-client", "luna-service2", "glib-2.0"] {
            println!("cargo:rustc-link-lib={library}");
        }
        println!("cargo:rerun-if-changed=src/harfbuzz_missing_from_sdk.c");
    }
    for source in sources {
        println!("cargo:rerun-if-changed={source}");
    }
    println!("cargo:rerun-if-env-changed=LUMEN_SDL_INCLUDE");
    println!("cargo:rerun-if-env-changed=LUMEN_SYSROOT");
}
