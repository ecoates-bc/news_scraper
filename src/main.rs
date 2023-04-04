use std::path::*;

use crate::preprocess::{tokenize_headline_text, load_tokenizer};

mod prepare_dataset;
mod preprocess;
mod scrape_data;

fn main() {
    let root_dir = Path::new("scraped");
    let raw_data =
        prepare_dataset::get_raw_dataset(root_dir, 0.2).expect("Dataset could not be loaded.");
    let tokenizer = load_tokenizer().expect("Could not load tokenizer.");
    dbg!("Tokenizer loaded...");

    let indexer = preprocess::create_word_indexer(&tokenizer, &raw_data.train)
        .expect("Cannot construct indexer.");
    let deindexer = preprocess::create_word_deindexer(&indexer);

    let entry = raw_data.train.last().unwrap();
    let tokens = tokenize_headline_text(&tokenizer, &entry.path).unwrap();
    dbg!(&tokens);
    dbg!(indexer.encode_sequence(&tokens));
    dbg!(deindexer.decode_sequence(&indexer.encode_sequence(&tokens)));
}
