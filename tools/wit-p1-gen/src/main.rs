//! wit-p1-gen — reads wit/cuo-mod.wit and writes the ClassicUO host's C# signature
//! table (csharp.rs): the p2 component bridge's view of the `host` / `packets` /
//! `assets` / `actions` imports and the world's own exports.
//!
//! Usage (from the SDK root): cargo run --manifest-path tools/wit-p1-gen/Cargo.toml -- wit <csharp-signature-file>
//! [<hostcalls-file> <hostcalls-cm-file>] (the repo root's `make gen-mod-wit-sig`). The
//! optional pair is the typed host-call dispatch (hostcalls.rs). The `wit-cs-gen` bin
//! writes the C# SDK's dotnet/Cuo.g.cs.

mod csharp;
mod hostcalls;

use anyhow::{bail, Context, Result};
use wit_parser::Resolve;

const INTERFACES: &[&str] = &["host", "packets", "assets", "actions"];

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 && args.len() != 5 {
        bail!("usage: wit-p1-gen <wit-dir> <csharp-signature-file> [<hostcalls-file> <hostcalls-cm-file>]");
    }
    let mut resolve = Resolve::default();
    let (pkg, _) = resolve.push_dir(&args[1]).context("parsing WIT")?;
    std::fs::write(&args[2], csharp::emit(&resolve, pkg, INTERFACES, "mod", &["on-packet"])?)?;
    if args.len() == 5 {
        let (shared, cm) = hostcalls::emit(&resolve, pkg, INTERFACES)?;
        std::fs::write(&args[3], shared)?;
        std::fs::write(&args[4], cm)?;
    }
    Ok(())
}
