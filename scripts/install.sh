#!/bin/sh
set -e

REPO="mintychochip/pebblehost-cli"
DEFAULT_PREFIX="${HOME}/.local/bin"
ALT_PREFIX="/usr/local/bin"

usage() {
  cat <<EOF
Usage: install.sh [OPTIONS]

Install the pb CLI from GitHub releases.

Options:
  -p, --prefix <dir>       Install directory (default: first writable of ~/.local/bin, /usr/local/bin)
  -t, --tag <tag>          Release tag, e.g. v2026.8.15.3
  -v, --version <version>  Release version, e.g. 2026.8.15.3
  -f, --force              Overwrite existing binary
  -s, --skip-version-check Do not run pb --version after install
  -h, --help               Show this help

Examples:
  curl -sSL https://raw.githubusercontent.com/${REPO}/master/scripts/install.sh | sh
  curl -sSL https://raw.githubusercontent.com/${REPO}/master/scripts/install.sh | sh -s -- --prefix /usr/local/bin
EOF
}

err() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

info() {
  printf '%s\n' "$1"
}

# Parse arguments
prefix=""
force=0
skip_version_check=0
arg_tag=""
arg_version=""

while [ $# -gt 0 ]; do
  case "$1" in
    -p|--prefix)
      [ -n "$2" ] || err "missing argument for $1"
      prefix="$2"
      shift 2
      ;;
    -t|--tag)
      [ -n "$2" ] || err "missing argument for $1"
      arg_tag="$2"
      shift 2
      ;;
    -v|--version)
      [ -n "$2" ] || err "missing argument for $1"
      arg_version="$2"
      shift 2
      ;;
    -f|--force)
      force=1
      shift
      ;;
    -s|--skip-version-check)
      skip_version_check=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      break
      ;;
    -*)
      err "unknown option: $1"
      ;;
    *)
      err "unexpected argument: $1"
      ;;
  esac
done

# Resolve target triple
os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux)
    case "$arch" in
      x86_64) target="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) target="aarch64-unknown-linux-gnu" ;;
      armv7l|armv7) target="armv7-unknown-linux-gnueabihf" ;;
      *) err "unsupported architecture on Linux: $arch" ;;
    esac
    ;;
  Darwin)
    case "$arch" in
      x86_64) target="x86_64-apple-darwin" ;;
      aarch64|arm64) target="aarch64-apple-darwin" ;;
      *) err "unsupported architecture on macOS: $arch" ;;
    esac
    ;;
  *)
    err "unsupported operating system: $os"
    ;;
esac

# Resolve release tag and version
if [ -n "$arg_tag" ]; then
  tag="$arg_tag"
  version="${tag#v}"
  [ "$tag" != "$version" ] || err "tag must start with 'v', e.g. v2026.8.15.3"
elif [ -n "$arg_version" ]; then
  version="$arg_version"
  tag="v${version}"
else
  info "resolving latest release..."
  latest_url="https://api.github.com/repos/${REPO}/releases/latest"
  tag="$(curl -fsSL "$latest_url" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)"
  [ -n "$tag" ] || err "failed to resolve latest release from ${latest_url}"
  version="${tag#v}"
fi

asset="pebblehost-cli-${version}-${target}.tar.gz"
download_url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

info "installing pb ${version} for ${target}"

# Resolve install prefix
if [ -z "$prefix" ]; then
  if [ -w "$DEFAULT_PREFIX" ] 2>/dev/null || mkdir -p "$DEFAULT_PREFIX" 2>/dev/null; then
    if [ -w "$DEFAULT_PREFIX" ]; then
      prefix="$DEFAULT_PREFIX"
    fi
  fi

  if [ -z "$prefix" ]; then
    if [ -w "$ALT_PREFIX" ]; then
      prefix="$ALT_PREFIX"
    fi
  fi

  if [ -z "$prefix" ]; then
    err "could not find a writable install directory. Use --prefix or run with sudo to install to ${ALT_PREFIX}"
  fi
fi

mkdir -p "$prefix"
[ -d "$prefix" ] || err "install directory does not exist: $prefix"

# Prepare temp directory
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

# Download
info "downloading ${download_url}"
curl -fsSL --output "${tmpdir}/${asset}" "$download_url" || err "download failed: ${download_url}"

# Extract
info "extracting..."
tar -xzf "${tmpdir}/${asset}" -C "$tmpdir" || err "failed to extract ${asset}"

# Locate extracted binary
if [ -f "${tmpdir}/pb" ]; then
  binary="${tmpdir}/pb"
elif [ -f "${tmpdir}/pb.exe" ]; then
  binary="${tmpdir}/pb.exe"
else
  err "could not find pb binary in the extracted archive"
fi

# Install
dest="${prefix}/$(basename "$binary" .exe)"
if [ -e "$dest" ] && [ "$force" -ne 1 ]; then
  err "binary already exists at ${dest}; use --force to overwrite"
fi
cp -f "$binary" "$dest" || err "failed to copy binary to ${dest}"
chmod +x "$dest" || err "failed to make binary executable"

info "installed: ${dest}"

# Verify
if [ "$skip_version_check" -ne 1 ]; then
  if command -v pb >/dev/null 2>&1 && pb --version >/dev/null 2>&1; then
    installed_version="$(pb --version 2>&1)"
    info "verified: ${installed_version}"
  elif "$dest" --version >/dev/null 2>&1; then
    installed_version="$("$dest" --version 2>&1)"
    info "verified: ${installed_version}"
  else
    info "install complete; add ${prefix} to your PATH to run pb"
  fi
fi
