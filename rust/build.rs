// Regenerate src/generated.rs from the canonical mod ABI schema on every build IF the
// `planus` (planus-cli 1.3.0) binary is on PATH, keeping the .fbs the single source of
// truth. When planus-cli is absent we fall back to the committed src/generated.rs so the
// crate still builds offline (best-effort — this mirrors tools/spikes/fb-compat/rust).
//
// planus 1.3.0 ships no first-party build-script codegen API (the `planus` crate is
// runtime-only); the supported codegen path is the `planus rust -o <out> <fbs>` CLI.
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    // ../abi/mod-abi.fbs is a snapshot of the canonical TinyEcs.Bevy.Modding/abi/mod-abi.fbs,
    // refreshed by the ClassicUO client's `make gen-mod-sdk`.
    let schema = PathBuf::from(manifest).join("..").join("abi").join("mod-abi.fbs");
    let out = PathBuf::from(manifest).join("src").join("generated.rs");

    println!("cargo:rerun-if-changed={}", schema.display());
    println!("cargo:rerun-if-changed=build.rs");

    match Command::new("planus")
        .arg("rust")
        .arg("-o")
        .arg(&out)
        .arg(&schema)
        .status()
    {
        Ok(s) if s.success() => {
            println!("cargo:warning=planus regenerated {}", out.display());
        }
        Ok(s) => panic!("planus codegen failed with status {s}"),
        Err(e) => {
            // planus-cli not on PATH — keep the committed generated.rs.
            println!("cargo:warning=planus-cli not found ({e}); using committed src/generated.rs");
        }
    }
}
