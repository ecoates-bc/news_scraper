use anyhow;
use nlprule::Tokenizer;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::{fs, io};

use crate::prepare_dataset::ArticleEntry;


pub struct Indexer {
    index_map: HashMap<String, usize>,
}

impl Indexer {
    fn new(index_map: HashMap<String, usize>) -> Indexer {
        Indexer {
            index_map: index_map,
        }
    }

    pub fn get_index(&self, token: &String) -> usize {
        if self.index_map.contains_key(token) {
            self.index_map[token]
        } else {
            self.index_map.len() + 1
        }
    }

    pub fn encode_sequence(&self, tokens: &Vec<String>) -> Vec<usize> {
        tokens.into_iter().map(|tok| self.get_index(tok)).collect()
    }
}

pub struct Deindexer {
    token_map: HashMap<usize, String>,
}

impl Deindexer {
    fn new(token_map: HashMap<usize, String>) -> Deindexer {
        Deindexer {
            token_map: token_map,
        }
    }

    pub fn get_token(&self, index: usize) -> String {
        if index > self.token_map.len() {
            "UNK".into()
        } else {
            self.token_map[&index].clone()
        }
    }

    pub fn decode_sequence(&self, idx: &Vec<usize>) -> Vec<String> {
        idx.into_iter().map(|id| self.get_token(*id)).collect()
    }
}

pub fn load_tokenizer() -> anyhow::Result<Tokenizer> {
    println!("Loading tokenizer...");
    Ok(Tokenizer::new("./en_tokenizer.bin")?)
}

pub fn read_article_text(path: &Path) -> io::Result<String> {
    let body = fs::read_to_string(path)?;
    if body.len() > 0 {
        Ok(body.to_lowercase())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Article body empty"))
    }
}

fn get_tokens(tokenizer: &Tokenizer, text: &String) -> Vec<String> {
    let tokens = tokenizer.pipe(text).flat_map(|s| s.tokens().to_vec());
    let tokens = tokens.map(|t| t.word().as_str().to_string());
    tokens.collect()
}

pub fn tokenize_article_text(
    tokenizer: &Tokenizer,
    article_path: &Path,
) -> anyhow::Result<Vec<String>> {
    let article_text = read_article_text(article_path)?;
    Ok(get_tokens(tokenizer, &article_text))
}

pub fn tokenize_headline_text(
    tokenizer: &Tokenizer,
    article_path: &Path,
) -> anyhow::Result<Vec<String>> {
    let headline = article_path
        .file_stem()
        .ok_or(anyhow::anyhow!("Could not find stem."))?;
    let headline = headline
        .to_str()
        .ok_or(anyhow::anyhow!("Could not convert to str"))?;
    let headline = String::from(headline).replace("_", " ");
    Ok(get_tokens(tokenizer, &headline))
}

pub fn create_word_indexer(
    tokenizer: &Tokenizer,
    ds: &Vec<ArticleEntry>,
) -> anyhow::Result<Indexer> {
    let article_tokens = ds
        .into_iter()
        .map(
            |entry| match tokenize_headline_text(tokenizer, &entry.path) {
                Ok(tokens) => tokens,
                Err(_) => vec![],
            },
        )
        .flatten();
    let token_set: HashSet<String, RandomState> = HashSet::from_iter(article_tokens);
    let indexer = HashMap::from_iter(token_set.iter().enumerate().map(|(i, s)| (s.clone(), i)));
    if indexer.len() > 0 {
        Ok(Indexer::new(indexer))
    } else {
        Err(anyhow::anyhow!("Could not create indexer."))
    }
}

pub fn create_word_deindexer(indexer: &Indexer) -> Deindexer {
    let deidx_iter = indexer
        .index_map
        .clone()
        .into_iter()
        .map(|(s, i)| (i, s.clone()));
    let deidxer = HashMap::from_iter(deidx_iter);
    Deindexer::new(deidxer)
}
