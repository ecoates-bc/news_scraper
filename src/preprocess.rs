use anyhow;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::{fs, io};
use tokenizers::tokenizer::{Result, Tokenizer};

use crate::prepare_dataset::ArticleEntry;

pub fn load_tokenizer(identifier: &str) -> Result<Tokenizer> {
    Tokenizer::from_pretrained(identifier, None)
}

pub fn read_article_text(path: &Path) -> io::Result<String> {
    let body = fs::read_to_string(path)?;
    if body.len() > 0 {
        Ok(body.to_lowercase())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Article body empty"))
    }
}

pub fn tokenize_article_text(tokenizer: &Tokenizer, article_path: &Path) -> Result<Vec<String>> {
    let article_text = read_article_text(article_path)?;
    let encoding = tokenizer.encode(article_text, false)?;
    Ok(encoding.get_tokens().to_vec())
}

pub fn create_word_indexer(
    tokenizer: &Tokenizer,
    ds: &Vec<ArticleEntry>,
) -> anyhow::Result<HashMap<String, usize>> {
    let article_tokens = ds
        .into_iter()
        .map(
            |entry| match tokenize_article_text(tokenizer, &entry.path) {
                Ok(tokens) => tokens,
                Err(_) => vec![],
            },
        )
        .flatten();
    let token_set: HashSet<String, RandomState> = HashSet::from_iter(article_tokens);
    let indexer = HashMap::from_iter(token_set.iter().enumerate().map(|(i, s)| (s.clone(), i)));
    if indexer.len() > 0 {
        Ok(indexer)
    } else {
        Err(anyhow::anyhow!("Could not create indexer."))
    }
}

pub fn create_word_deindexer(indexer: &HashMap<String, usize>) -> HashMap<usize, String> {
    let deidx_iter = indexer.into_iter().map(|(s, i)| (*i, s.clone()));
    HashMap::from_iter(deidx_iter)
}
