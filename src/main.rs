use std::path::*;

use crate::preprocess::{load_tokenizer, tokenize_headline_text};

mod prepare_dataset;
mod preprocess;
mod scrape_data;
mod tf_idf;

fn main() {
    let root_dir = Path::new("scraped");
    let raw_data =
        prepare_dataset::get_raw_dataset(root_dir, 0.2).expect("Dataset could not be loaded.");
    let tokenizer = load_tokenizer().expect("Could not load tokenizer.");
    println!("Tokenizer loaded...");

    let indexer = preprocess::create_word_indexer(&tokenizer, &raw_data.train)
        .expect("Cannot construct indexer.");
    let deindexer = preprocess::create_word_deindexer(&indexer);

    let token_counter = tf_idf::get_token_article_frequencies(&tokenizer, &raw_data.train)
        .expect("Could not count headline token frequencies.");

    dbg!(tf_idf::get_tfidf_distribution(&token_counter, &tokenizer, &raw_data.train));
}
