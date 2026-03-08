#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

log() {
  echo "[setup] $1"
}

warn() {
  echo "[setup] WARNING: $1"
}

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

run_privileged() {
  if has_cmd sudo; then
    sudo "$@"
    return $?
  fi

  if [[ "${EUID}" -eq 0 ]]; then
    "$@"
    return $?
  fi

  return 1
}

install_system_packages() {
  if has_cmd apt-get; then
    log "Installing Linux dependencies with apt-get (Vulkan + native build deps)..."
    local packages=(
      build-essential
      pkg-config
      curl
      ca-certificates
      libx11-dev
      libxi-dev
      libxrandr-dev
      libxcursor-dev
      libxinerama-dev
      libasound2-dev
      libudev-dev
      libwayland-dev
      libxkbcommon-dev
      libvulkan-dev
      vulkan-tools
    )

    if run_privileged apt-get update && run_privileged apt-get install -y "${packages[@]}"; then
      return
    fi

    warn "Could not install apt packages automatically (missing privilege escalation)."
    warn "Please install required packages manually, then re-run this script."
    return
  fi

  if has_cmd dnf; then
    log "Installing Linux dependencies with dnf (Vulkan + native build deps)..."
    local packages=(
      gcc
      gcc-c++
      make
      pkgconf-pkg-config
      curl
      ca-certificates
      libX11-devel
      libXi-devel
      libXrandr-devel
      libXcursor-devel
      libXinerama-devel
      alsa-lib-devel
      systemd-devel
      wayland-devel
      libxkbcommon-devel
      vulkan-loader-devel
      vulkan-tools
    )

    if run_privileged dnf install -y "${packages[@]}"; then
      return
    fi

    warn "Could not install dnf packages automatically (missing privilege escalation)."
    warn "Please install required packages manually, then re-run this script."
    return
  fi

  if has_cmd pacman; then
    log "Installing Linux dependencies with pacman (Vulkan + native build deps)..."
    local packages=(
      base-devel
      pkgconf
      curl
      ca-certificates
      libx11
      libxi
      libxrandr
      libxcursor
      libxinerama
      alsa-lib
      systemd
      wayland
      libxkbcommon
      vulkan-icd-loader
      vulkan-tools
    )

    if run_privileged pacman -Sy --needed --noconfirm "${packages[@]}"; then
      return
    fi

    warn "Could not install pacman packages automatically (missing privilege escalation)."
    warn "Please install required packages manually, then re-run this script."
    return
  fi

  warn "No supported package manager found (apt-get/dnf/pacman)."
  warn "Install Vulkan and native build dependencies manually for your distro."
}

install_rust_toolchain() {
  if ! has_cmd rustup; then
    log "Rustup not found. Installing rustup + Rust stable toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  fi

  if [[ -f "${HOME}/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    source "${HOME}/.cargo/env"
  fi

  if ! has_cmd rustup || ! has_cmd cargo; then
    echo "[setup] ERROR: rustup/cargo are still unavailable after installation attempt."
    echo "[setup] Add ${HOME}/.cargo/bin to PATH and re-run this script."
    exit 1
  fi

  log "Ensuring stable Rust toolchain and useful components are installed..."
  rustup toolchain install stable
  rustup default stable
  rustup component add rustfmt clippy
}

prepare_project() {
  cd "${PROJECT_ROOT}"

  if [[ ! -f Cargo.toml ]]; then
    log "Cargo.toml not found. Initializing Rust binary crate in project root..."
    cargo init --name openfire --vcs none .
  fi

  log "Fetching Rust crate dependencies..."
  cargo fetch
}

log "Starting OpenFire development environment setup..."
install_system_packages
install_rust_toolchain
prepare_project
log "Setup complete. You can now run: cargo run"