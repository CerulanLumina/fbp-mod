use crate::iter::ContainerExt;
use anyhow::Context;
use clap::{Parser, Subcommand};
use either::Either;
use factorio_blueprint::{BlueprintCodec, Container};
use std::{
    fs::File,
    io::{copy, stdin, stdout, StdinLock, StdoutLock},
    path::PathBuf,
};

mod iter;

fn manipulations(
    command: Command,
    input: Either<File, StdinLock>,
    output: Either<File, StdoutLock>,
) -> anyhow::Result<()> {
    let mut bp = Container::decode(input).context("Decoding blueprint string")?;

    match command {
        Command::RemoveEntity { name } => {
            bp.blueprints()
                .for_each(|a| a.entities.retain(|e| e.name != name));
        }
        Command::RemoveTile { name } => {
            bp.blueprints()
                .for_each(|a| a.tiles.retain(|t| t.name != name));
        }
        Command::TransmuteEntity { from, to } => {
            bp.blueprints().for_each(|bp| {
                bp.entities
                    .iter_mut()
                    .filter(|e| e.name == from)
                    .for_each(|e| e.name.clone_from(&to));
            });
        }
        Command::TransmuteTile { from, to } => {
            bp.blueprints().for_each(|bp| {
                bp.tiles
                    .iter_mut()
                    .filter(|e| e.name == from)
                    .for_each(|e| e.name.clone_from(&to));
            });
        }
        _ => unreachable!(),
    }
    bp.encode(output).context("Writing blueprint to stdout")?;
    Ok(())
}

fn codecs(
    command: &Command,
    mut input: Either<File, StdinLock>,
    mut output: Either<File, StdoutLock>,
) -> anyhow::Result<()> {
    match command {
        &Command::Decode { pretty } => BlueprintCodec::decode_reader(input, |mut reader| {
            if pretty {
                let mut formatter = jsonxf::Formatter::pretty_printer();
                "\n".clone_into(&mut formatter.trailing_output);
                formatter.format_stream(&mut reader, &mut output)
            } else {
                copy(&mut reader, &mut output).and(Ok(()))
            }
        })
        .context("Decoding and writing blueprint JSON"),
        Command::Encode => BlueprintCodec::encode_writer(output, |mut writer| {
            jsonxf::minimize_stream(&mut input, &mut writer)
        })
        .context("Encoding and writing blueprint string"),
        _ => unreachable!(),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let input = match cli.input {
        Some(path) => Either::Left(File::open(path).context("Opening file")?),
        None => Either::Right(stdin().lock()),
    };

    let output = match cli.output {
        Some(path) => {
            let file = if cli.force {
                File::create(path).context("Creating & overwriting file")
            } else {
                File::create_new(path).context("Creating new file")
            }?;
            Either::Left(file)
        }
        None => Either::Right(stdout().lock()),
    };

    match cli.command {
        command @ (Command::Decode { .. } | Command::Encode) => codecs(&command, input, output),
        command @ (Command::RemoveEntity { .. }
        | Command::RemoveTile { .. }
        | Command::TransmuteEntity { .. }
        | Command::TransmuteTile { .. }) => manipulations(command, input, output),
    }
    .context("Error running command")?;

    Ok(())
}

#[derive(Parser)]
struct Cli {
    /// Input file containing blueprint string, stdin if absent
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file to write blueprint string, stdout if absent.
    /// Will not overwrite unless -f/--force is used.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Allow overwriting the output file
    #[arg(short, long)]
    force: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Simply decode a blueprint string to JSON
    Decode {
        /// Pretty-print the JSON
        #[arg(short, long)]
        pretty: bool,
    },
    /// Simple encode JSON to a blueprint string
    Encode,
    /// Remove all entities of a type
    RemoveEntity {
        /// Name of the entity type to remove
        name: String,
    },
    /// Remove all tiles of a type
    RemoveTile {
        /// Name of the tile type to remove
        name: String,
    },
    /// Change all entities of a type to a different type
    TransmuteEntity {
        /// Source entity
        from: String,
        /// Destination entity
        to: String,
    },
    /// Change all entities of a type to a different type
    TransmuteTile {
        /// Source entity
        from: String,
        /// Destination entity
        to: String,
    },
}
