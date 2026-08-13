#!/bin/sh
set -eu

REPO="electiveDev/weathr"

require_commands() {
    for cmd in curl mktemp sed grep uname head; do
        command -v "$cmd" >/dev/null 2>&1 || {
            echo "Error: required command '$cmd' not found" >&2
            exit 1
        }
    done
}

detect_os() {
    os="$(uname -s)"

    case "$os" in
        Linux*)   echo "linux" ;;
        Darwin*)  echo "macos" ;;
        FreeBSD*) echo "freebsd" ;;
        *)
            echo "Error: unsupported OS: $os" >&2
            exit 1
            ;;
    esac
}

detect_arch() {
    arch="$(uname -m)"

    case "$arch" in
        x86_64|amd64) echo "amd64" ;;
        aarch64|arm64) echo "arm64" ;;
        *)
            echo "Error: unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac
}

detect_libc() {
    os="$1"

    if [ "$os" != "linux" ]; then
        echo ""
        return 0
    fi

    if command -v ldd >/dev/null 2>&1 && ldd /bin/sh 2>&1 | grep -q musl; then
        echo "-musl"
    else
        echo ""
    fi
}

get_latest_tag() {
    curl -fsSL -I -o /dev/null -w '%{url_effective}' \
        "https://github.com/${REPO}/releases/latest" \
        | sed 's#.*/##'
}

build_binary_name() {
    os="$1"
    arch="$2"
    libc="$3"

    if [ -z "$libc" ]; then
        echo "weathr-${os}-${arch}"
    else
        echo "weathr-${os}${libc}-${arch}"
    fi
}

download_binary() {
    url="$1"
    output="$2"

    if ! curl -fSL --retry 3 --retry-delay 1 "$url" -o "$output"; then
        echo "Error: failed to download binary" >&2
        exit 1
    fi

    if [ ! -s "$output" ]; then
        echo "Error: download incomplete or empty" >&2
        exit 1
    fi
}

install_binary() {
    src="$1"

    install_dir="$HOME/.local/bin"
    if [ "$(id -u)" -eq 0 ]; then
        install_dir="/usr/local/bin"
    fi

    mkdir -p "$install_dir"
    mv "$src" "$install_dir/weathr"

    echo "✓ weathr installed to $install_dir/weathr"

    case ":$PATH:" in
        *":$install_dir:"*|*":$install_dir/:"*) ;;
        *)
            echo ""
            echo "Note: $install_dir is not in your PATH"
            echo "Add this to your shell config:"
            echo "export PATH=\"\$PATH:$install_dir\""
            ;;
    esac
}

main() {
    echo "Installing weathr..."

    require_commands

    OS="$(detect_os)"
    ARCH="$(detect_arch)"
    LIBC="$(detect_libc "$OS")"

    if [ "$OS" = "freebsd" ] && [ "$ARCH" != "amd64" ]; then
        echo "Error: FreeBSD is only supported on x86_64" >&2
        exit 1
    fi

    BINARY_NAME="$(build_binary_name "$OS" "$ARCH" "$LIBC")"
    LATEST_TAG="$(get_latest_tag)"

    if [ -z "$LATEST_TAG" ] || [ "$LATEST_TAG" = "null" ]; then
        echo "Error: could not determine latest release" >&2
        exit 1
    fi

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${BINARY_NAME}"

    if [ -z "$LIBC" ]; then
        echo "Detected platform: $OS $ARCH"
    else
        echo "Detected platform: $OS $ARCH $LIBC"
    fi
    echo "Latest release: $LATEST_TAG"
    echo "Downloading ${BINARY_NAME}..."

    TMP_DIR="$(mktemp -d)"
    trap 'rm -rf -- "$TMP_DIR"' EXIT

    download_binary "$DOWNLOAD_URL" "$TMP_DIR/weathr"

    chmod +x "$TMP_DIR/weathr"

    install_binary "$TMP_DIR/weathr"
}

main "$@"
