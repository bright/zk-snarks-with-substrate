{ project ? import ./nix/default.nix { }
}:

project.pkgs.mkShell {
  buildInputs = project.devTools;
  LIBCLANG_PATH = "${project.pkgs.llvmPackages.libclang}/lib/libclang.so";
  PROTOC = "${project.pkgs.protobuf}/bin/protoc";
  ROCKSDB = "${project.pkgs.rocksdb}/lib/librocksdb.so";
}
