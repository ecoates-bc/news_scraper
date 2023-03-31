use std::path::*;

mod prepare_dataset;
mod preprocess;
mod scrape_data;

fn main() {
    let root_dir = Path::new("scraped");
    let raw_data =
        prepare_dataset::get_raw_dataset(root_dir, 0.2).expect("Dataset could not be loaded.");
    let tokenizer =
        preprocess::load_tokenizer("bert-base-cased").expect("Tokenizer could not be loaded");

    let indexer = preprocess::create_word_indexer(&tokenizer, &raw_data.train)
        .expect("Cannot construct indexer.");
    let deindexer = preprocess::create_word_deindexer(&indexer);

    dbg!(indexer["into"]);
    dbg!(&deindexer[&indexer["into"]]);
}
