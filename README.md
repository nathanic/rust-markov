# Markov Text Generator in Rust

This is just a silly little project to explore Rust.

The program reads in text files and (roughly) builds a [Markov model](https://en.wikipedia.org/wiki/Markov_modelhttps://en.wikipedia.org/wiki/Markov_modelhttps://en.wikipedia.org/wiki/Markov_model) of the character transitions.  From these models, it can generate new random text with similar statistical properties to the source material.

I recently cargoized it, and it now depends on a [cargoized fork I made of rust-msgpack](https://github.com/nathanic/rust-msgpackhttps://github.com/nathanic/rust-msgpack).

TODO: sample output
