//! Text Utilities
//!
//! Provides text-processing helpers used during indexing, primarily for
//! tokenization of eBook contents. 
//!
//! ## Responsibilities
//! - Normalize text by converting to lowercase  
//! - Extract valid alphabetic tokens using regular expressions  
//! - Filter out very short or irrelevant words  
//! - Return unique tokens as a `HashSet<String>` for efficient indexing

use regex::Regex;
use std::collections::HashSet;

pub fn tokenize_text(text: &str) -> HashSet<String> {
    let re = Regex::new(r"\b[a-zA-Z]+\b").unwrap();
    re.find_iter(&text.to_lowercase())
        .map(|m| m.as_str().to_string())
        .filter(|word| word.len() > 2)
        .collect()
}