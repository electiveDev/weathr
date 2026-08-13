#!/bin/sh
set -eu

if [ "$#" -ne 5 ]; then
    echo "usage: $0 <version> <arch> <binary-path> <output-dir> <packagers>" >&2
    exit 1
fi

VERSION="$1"
ARCH="$2"
BINARY_PATH="$3"
OUTPUT_DIR="$4"
PACKAGERS="$5"
PACKAGE_ROOT="target/package-root"
MANPAGE_DIR="$PACKAGE_ROOT/usr/share/man/man1"
BINARY_DIR="$PACKAGE_ROOT/usr/bin"
DOC_DIR="$PACKAGE_ROOT/usr/share/doc/weathr"
NFPMSPEC="target/nfpm.yaml"

rm -rf "$PACKAGE_ROOT"
mkdir -p "$BINARY_DIR" "$MANPAGE_DIR" "$DOC_DIR" "$OUTPUT_DIR"

install -m755 "$BINARY_PATH" "$BINARY_DIR/weathr"
cargo run --locked --release --bin generate-manpage -- "$MANPAGE_DIR/weathr.1"
gzip -9f "$MANPAGE_DIR/weathr.1"
install -m644 README.md "$DOC_DIR/README.md"
install -m644 LICENSE "$DOC_DIR/LICENSE"

cat > "$NFPMSPEC" <<EOF
name: weathr
arch: ${ARCH}
platform: linux
version: ${VERSION}
release: 1
section: utils
priority: optional
maintainer: Dony Mulya <veirt@duck.com>
description: |
  Terminal-based ASCII weather application with animated scenes driven by real-time weather data.
homepage: https://github.com/electiveDev/weathr
license: GPL-3.0-or-later
contents:
  - src: ${PACKAGE_ROOT}/usr/bin/weathr
    dst: /usr/bin/weathr
  - src: ${PACKAGE_ROOT}/usr/share/man/man1/weathr.1.gz
    dst: /usr/share/man/man1/weathr.1.gz
  - src: README.md
    dst: /usr/share/doc/weathr/README.md
  - src: LICENSE
    dst: /usr/share/doc/weathr/LICENSE
rpm:
  group: Applications/Utilities
EOF

OLD_IFS="$IFS"
IFS=,
set -- $PACKAGERS
IFS="$OLD_IFS"

for packager in "$@"; do
    nfpm package --config "$NFPMSPEC" --target "$OUTPUT_DIR" --packager "$packager"
done
