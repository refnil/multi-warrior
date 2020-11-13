{ 
sources ? import ./nix/sources.nix, 
mozilla-overlay ? import sources.nixpkgs-mozilla,
pkgs ? import sources.nixpkgs { config.allowUnfree = true; overlays = [ mozilla-overlay ]; },
nixGL ? import sources.nixGL { inherit pkgs; }
}:
let 
  nightly-rust = pkgs.latest.rustChannels.nightly;
  rust = nightly-rust.rustc;
  cargo = pkgs.cargo;

  steam-run = pkgs.steam-run;
  wrap = name: pkg: pkgs.stdenv.mkDerivation {
    name = "steam-run-${name}";
    version = "dontcare";

    phases = [ "buildPhase" ];
    nativeBuildInputs = [ pkgs.makeWrapper ];
    buildCommand = ''
      mkdir -p $out/bin
      makeWrapper ${steam-run}/bin/steam-run $out/bin/${name} --add-flags ${pkg}/bin/${name}
    '';
  };
  wrapCargo = wrap "cargo" cargo;
  wrapCoz = wrap "coz" pkgs.coz;
in pkgs.mkShell {
  buildInputs = [
    wrapCargo
    pkgs.alsaLib
    pkgs.pkgconfig
    pkgs.libudev
    pkgs.vulkan-headers
    pkgs.vulkan-loader
    pkgs.vulkan-tools
    pkgs.x11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    #nixGL.nixVulkanNvidia
    pkgs.lld
    pkgs.cargo-watch
    pkgs.aseprite
    wrapCoz
  ];
}
