{
  description = "DevShell for maturin + Rust + Python on NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
	  (python3.withPackages (ps: with ps; [ maturin pip setuptools wheel virtualenv]))
          rustc
          cargo
          clang
          llvmPackages.libclang
          zstd
        ];

        # Make libclang visible to bindgen/maturin
        shellHook = ''
          export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
          echo "🔧 DevShell ready. LIBCLANG_PATH set to $LIBCLANG_PATH"
        '';
      };
    };
}

