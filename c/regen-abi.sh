#!/bin/sh
# Regenerate c/generated/ (flatcc reader + builder headers) from abi/mod-abi.fbs.
#
#   FLATCC=/path/to/flatcc ./c/regen-abi.sh
#
# flatcc is the v0.6.1 compiler (https://github.com/dvidelabs/flatcc, `cmake` +
# `make flatcc_cli`, binary lands in bin/). The runtime sources vendored under
# third_party/flatcc are from the same tag — keep the two in step.
#
# Two quirks of feeding flatcc this schema, both handled here:
# - flatcc rejects non-ASCII bytes even inside comments (the schema has em-dashes).
# - the header guard is derived from the file name, and `mod-abi` would make it
#   MOD-ABI_READER_H (not an identifier) — so the copy is named mod_abi.fbs.
set -e
here=$(cd "$(dirname "$0")" && pwd)
flatcc=${FLATCC:-flatcc}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
LC_ALL=C sed 's/[\x80-\xFF]/-/g' "$here/../abi/mod-abi.fbs" > "$tmp/mod_abi.fbs"
"$flatcc" -a -o "$tmp" "$tmp/mod_abi.fbs"
for f in flatbuffers_common_reader.h flatbuffers_common_builder.h mod_abi_reader.h mod_abi_builder.h; do
    cp "$tmp/$f" "$here/generated/$f"
done
echo "regenerated $here/generated"
