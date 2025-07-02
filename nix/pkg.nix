{
  rustPlatform,
  pkg-config,
  libcec_platform,
  libcec,
  systemd,
}:
rustPlatform.buildRustPackage rec {
  pname = "htpc-cec-ctrl";
  version = "0.1.0";

  src = "${../.}";

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    libcec_platform
    libcec
    systemd
  ];
}
