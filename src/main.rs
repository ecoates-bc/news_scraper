use std::path::*;
use anyhow::Result;

mod prepare_dataset;
mod scrape_data;

fn main() {
    let root_dir = Path::new("scraped");
    let raw_data = prepare_dataset::get_raw_dataset(root_dir, 0.2);
    dbg!(raw_data);
}
