#!/bin/sh
# Regenerate dotnet/ModAbi.g.cs from abi/mod-abi.fbs.
#
#   FLATC=/path/to/flatc ./dotnet/regen-abi.sh
#
# flatc must match the Google.FlatBuffers package version in CuoModSdk.csproj (25.2.10).
# The object API (--gen-object-api) is used only for the setup-time tables; the per-tick
# inputs are read through the zero-copy struct accessors.
set -e
here=$(cd "$(dirname "$0")" && pwd)
flatc=${FLATC:-flatc}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
"$flatc" --csharp --gen-object-api --gen-onefile -o "$tmp" "$here/../abi/mod-abi.fbs"
cp "$tmp/mod-abi_generated.cs" "$here/ModAbi.g.cs"
echo "regenerated $here/ModAbi.g.cs"
