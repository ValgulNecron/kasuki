use std::error::Error;
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("shard_descriptor.bin"))
        .compile_protos(&["proto/shard.proto"], &["proto"])?;

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("info_descriptor.bin"))
        .compile_protos(&["proto/info.proto"], &["proto"])?;

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("command_descriptor.bin"))
        .compile_protos(&["proto/command.proto"], &["proto"])?;

    cynic_codegen::register_schema("anilist")
        .from_sdl_file("schemas/anilist.graphql")?
        .as_default()?;

    Ok(())
}
