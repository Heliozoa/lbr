# `jadata`

Derives the `kanjifile.json` and `wordfile.json` files used by lbr from

- [KANJIDIC2](https://www.edrdg.org/wiki/index.php/KANJIDIC_Project) (`kanjidic2.xml`) from The Electronic Dictionary Research and Development Group
- [KRADFILE](https://www.edrdg.org/krad/kradinf.html) (`kradfile`) from the The Electronic Dictionary Research and Development Group
- [JMdict](https://www.edrdg.org/wiki/index.php/JMdict-EDICT_Dictionary_Project) (`JMdict_e_examp.xml`) from The Electronic Dictionary Research and Development Group
- [JmdictFurigana](https://github.com/Doublevil/JmdictFurigana) (`JmdictFurigana.json`) from Doublevil
- `./included/kanjifile_manual.json`, a manually maintained list of kanji that are missing from the previous files
- `./included/kanjifile_names.json`, a manually maintained list of kanji names
- `./included/kanjifile_similar.json`, a manually maintained list of kanji that are similar to each other

## License

jadata's code is licensed under AGPL-3.0.

The files created by the program are licensed under CC BY-SA 4.0, matching the license of the files used in their generation.
