{
  description = "shell for building colonq";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, naersk }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      naersk' = pkgs.callPackage naersk {};
      mpv-unwrapped = pkgs.mpv-unwrapped.override {
        ffmpeg_5 = pkgs.ffmpeg_5-full;
      };
      buildInputs = [
        pkgs.pkgconfig
        pkgs.freetype
        pkgs.SDL2
        pkgs.SDL2_image
        pkgs.SDL2_mixer
        mpv-unwrapped
        pkgs.llvm
        pkgs.clang
        pkgs.llvmPackages.libclang
        pkgs.openssl
      ];
      shell = pkgs.mkShell {
        inherit buildInputs;
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };
      colonq = naersk'.buildPackage {
        inherit buildInputs;
        src = ./.;
      };
    in {
      defaultPackage.x86_64-linux = colonq;
      devShell.x86_64-linux = shell;
      packages.x86_64-linux = {
        inherit colonq shell;
      };
    };
}
