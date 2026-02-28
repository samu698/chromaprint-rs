use std::env;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LinkType {
    Static,
    Dynamic,
}

struct Target {
    unix: bool,
    os: String,
    env: String,
}

fn get_link_type(target: &Target) -> LinkType {
    let feature_static = env::var("CARGO_FEATURE_STATIC").is_ok();
    let feature_dynamic = env::var("CARGO_FEATURE_DYNAMIC").is_ok();

    match (feature_static, feature_dynamic) {
        (false, false) if !target.unix || target.env == "musl" || target.os == "macos" => LinkType::Static,
        (false, false) => LinkType::Dynamic,
        (true, _) => LinkType::Static,
        (false, true) => LinkType::Dynamic,
    }
}

fn build_vendored(link_type: LinkType) {
    let mut cmake = cmake::Config::new("src/chromaprint");

    let build_shared_libs = match link_type {
        LinkType::Dynamic => "ON",
        LinkType::Static => "OFF",
    };

    let install_dir = cmake.define("BUILD_TESTS", "OFF")
        .define("BUILD_SHARED_LIBS", build_shared_libs)
        .build();

    match link_type {
        LinkType::Dynamic => {
            let lib = install_dir.join("lib").into_os_string().into_string().unwrap();
            println!("cargo:rustc-link-search=native={lib}");
            println!("cargo:rustc-link-lib=chromaprint");
        }
        LinkType::Static => {
            println!("cargo:rustc-link-lib=static=chromaprint");
        }
    }
}

// Returns whether it found the library
fn find_system_lib(link_type: LinkType) -> bool {
    let found = pkg_config::Config::new()
        .statik(link_type == LinkType::Static)
        .probe("libchromaprint")
        .is_ok();
    if found { return true; }

    let found = vcpkg::Config::new()
        .find_package("libchromaprint")
        .is_ok();
    if found { return true; }
    
    false
}

fn generate_tests(include_path: &str) {
    let mut tgen = ctest::TestGenerator::new();

    tgen.include(include_path)
        .header("chromaprint.h")
        .language(ctest::Language::C)
        .edition(2024);

    tgen.skip_struct(|s| s.ident() == "ChromaprintAlgorithm");

    ctest::generate_test(&mut tgen, "src/lib.rs", "ctest.rs")
        .expect("Failed to generate tests");
}

fn main() {
    let target = Target {
        unix: env::var("CARGO_CFG_UNIX").is_ok(),
        os: env::var("CARGO_CFG_TARGET_OS").unwrap(),
        env: env::var("CARGO_CFG_TARGET_ENV").unwrap(),
    };

    let link_type = get_link_type(&target);

    let feature_vendored = env::var("CARGO_FEATURE_VENDORED").is_ok();
    let feature_system = env::var("CARGO_FEATURE_SYSTEM").is_ok();
    match (feature_vendored, feature_system) {
        (false, false) => {
            if find_system_lib(link_type) { return; }
            build_vendored(link_type);
        }
        (true, _) => build_vendored(link_type),
        (false, true) => {
            if find_system_lib(link_type) { return; }
            panic!("\
                Failed to find chromaprint library.\n\
                \n\
                Help: Check if the library is installed\n\
                Help: You can disable the system feature to compile the library directly\n\
            ")
        }
    }

    generate_tests("src/chromaprint/src/include/");
}
