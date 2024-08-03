## Words
There are three different kinds of ids related to words:

- LBR database id
- JMdict seq
- ichiran seq

Ichiran uses JMdict seqs where applicable, but also adds its own for extra words.

We can probably rely on JMdict ids being stable, but we may want to consider adding custom ids to wordfile and using them as stable ids in the LBR db, especially if we need to add custom words to it later.

A word is identified by the tuple (JMdict seq, written form, reading). Ichiran gives us the written form and reading and the ichiran seq, which we can use to retrieve the JMdict seq.
