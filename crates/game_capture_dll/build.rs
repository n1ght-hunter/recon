extern crate cc;
use std::env;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();

    let parts = target.splitn(4, '-').collect::<Vec<_>>();
    let arch = parts[0];

    let hde = match arch {
        "i686" => "hde/hde32.c",
        "x86_64" => "hde/hde64.c",
        _ => panic!("Architecture '{arch}' not supported."),
    };

    let minhook_hash = get_minhook_hash();

    println!("cargo:rerun-if-changed={}", minhook_hash);
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );

    download_minhook_files();

    let mh_src_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("minhook/src");

    cc::Build::new()
    .file(mh_src_dir.join("buffer.c"))
    .file(mh_src_dir.join("hook.c"))
    .file(mh_src_dir.join("trampoline.c"))
    .file(mh_src_dir.join(hde))
    .compile("libminhook.a");

    #[cfg(feature = "opengl3")]
    {
        use std::fs::File;

        use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

        let dest = env::var("OUT_DIR").unwrap();
        let mut file = File::create(Path::new(&dest).join("gl_bindings.rs")).unwrap();

        Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::All, [])
            .write_bindings(StructGenerator, &mut file)
            .unwrap();
    }
}

fn get_minhook_hash() -> String {
    let res = ureq::get("https://api.github.com/repos/TsudaKageyu/minhook/branches/master")
        .call()
        .unwrap()
        .into_json::<serde_json::Value>()
        .unwrap();
    res["commit"]["sha"].as_str().unwrap().to_string()
}

const BASE_URL: &str = "https://raw.githubusercontent.com/TsudaKageyu/minhook/refs/heads/master/";

fn download_minhook_file(file: &str) {
    let url = format!("{}{}", BASE_URL, file);
    let res = ureq::get(&url).call().unwrap().into_string().unwrap();

    let path = format!("{}/minhook/{}", env::var("OUT_DIR").unwrap(), file);
    std::fs::write(path, res).unwrap();
}

fn create_dir(dir: &str) {
    let path = format!("{}/{}", env::var("OUT_DIR").unwrap(), dir);
    std::fs::create_dir_all(path).unwrap();
}

fn download_minhook_files() {
    create_dir("minhook");
    create_dir("minhook/src");
    create_dir("minhook/src/hde");
    create_dir("minhook/include");


    download_minhook_file("src/buffer.c");
    download_minhook_file("src/buffer.h");
    download_minhook_file("src/hook.c");
    download_minhook_file("src/trampoline.c");
    download_minhook_file("src/trampoline.h");
    download_minhook_file("src/hde/hde32.c");
    download_minhook_file("src/hde/hde32.h");
    download_minhook_file("src/hde/hde64.c");
    download_minhook_file("src/hde/hde64.h");
    download_minhook_file("src/hde/pstdint.h");
    download_minhook_file("src/hde/table64.h");
    download_minhook_file("src/hde/table32.h");
    download_minhook_file("include/MinHook.h");
}
