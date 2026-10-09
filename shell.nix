{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = [
    pkgs.espflash
    pkgs.espup
    pkgs.ffmpeg
    pkgs.go-task
    pkgs.probe-rs-tools
    (pkgs.python3.withPackages (p: [ p.matplotlib ]))
    pkgs.rustup
    pkgs.v4l-utils
  ];

}
