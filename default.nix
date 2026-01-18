{
  makeRustPlatform,
  rust-bin,
  openssl,
  pkg-config,
}:
let
  toolchain = rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  rustPlatform = makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = "ghr";
  version = "0.4.5";

  buildInputs = [ openssl ];
  nativeBuildInputs = [ pkg-config ];

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
