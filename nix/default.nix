{
  lib,
  craneLib,
  libiconv,
}: craneLib.buildPackage {
  src = craneLib.cleanCargoSource ../.;

  doCheck = true;

  buildInputs = [
    
  ]
  ++ lib.optionals lib.stdenv.isDarwin [ libiconv ];
}
