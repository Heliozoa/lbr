## Words
There are three different kinds of ids related to words:

- LBR database id
- JMdict seq
- ichiran seq

Ichiran uses JMdict seqs where applicable, but also adds its own for extra words.

We can probably not rely on ichiran seqs being stable, might be best to update the `word_ichiran` table whenever updating the ichiran db.

We can probably rely on JMdict ids being stable, but we may want to consider adding custom ids to wordfile and using them as stable ids in the LBR db, especially if we need to add custom words to it later.
