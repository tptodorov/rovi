{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = [
    pkgs.espflash
    pkgs.go-task
    pkgs.rustup
  ];

}
