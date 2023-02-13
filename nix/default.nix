{
  lib,
  craneLib,
  libiconv,
  stdenv,
}: craneLib.buildPackage {
  src = craneLib.cleanCargoSource ../.;

  doCheck = true;

  buildInputs = [
    
  ]
  ++ lib.optionals stdenv.isDarwin [ libiconv ];
}
