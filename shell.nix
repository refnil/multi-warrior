{ 
sources ? import ./nix/sources.nix, 
pkgs ? import sources.nixpkgs { config.allowUnfree = true; overlays = [ mozilla-overlay ]; },
mozilla-overlay ? import sources.nixpkgs-mozilla,
nixGL ? import sources.nixGL { inherit pkgs; }
}:
let 
  nightly-rust = pkgs.latest.rustChannels.nightly;
  rust = nightly-rust.rust.override {
    extensions = [
      "rust-src"
      "rls-preview"
      "clippy-preview"
      "rustfmt-preview"
      # "rust-analysis"
      "rls-preview"
    ];
  };

  wrap = name: pkg: pkgs.stdenv.mkDerivation {
    name = "steam-run-${name}";
    version = "dontcare";

    phases = [ "buildPhase" ];
    nativeBuildInputs = [ pkgs.makeWrapper ];
    buildCommand = ''
      mkdir -p $out/bin
      makeWrapper ${pkgs.steam-run}/bin/steam-run $out/bin/${name} --add-flags ${pkg}/bin/${name}
    '';
  };

  wrapCargo = wrap "cargo" rust;
in pkgs.mkShell {
  buildInputs = [
    wrapCargo
    rust
    pkgs.alsaLib
    pkgs.pkgconfig
    pkgs.udev
    pkgs.vulkan-headers
    pkgs.vulkan-loader
    pkgs.vulkan-tools
    pkgs.xlibsWrapper
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    pkgs.lld
    pkgs.clang
    pkgs.cargo-watch
    pkgs.cargo-expand
    pkgs.cargo-cache
    pkgs.cargo-bloat
    pkgs.aseprite
    pkgs.peek
  ];
}
