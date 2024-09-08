use anyhow::Context;
use clap::Parser;
use prost::Message;
use prost_reflect::DescriptorPool;
use prost_reflect::DynamicMessage;
use prost_reflect::SerializeOptions;
use serde_json::de::Deserializer;
use std::env::temp_dir;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

#[derive(clap::Parser)]
#[clap(version, about)]
struct Args {
    /// Path to the `.proto` file containing the message definition.
    #[arg(short, long)]
    proto: PathBuf,
    /// Specify the directories in which to search for imports. May be specified multiple times.
    /// If omitted, the current working directory is used.
    #[arg(long)]
    path: Vec<PathBuf>,
    /// Name of the message definition, e.g. `somenamespace.SomeMessage`.
    #[arg(short, long)]
    message: String,

    #[command(subcommand)]
    cmd: ArgsCommand,
}

#[derive(clap::Subcommand)]
#[clap(infer_subcommands = true)]
enum ArgsCommand {
    /// Read a JSON object (UTF-8) from stdin and write its protobuf-encoded raw bytes to stdout.
    Encode,
    /// Read protobuf-encoded raw bytes from stdin and write its JSON representation (UTF-8) to sdout.
    Decode,
}

fn main() -> anyhow::Result<()> {
    let Args {
        proto,
        path,
        message,
        cmd,
    } = Args::parse();

    let mut descriptor_path = temp_dir();
    descriptor_path.push(Uuid::new_v4().to_string());

    let output = Command::new("protoc").arg("--version").output()?;
    if !output.status.success() {
        anyhow::bail!("failed to run `protoc --version`")
    }

    let mut command = Command::new("protoc");

    command.arg(proto);

    for path in path {
        command.arg("--proto_path");
        command.arg(path);
    }

    command
        .arg("--include_imports")
        .arg("--descriptor_set_out")
        .arg(&descriptor_path);

    let output = command.output()?;

    if !output.status.success() {
        anyhow::bail!(
            "failed to compile descriptor set file using protoc - stderr:{}",
            String::from_utf8(output.stderr).unwrap_or_else(|_| "<NON-UTF8>".to_string())
        );
    }

    let descriptor_bytes = std::fs::read(descriptor_path)?;
    let desriptor_pool = DescriptorPool::decode(descriptor_bytes.as_slice())?;
    let message_descriptor = desriptor_pool
        .get_message_by_name(&message)
        .context(format!("no descriptor found for message `{message}`"))?;

    let mut stdin = std::io::stdin().lock();
    let mut bytes = vec![];
    stdin.read_to_end(&mut bytes)?;

    let mut stdout = std::io::stdout().lock();
    match cmd {
        ArgsCommand::Encode => {
            let mut deserializer = Deserializer::from_slice(&bytes);
            let dynamic_message =
                DynamicMessage::deserialize(message_descriptor.clone(), &mut deserializer)?;
            deserializer.end()?;
            let bytes = dynamic_message.encode_to_vec();
            stdout.write_all(&bytes)?;
        }
        ArgsCommand::Decode => {
            let msg = DynamicMessage::decode(message_descriptor, bytes.as_slice())?;
            let mut serializer = serde_json::Serializer::new(&mut stdout);
            msg.serialize_with_options(
                &mut serializer,
                &SerializeOptions::new().use_proto_field_name(true),
            )?;
        }
    }

    stdout.flush()?;

    Ok(())
}
