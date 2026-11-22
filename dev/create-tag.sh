#!/bin/sh

# usage:
# ./create-tag.sh -v 0.1.5 -m "Kyanite 0.1.5 (changes)"

set -eu

usage() {
  cat >&2 <<EOF
Usage: $0 -v|--version X.Y.Z -m|--message "message"

  -v, --version   Version (with or without leading 'v'). Required, only once.
  -m, --message   Tag message. Required, only once.
  -h, --help      Show this help.

Example:
  $0 -v 0.1.5 -m "Kyanite 0.1.5 (fix bashrc alias)"
EOF
}

die() {
  echo "Error: $*" >&2
  usage
  exit 1
}

version=""
message=""
have_v=0
have_m=0

while [ $# -gt 0 ]; do
  case "$1" in
    -v|--version)
      [ "$have_v" -eq 0 ] || die "-v/--version given more than once"
      [ $# -ge 2 ] || die "missing value for $1"
      version=$2; have_v=1; shift 2 ;;
    --version=*)
      [ "$have_v" -eq 0 ] || die "-v/--version given more than once"
      version=${1#*=}; have_v=1; shift ;;
    -m|--message)
      [ "$have_m" -eq 0 ] || die "-m/--message given more than once"
      [ $# -ge 2 ] || die "missing value for $1"
      message=$2; have_m=1; shift 2 ;;
    --message=*)
      [ "$have_m" -eq 0 ] || die "-m/--message given more than once"
      message=${1#*=}; have_m=1; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      die "unknown argument: $1" ;;
  esac
done

[ "$have_v" -eq 1 ] && [ -n "$version" ] || die "missing -v/--version"
[ "$have_m" -eq 1 ] && [ -n "$message" ] || die "missing -m/--message"

# Accept "0.1.5" or "v0.1.5"
version=${version#v}

# Only vX.Y.Z tags trigger the release workflow
printf '%s' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' \
  || die "version must be in X.Y.Z format (got: $version)"

tag="v$version"

echo "Creating tag $tag: $message"
git tag -a "$tag" -m "$message"
git push origin "$tag"
