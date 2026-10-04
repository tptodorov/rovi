{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = [
    pkgs.espflash
    pkgs.espup
    pkgs.go-task
    pkgs.rustup
  ];

}
