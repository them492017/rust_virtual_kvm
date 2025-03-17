{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") {} }:

let
  sessionType = builtins.getEnv "XDG_SESSION_TYPE";
  isX11 = sessionType == "x11";
in
pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.pkg-config
  ] ++ (if isX11 then [
    pkgs.xorg.libX11
    pkgs.xorg.libXtst
  ] else []);

  shellHook = ''
    if [ "$XDG_SESSION_TYPE" != "x11" ]; then
      echo "Only X11 backends are currently supported" >&2
      exit 1
    fi

    export PKG_CONFIG_PATH="${pkgs.xorg.libX11}/lib/pkgconfig:${pkgs.xorg.libXtst}/lib/pkgconfig"
  '';
}
