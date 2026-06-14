#!/usr/bin/env bash
set -euo pipefail

CX_REPO="${CX_REPO:-junf108/cx}"
CX_VERSION="${CX_VERSION:-latest}"
CX_INSTALL_DIR="${CX_INSTALL_DIR:-/usr/local/bin}"

# --- helpers ----------------------------------------------------------------
info()  { printf "\033[1;34m>\033[0m %s\n" "$*"; }
error() { printf "\033[1;31m!\033[0m %s\n" "$*" >&2; }
ok()    { printf "\033[1;32m✓\033[0m %s\n" "$*"; }

cleanup() { rm -rf "$_tmpdir"; }
_tmpdir=""
trap cleanup EXIT

# --- detect platform --------------------------------------------------------
detect_target() {
  local _os _arch
  _os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  _arch="$(uname -m)"

  case "$_os" in
    darwin) _os="apple-darwin" ;;
    linux)
      # Use musl for aarch64 (fully static), glibc for x86_64
      if [ "$_arch" = "aarch64" ] || [ "$_arch" = "arm64" ]; then
        _os="unknown-linux-musl"
      else
        _os="unknown-linux-gnu"
      fi
      ;;
    mingw*|msys*|cygwin*) _os="pc-windows-msvc" ;;
    *)
      error "unsupported OS: $(uname -s)"
      exit 1
      ;;
  esac

  case "$_arch" in
    x86_64|amd64) _arch="x86_64" ;;
    aarch64|arm64) _arch="aarch64" ;;
    *)
      error "unsupported architecture: $(uname -m)"
      exit 1
      ;;
  esac

  printf "%s-%s\n" "$_arch" "$_os"
}

TARGET="$(detect_target)"
info "detected target: ${TARGET}"

# --- resolve download URL ---------------------------------------------------
resolve_url() {
  local _tag _url

  if [ "$CX_VERSION" = "latest" ]; then
    _tag="$(curl -fsSL "https://api.github.com/repos/${CX_REPO}/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": "//;s/".*//')"
    if [ -z "$_tag" ]; then
      error "could not determine latest release tag"
      exit 1
    fi
  else
    _tag="$CX_VERSION"
  fi

  _url="https://github.com/${CX_REPO}/releases/download/${_tag}/cx-${TARGET}.tar.gz"
  printf "%s\n" "$_url"
}

DOWNLOAD_URL="$(resolve_url)"
info "download url: ${DOWNLOAD_URL}"

# --- download & install -----------------------------------------------------
_tmpdir="$(mktemp -d)"

info "downloading cx..."
curl -fsSL "$DOWNLOAD_URL" -o "${_tmpdir}/cx.tar.gz"

info "verifying checksum..."
if [ -n "${CX_CHECKSUM:-}" ]; then
  echo "${CX_CHECKSUM}  ${_tmpdir}/cx.tar.gz" | sha256sum -c - >/dev/null 2>&1 || {
    error "checksum mismatch"
    exit 1
  }
  ok "checksum verified"
else
  # Try to download and verify checksum automatically
  if curl -fsSL "${DOWNLOAD_URL}.sha256" -o "${_tmpdir}/cx.tar.gz.sha256" 2>/dev/null; then
    (cd "$_tmpdir" && sha256sum -c cx.tar.gz.sha256) || {
      error "checksum verification failed"
      exit 1
    }
    ok "checksum verified"
  else
    info "no checksum file found, skipping verification"
  fi
fi

info "extracting..."
tar xzf "${_tmpdir}/cx.tar.gz" -C "$_tmpdir"

if [ ! -f "${_tmpdir}/cx" ] && [ ! -f "${_tmpdir}/cx.exe" ]; then
  error "binary not found in archive"
  ls -la "$_tmpdir"
  exit 1
fi

# Determine binary name
BINARY="cx"
[ -f "${_tmpdir}/cx.exe" ] && BINARY="cx.exe"

install -v "${_tmpdir}/${BINARY}" "${CX_INSTALL_DIR}/cx"

ok "cx installed to ${CX_INSTALL_DIR}/cx"

# --- verify ----------------------------------------------------------------
if command -v cx &>/dev/null; then
  CX_PATH="$(command -v cx)"
  ok "cx is ready at ${CX_PATH}"
else
  info "add ${CX_INSTALL_DIR} to your \$PATH if it is not already"
fi
