fn generate(header: &str, file: &str, matching: &str, cplusplus: bool) {
    use std::path::PathBuf;
    let src_path = PathBuf::from("src/");
    let mut builder = bindgen::Builder::default();
    if cplusplus {
        builder = builder.clang_args(&["-x", "c++"]);
    }
    let bindings = builder
        .use_core()
        .ctypes_prefix("libc")
        // The input header we would like to generate
        // bindings for.
        .header(header)
        .whitelist_type(matching)
        .whitelist_function(matching)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings.write_to_file(src_path.join(file))
        .expect("Couldn't write bindings!");
}

fn compile(cfg: &mut cmake::Config, name: &str) {
    let target = if cfg!(feature = "plain-c") {
        cfg.define("MP_USE_C", "ON");
        name.to_string()
    } else {
        let mut tmp = name.to_string();
        tmp.push('x');
        tmp
    };

    let mut mprompt = cfg.build_target(name).build();
    mprompt.push("./build");
    println!("cargo:rustc-link-lib={}", target);

    if cfg!(all(windows, target_env = "msvc")) {
        println!(
            "cargo:rustc-link-search=native={}/{}",
            mprompt.display(),
            cfg.get_profile()
        );
    } else {
        println!("cargo:rustc-link-search=native={}", mprompt.display());
    }
}

fn main() {
    println!("cargo:rerun-if-changed=mpeff_wrapper.h");

    if cfg!(feature = "plain-c") {
        if cfg!(feature = "mpeff") {
            generate("mpeff_wrapper.h", "mpeff.rs", "mpe_.*", false);
        }
        generate("libmprompt/include/mprompt.h", "mprompt.rs", "mp_.*", false);
    } else {
        if cfg!(feature = "mpeff") {
            generate("mpeff_wrapper.h", "mpeff.rs", "mpe_.*", true);
        }
        generate("libmprompt/include/mprompt.h", "mprompt.rs", "mp_.*", true);
    }

    let profile = if cfg!(feature = "debug") {
        "Debug"
    } else {
        "Release"
    };

    let cfg = &mut cmake::Config::new("libmprompt");

    cfg.profile(profile)
        .very_verbose(true);

    // mprompt
    compile(cfg, "mprompt");
    if cfg!(feature = "mpeff") {
        compile(cfg, "mpeff");
    }

    if cfg!(not(feature = "plain-c")) {
        if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=dylib=c++");
        }

        if cfg!(target_os = "openbsd") {
            println!("cargo:rustc-link-lib=dylib=c++");
        }

        if cfg!(target_os = "freebsd") {
            println!("cargo:rustc-link-lib=dylib=c++");
        }

        if cfg!(target_os = "linux") {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }

        if cfg!(all(windows, target_env = "gnu")) {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
    }
}