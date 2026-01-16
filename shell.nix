{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixpkgs-unstable.tar.gz") {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    clippy
    rustfmt

    # Build tools
    pkg-config
    cmake
  ];

  buildInputs = with pkgs; [
    # Audio
    alsa-lib

    # OpenSSL
    openssl

    # GTK and friends
    gtk3
    glib
    cairo
    pango
    gdk-pixbuf
    atk

    # X11
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    libxkbcommon
    xdotool

    # For bindgen
    llvmPackages.libclang
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

  shellHook = ''
    echo "whisp development shell"
    echo "Rust: $(rustc --version)"
  '';
}
