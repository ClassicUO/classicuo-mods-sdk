//! wit-p1-gen — reads wit/cuo-mod.wit and writes the ClassicUO host's C# signature
//! table (csharp.rs): the p2 component bridge's view of the `host` / `packets` /
//! `assets` / `actions` imports and the world's own exports.
//!
//! Usage (from the SDK root): cargo run --manifest-path tools/wit-p1-gen/Cargo.toml -- wit <csharp-signature-file>
//! (the repo root's `make gen-mod-wit-sig`). The `wit-cs-gen` bin writes the C# SDK's
//! dotnet/Cuo.g.cs.

mod csharp;

use anyhow::{bail, Context, Result};
use wit_parser::Resolve;

const INTERFACES: &[&str] = &["host", "packets", "assets", "actions"];

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        bail!("usage: wit-p1-gen <wit-dir> <csharp-signature-file>");
    }
    let mut resolve = Resolve::default();
    let (pkg, _) = resolve.push_dir(&args[1]).context("parsing WIT")?;
    std::fs::write(&args[2], csharp::emit(&resolve, pkg, INTERFACES, "mod", &["on-packet"])?)?;
    Ok(())
}
