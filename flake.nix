{
  description = "html5-picture development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      devShellName = "html5-picture";
    in
    {
      devShells."x86_64-linux".default =
        let
          pkgs = import nixpkgs {
            system = "x86_64-linux";
          };
        in
        pkgs.mkShell {
          name = devShellName;
          packages = with pkgs; [
          ];
        };
      devShells."aarch64-darwin".default =
        let
          pkgs = import nixpkgs {
            system = "aarch64-darwin";
          };
        in
        pkgs.mkShell {
          name = devShellName;
          packages = with pkgs; [
          ];
          buildInputs = with pkgs; [
            libiconv
            openssl
          ];
        };
    };
}
