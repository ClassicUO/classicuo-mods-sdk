#!/bin/sh
# Regenerate dotnet/Wit/ (the canonical-ABI glue for world cuo:modding/mod) from ../wit.
#
#   ./dotnet/regen-wit.sh            (needs wit-bindgen-cli on PATH: cargo install wit-bindgen-cli)
#
# The import bindings are used as generated. The EXPORTS are per mod (`setup` + one
# export per system, generated into the mod by ModSdk.targets / ModDescribe.cs, which
# also writes the mod's own world around Wit/ModWorld_component_type.wit), so
# wit-bindgen's export half is stripped from ModWorld.cs.
set -e
here=$(cd "$(dirname "$0")" && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
wit-bindgen c-sharp -r native-aot --internal -w cuo:modding/mod --out-dir "$tmp" "$here/../wit" >/dev/null
rm -rf "$here/Wit"
mkdir -p "$here/Wit"
for f in "$tmp"/*; do
  name=$(basename "$f" | sed -e 's/^ModWorld\.wit\.Imports\.//')
  cp "$f" "$here/Wit/$name"
done
mv "$here/Wit/Mod.cs" "$here/Wit/ModWorld.cs"
sed -i -e '/^    internal interface IModWorldExports{/,/^    }$/d' -e '/^    namespace Exports {/,/^    }$/d' "$here/Wit/ModWorld.cs"
echo "regenerated $here/Wit"
