//! Creates the `kanjifile.json` and `wordfile.json` files.

mod cli;

use clap::Parser;
use cli::{Cli, Command};
use eyre::WrapErr;
use jadata::{
    jmdict::JMDict, jmdict_furigana, kanjidic2::Kanjidic2, kanjifile::Kanjifile,
    kanjifile_manual::KanjifileManual, kanjifile_names::KanjifileNames,
    kanjifiles_similar::KanjifileSimilar, kradfile::Kradfile, wordfile::Wordfile,
};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Kanjifile {
            kanjidic,
            kradfile,
            names,
            similar,
            manual,
            output,
        } => {
            create_kanjifile(&kanjidic, &kradfile, &names, &similar, &manual, &output)?;
        }
        Command::Wordfile {
            jmdict,
            jmdict_version: version,
            furigana,
            output,
        } => {
            create_wordfile(&jmdict, &furigana, &output, version)?;
        }
    }

    Ok(())
}

fn create_kanjifile(
    kanjidic_path: &Path,
    kradfile_path: &Path,
    names_path: &Path,
    similar_path: &Path,
    manual_path: &Path,
    output_path: &Path,
) -> eyre::Result<()> {
    tracing::info!("opening files");
    let kd2 = open(kanjidic_path)?;
    let kf = open(kradfile_path)?;
    let kfn = open(names_path)?;
    let kfs = open(similar_path)?;
    let kfm = open(manual_path)?;

    tracing::info!("deserializing files");
    let kd2: Kanjidic2 = serde_xml_rs::from_reader(BufReader::new(kd2))?;
    let kf: Kradfile = Kradfile::from(BufReader::new(kf))?;
    let kfn: KanjifileNames = serde_json::from_reader(BufReader::new(kfn))?;
    let kfs: KanjifileSimilar = serde_json::from_reader(BufReader::new(kfs))?;
    let kfm: KanjifileManual = serde_json::from_reader(BufReader::new(kfm))?;

    tracing::info!("producing kanjifile");
    let kanjifile = Kanjifile::derive(kd2, kf, kfn, kfs, kfm);

    tracing::info!("writing output");
    let kf = File::create(output_path)?;
    serde_json::to_writer_pretty(BufWriter::new(kf), &kanjifile)?;
    Ok(())
}

fn create_wordfile(
    jmdict: &Path,
    jmdict_furigana: &Path,
    output: &Path,
    version: String,
) -> eyre::Result<()> {
    tracing::info!("opening files");
    let jmdict = open(jmdict)?;
    let furigana = open(jmdict_furigana)?;

    tracing::info!("deserializing");
    let jmdict = JMDict::deserialize(BufReader::new(jmdict))?;
    let furigana: Vec<jmdict_furigana::Furigana> =
        serde_json::from_reader(BufReader::new(furigana))?;

    tracing::info!("producing wordfile");
    let wordfile = Wordfile::from_jmdict_with_furigana(jmdict, version, furigana);

    tracing::info!("writing output");
    let wf = File::create(output)?;
    serde_json::to_writer_pretty(BufWriter::new(wf), &wordfile)?;
    Ok(())
}

fn open(path: &Path) -> eyre::Result<File> {
    File::open(path).wrap_err_with(|| format!("Failed to open file at '{}'", path.display()))
}
