{
  lib,
  craneLib,
  libiconv,
}: craneLib.buildPackage {
  src = craneLib.clearnCargoSource ../.;

  doCheck = true;

  buildInputs = [
    
  ]
  ++ lib.optionals lib.stdenv.isDarwin [ libiconv ];
}
