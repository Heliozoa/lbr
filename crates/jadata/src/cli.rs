use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Kanjifile {
        /// The path to the input KANJIDIC2 file,
        #[arg(short = 'd', long)]
        kanjidic: PathBuf,
        /// The path to the input KRADFILE.
        #[arg(short, long)]
        kradfile: PathBuf,
        /// The path to the kanjifile_names.json file.
        #[arg(short, long)]
        names: PathBuf,
        /// The path to the kanjifile_similar.json file.
        #[arg(short, long)]
        similar: PathBuf,
        /// The path to the kanjifile_manual.json file.
        #[arg(short, long)]
        manual: PathBuf,
        /// The path to the output kanjifile.
        #[arg(short, long)]
        output: PathBuf,
    },
    Wordfile {
        /// The path to the input JMDICT file.
        #[arg(short, long)]
        jmdict: PathBuf,
        /// The revision number of the input JMDICT file (Rev.)
        #[arg(short = 'v', long)]
        jmdict_version: String,
        /// The path to the input JMDICT furigana file.
        #[arg(short, long)]
        furigana: PathBuf,
        /// The path to the output wordfile.
        #[arg(short, long)]
        output: PathBuf,
    },
}
