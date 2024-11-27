{ lib
, rustPlatform
, fetchFromGitHub
, pkg-config
, stdenv
, darwin
, alsa-lib
, jack2
}:

rustPlatform.buildRustPackage rec {
  pname = "jack_notification_rs";
  version = "0.1.0";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  cargoLock.outputHashes = {
         "jack-0.13.0" = "sha256-OiaB6sVY9Ot5PKd5gdeUNYZ57X42mv8YPcmEkKavkLM=";
       };

  nativeBuildInputs = [
    pkg-config
    rustPlatform.bindgenHook
  ];

  buildInputs = [
    alsa-lib
    jack2
  ];

  meta = with lib; {
    description = "jack notifications for rust";
    homepage = "git@github.com:crop2000/" + pname + ".git";
    license = licenses.gpl3Only;
    maintainers = with maintainers; [ ];
    mainProgram = "jack_notification_rs";
  };
}