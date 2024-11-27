{ sources ? import ./npins }:
let
  system = builtins.currentSystem;
  pkgs = import sources.nixpkgs { config = { allowUnfree = true; }; overlays = [ ]; };
  # extensions = (import sources.nix-vscode-extensions).extensions.${system};
  rustcodium =
    let
      inherit (pkgs) vscode-with-extensions vscodium;
      rustExtensions = builtins.attrValues {
        inherit (pkgs.vscode-extensions.jnoortheen) nix-ide;
        inherit (pkgs.vscode-extensions.rust-lang) rust-analyzer;
        inherit (pkgs.vscode-extensions.vadimcn) vscode-lldb;
      };
    in
    (vscode-with-extensions.override {
      vscode = vscodium;
      vscodeExtensions = rustExtensions;
    });
in
pkgs.mkShell {
  packages = with pkgs; [
    rustcodium
  ];
  nativeBuildInputs = with pkgs; [
    cargo
    rust-analyzer
    rustc
    rustfmt
    rustPackages.clippy
    pkgs.rustPlatform.bindgenHook
    clippy
    lldb
  ];
  buildInputs = with pkgs; [
    pkg-config
    alsa-lib
    jack2
    faust
  ];
}
