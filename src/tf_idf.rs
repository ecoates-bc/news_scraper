use anyhow::{anyhow, Result};
use nlprule::Tokenizer;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};

use crate::prepare_dataset::ArticleEntry;
use crate::preprocess::{tokenize_headline_text, tokenize_article_text, read_article_text};

pub struct DocumentFrequencyCounter {
    pub counter: HashMap<String, usize>,
    pub n_documents: usize,
}


#[derive(Hash)]
pub struct TfIdfResult {
    token: String,
    tfidf: u64,
}

impl TfIdfResult {
    fn new(token: &String, tfidf: f32) -> Self {
        TfIdfResult { token: String::from(token), tfidf: (tfidf * 100.) as u64 }
    }
}

impl PartialEq for TfIdfResult {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

impl Eq for TfIdfResult {}

impl DocumentFrequencyCounter {
    pub fn new(doc_freq_map: HashMap<String, usize>, n_documents: usize) -> Self {
        DocumentFrequencyCounter {
            counter: doc_freq_map,
            n_documents: n_documents
        }
    }

    pub fn get_doc_freq(&self, token: &String) -> usize {
        if self.counter.contains_key(token) {
            self.counter[token]
        } else {
            0
        }
    }

    pub fn get_inv_doc_freq(&self, token: &String) -> f32 {
        (self.n_documents as f32 / (1. + self.get_doc_freq(token) as f32)).ln()
    }
}

pub fn get_token_headline_frequencies(
    tokenizer: &Tokenizer,
    train_set: &Vec<ArticleEntry>,
) -> Result<DocumentFrequencyCounter> {
    let mut counter: HashMap<String, usize> = HashMap::new();
    for entry in train_set {
        let tokens = tokenize_headline_text(tokenizer, &entry.path)?;
        tokens.into_iter().for_each(|token| {
            counter.entry(token).and_modify(|e| *e += 1).or_insert(1);
        });
    }
    dbg!(&counter);
    Ok(DocumentFrequencyCounter::new(counter, train_set.len()))
}

pub fn get_token_article_frequencies(
    tokenizer: &Tokenizer,
    train_set: &Vec<ArticleEntry>,
) -> Result<DocumentFrequencyCounter> {
    let mut counter: HashMap<String, usize> = HashMap::new();
    for entry in train_set {
        let tokens = tokenize_article_text(tokenizer, &entry.path);
        match tokens {
            Ok(seq) => {
                let token_set: HashSet<String> = HashSet::from_iter(seq);
                token_set.into_iter().for_each(|token| {
                    counter.entry(token).and_modify(|e| *e += 1).or_insert(1);
                })
            },
            Err(_) => (),
        }
        ;
    }
    dbg!(&counter);
    Ok(DocumentFrequencyCounter::new(counter, train_set.len()))
}

pub fn get_tf_idf(counter: &DocumentFrequencyCounter, token: &String, headline_tokens: &Vec<String>) -> Result<f32> {
    if !headline_tokens.contains(token) {
        Err(anyhow!("Headline does not contain token."))
    } else {
        let term_frequency = headline_tokens.into_iter().fold(0, |acc, other_token| {
            if token == other_token {
                acc + 1
            } else {
                acc
            }
        });
        let term_frequency = term_frequency as f32 / headline_tokens.len() as f32;
        let inverse_document_frequency = counter.get_inv_doc_freq(token);

        if term_frequency * inverse_document_frequency > 0.01 {
            dbg!(token, term_frequency, inverse_document_frequency);
        }

        Ok(term_frequency * inverse_document_frequency)
    }
}

pub fn entry_into_headline_tfidf(counter: &DocumentFrequencyCounter, tokenizer: &Tokenizer, entry: &ArticleEntry) -> Result<Vec<(String, f32)>> {
    let headline_tokens = tokenize_headline_text(tokenizer, &entry.path)?;
    let mut tf_idf_results: Vec<(String, f32)> = Vec::new();

    for token in &headline_tokens {
        let tfidf = get_tf_idf(counter, &token, &headline_tokens)?;
        tf_idf_results.push((token.clone(), tfidf));
    }
    Ok(tf_idf_results)
}

pub fn entry_into_article_tfidf(counter: &DocumentFrequencyCounter, tokenizer: &Tokenizer, entry: &ArticleEntry) -> Result<Vec<TfIdfResult>> {
    let article_tokens = tokenize_article_text(tokenizer, &entry.path)?;
    let mut tf_idf_results: Vec<TfIdfResult> = Vec::new();

    for token in &article_tokens {
        let tfidf = get_tf_idf(counter, &token, &article_tokens)?;
        tf_idf_results.push(TfIdfResult::new(token, tfidf));
    }
    Ok(tf_idf_results)
}

pub fn get_tfidf_distribution(counter: &DocumentFrequencyCounter, tokenizer: &Tokenizer, train_ds: &Vec<ArticleEntry>) -> Result<Vec<(u64, usize)>> {
    let mut dist_counter: HashMap<u64, usize> = HashMap::new();
    for entry in train_ds {
        let tf_idf_results = entry_into_article_tfidf(counter, tokenizer, entry);
        match tf_idf_results {
            Ok(res) => {
                let mut tf_idf_set: HashSet<_> = HashSet::from_iter(res.iter());

                tf_idf_set.into_iter().for_each(|result| {
                    let tfidf = result.tfidf;
                    dist_counter.entry(tfidf).and_modify(|e| *e += 1).or_insert(1);
                });
            }
            Err(_) => ()
        }

    }
    let mut result_vec: Vec<(u64, usize)> = dist_counter.into_iter().map(|(key, val)| (key, val)).collect();
    result_vec.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result_vec)
}