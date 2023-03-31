use anyhow::Result;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::{fs, io, path::*};

#[derive(Debug, Clone)]
struct ArticleEntry {
    date: String,
    path: PathBuf,
    site: String,
}

#[derive(Debug)]
pub struct RawDataset {
    train: Vec<ArticleEntry>,
    test: Vec<ArticleEntry>,
}

fn get_news_site_paths(root_dir: &Path, site: String) -> Result<Vec<ArticleEntry>> {
    let day_dir_paths = fs::read_dir(root_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let entries = day_dir_paths
        .into_iter()
        .flat_map(|path| match read_day_path(&path, &site) {
            Ok(articles) => articles,
            Err(_) => Vec::new(),
        })
        .collect();
    Ok(entries)
}

fn parse_date(date_buf: &PathBuf) -> String {
    let date_slice = date_buf
        .file_name()
        .expect("Cannot get file name.")
        .to_str()
        .expect("Cannot convert path to string.");
    String::from(date_slice)
        .replace("March", "03")
        .replace("Mar", "03")
        .replace("Feb", "02")
}

fn read_day_path(day_dir: &PathBuf, site: &String) -> Result<Vec<ArticleEntry>> {
    let article_paths = fs::read_dir(day_dir)?
        .map(|res| {
            res.map(|e| ArticleEntry {
                date: parse_date(day_dir),
                path: e.path(),
                site: String::from(site),
            })
        })
        .collect::<Result<Vec<_>, io::Error>>()?;

    return Ok(article_paths);
}

pub fn get_raw_dataset(scraped_path: &Path, train_test_split: f32) -> Result<RawDataset> {
    let sites = fs::read_dir(scraped_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    let mut articles: Vec<ArticleEntry> = sites
        .into_iter()
        .flat_map(|dir| {
            dbg!(&dir);
            get_news_site_paths(
                &dir,
                String::from(
                    dir.file_name()
                        .expect("Cannot read directory name.")
                        .to_str()
                        .expect("Cannot convert to str"),
                ),
            )
        })
        .flatten()
        .collect();

    let mut rng = StdRng::seed_from_u64(12345);
    articles.shuffle(&mut rng);

    let n_train = (articles.len() as f32 * train_test_split).round() as usize;
    Ok(RawDataset {
        train: articles[..n_train].to_vec(),
        test: articles[n_train..].to_vec(),
    })
}
