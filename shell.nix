{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = [
    pkgs.espflash
    pkgs.espup
    pkgs.ffmpeg
    pkgs.go-task
    pkgs.iw
    pkgs.probe-rs-tools
    (pkgs.python3.withPackages (p: [
      p.bleak
      p.matplotlib
      p.opencv4
    ]))
    pkgs.rustup
    pkgs.v4l-utils
  ];

}
