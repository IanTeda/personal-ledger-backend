// build.rs
// Build script for personal-ledger-backend
// Compiles protobuf files using tonic_prost_build to generate Rust gRPC code
// 
// Note: Uncomment `out_dir` and `.file_descriptor` if you want tonic_prost_build
// to build the code in the OUT_DIR (i.e. /target) instead of directly in src/rpc.
// This will also require adjusting the module paths in src/rpc/mod.rs accordingly.

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the cargo OUT_DIR environment variable, which is where the generated code will be placed
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    
    // Re-run the build script if any .proto files change
    println!("cargo:rerun-if-changed=proto/");
    // Re-run the build script if this file changes
    println!("cargo:rerun-if-changed=build.rs");

    // Compile all .proto files in the proto/ directory
    tonic_prost_build::configure()
        // .out_dir("src/rpc")
        .protoc_arg("--experimental_allow_proto3_optional")
        .protoc_arg("--proto_path=/usr/include")
        .build_client(true)
        .build_server(true)
        .build_transport(true)
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::prost_types")
        .file_descriptor_set_path(out_dir.join("personal_ledger_descriptor.bin"))
        .compile_protos(
          &["proto/utilities.proto", "proto/categories.proto"], 
          &["proto/", "/usr/include"])?;
    Ok(())
}
