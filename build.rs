use std::error::Error;
use std::{env, path::PathBuf};

/// The main function of the build script.
///
/// This function is responsible for generating Rust code from Protocol Buffers using the `tonic_build` crate.
/// It first retrieves the output directory from the environment variable "OUT_DIR".
/// It then configures the `tonic_build` to output a file descriptor set binary file in the output directory.
/// The function then compiles the Protocol Buffers file `shard.proto` located in the `proto` directory.
/// Finally, it compiles the Protocol Buffers file `shard.proto` again without any additional configuration.
///
/// # Returns
///
/// This function returns a `Result` with an empty tuple as the `Ok` variant and a `Box<dyn Error>` as the `Err` variant.
/// If the function executes successfully, it will return `Ok(())`.
/// If an error occurs during the execution, it will return `Err(e)` where `e` is the error that occurred.
fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("shard_descriptor.bin"))
        .compile(&["proto/shard.proto"], &["proto"])?;
    tonic_build::compile_protos("proto/shard.proto")?;

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("info_descriptor.bin"))
        .compile(&["proto/info.proto"], &["proto"])?;
    tonic_build::compile_protos("proto/info.proto")?;

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("command_descriptor.bin"))
        .compile(&["proto/command.proto"], &["proto"])?;
    tonic_build::compile_protos("proto/command.proto")?;

    tonic_build::configure().file_descriptor_set_path(out_dir.join("federation_descriptor.bin"))
        .compile(&["proto/federation.proto"], &["proto"])?;
    tonic_build::compile_protos("proto/federation.proto")?;

    cynic_codegen::register_schema("anilist")
        .from_sdl_file("schemas/anilist.graphql")?
        .as_default()?;

    Ok(())
}
