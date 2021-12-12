use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder, SpirvMetadata};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    path: PathBuf,
    /// Split up the resulting SPIR-V module into one file per entry point.
    #[structopt(long)]
    multimodule: bool,
    #[structopt(long)]
    debug: bool,
    #[structopt(long, default_value = "spirv-unknown-spv1.0")]
    target: String,
    /// A list of capabilities to enable, such as `Int8`.
    #[structopt(long, parse(try_from_str = parse_capability))]
    capabilities: Vec<Capability>,
    /// A list of extensions to enable, such as `SPV_KHR_ray_tracing`.
    #[structopt(long)]
    extensions: Vec<String>,
    /// The directory to write the output shaders to. By default they're written to the parent of <path>.
    #[structopt(long)]
    output: Option<PathBuf>,
    #[structopt(long, default_value = "none", parse(try_from_str = parse_spirv_metadata))]
    spirv_metadata: SpirvMetadata,
}

fn parse_spirv_metadata(string: &str) -> anyhow::Result<SpirvMetadata> {
    match string {
        "none" => Ok(SpirvMetadata::None),
        "full" => Ok(SpirvMetadata::Full),
        "name-variables" => Ok(SpirvMetadata::NameVariables),
        _ => Err(anyhow::anyhow!(
            "Expecting one of none,full,name-variables: {}",
            string
        )),
    }
}

fn parse_capability(string: &str) -> anyhow::Result<Capability> {
    Capability::from_str(string)
        .map_err(|()| anyhow::anyhow!("Failed to parse capability: {}", string))
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let output = if let Some(output) = opt.output.as_ref() {
        output
    } else {
        opt.path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Path has no parent: {}", opt.path.display()))?
    };

    let file_name = opt
        .path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Path has no file name: {}", opt.path.display()))?;

    let mut builder = SpirvBuilder::new(&opt.path, &opt.target)
        .print_metadata(MetadataPrintout::None)
        .multimodule(opt.multimodule)
        .release(!opt.debug)
        .spirv_metadata(opt.spirv_metadata);

    for extension in &opt.extensions {
        builder = builder.extension(extension);
    }

    for capability in &opt.capabilities {
        builder = builder.capability(*capability);
    }

    let result = builder.build()?;

    std::fs::create_dir_all(&output)?;

    if opt.multimodule {
        for (name, path) in result.module.unwrap_multi() {
            let name = name.replace("::", "_");
            let mut output = output.join(name);
            output.set_extension("spv");
            std::fs::copy(path, &output)?;
        }
    } else {
        let mut output = output.to_path_buf();
        output.push(file_name);
        output.set_extension("spv");
        std::fs::copy(result.module.unwrap_single(), &output)?;
    }

    Ok(())
}
