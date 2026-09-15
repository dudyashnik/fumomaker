{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    pkg-config
    openssl
    rustup
    xorg.libX11
    xorg.libXi
    libGL
    alsa-lib
  ];
  shellHook = ''
    export PATH="$HOME/.cargo/bin:$PATH"
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.libGL pkgs.xorg.libX11 pkgs.xorg.libXi pkgs.libxkbcommon ]}:$LD_LIBRARY_PATH"
  '';
}
