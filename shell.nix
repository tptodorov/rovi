{
  pkgs ? import <nixpkgs> { },
}:

let
  # The rovi-protocol Python binding (ADR-0017), built from the repo's Rust crate.
  roviProtocol = pkgs.python3Packages.buildPythonPackage {
    pname = "rovi-protocol";
    version = "0.1.0";
    pyproject = true;
    src = pkgs.lib.fileset.toSource {
      root = ./.;
      fileset = pkgs.lib.fileset.unions [
        ./crates/rovi-protocol/Cargo.toml
        ./crates/rovi-protocol/src
        ./bindings/python/Cargo.toml
        ./bindings/python/Cargo.lock
        ./bindings/python/pyproject.toml
        ./bindings/python/src
      ];
    };
    cargoRoot = "bindings/python";
    buildAndTestSubdir = "bindings/python";
    cargoDeps = pkgs.rustPlatform.importCargoLock {
      lockFile = ./bindings/python/Cargo.lock;
    };
    nativeBuildInputs = [
      pkgs.rustPlatform.cargoSetupHook
      pkgs.rustPlatform.maturinBuildHook
    ];
    pythonImportsCheck = [ "rovi_protocol" ];
  };
in
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
      roviProtocol
    ]))
    pkgs.rustup
    pkgs.v4l-utils
  ];

}
