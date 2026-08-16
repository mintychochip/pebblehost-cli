#!/bin/sh
set -e

REPO="mintychochip/pebblehost-cli"

usage() {
  cat <<EOF
Usage: update.sh [OPTIONS]

Update the pb CLI to the latest GitHub release.

Options:
  -t, --tag <tag>          Install a specific release tag, e.g. v2026.8.15.3
  -v, --version <version>  Install a specific release version, e.g. 2026.8.15.3
  -h, --help               Show this help
EOF
}

err() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

arg_tag=""
arg_version=""

while [ $# -gt 0 ]; do
  case "$1" in
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

pb_path="$(command -v pb)" || err "pb not found in PATH"
prefix="$(dirname "$pb_path")"

echo "updating pb in ${prefix}..."

set -- --prefix "$prefix" --force
[ -n "$arg_tag" ] && set -- "$@" --tag "$arg_tag"
[ -n "$arg_version" ] && set -- "$@" --version "$arg_version"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

installer="${tmpdir}/install.sh"
curl -fsSL "https://raw.githubusercontent.com/${REPO}/master/scripts/install.sh" --output "$installer" || err "failed to download installer"
sh "$installer" "$@"
