//! Indexing Service
//!
//! Handles the extraction, tokenization, and indexing of eBook data stored in the
//! datalake. Each book’s header and body are parsed to extract metadata and
//! normalized word tokens, which are then written into the configured backend
//! (Redis or PostgreSQL).
//!
//! ## Responsibilities
//! - Read header and body files for each book from the datalake  
//! - Extract metadata (title, author, language, year) from the header  
//! - Tokenize the book’s text content and title into searchable words  
//! - Store metadata and word-to-book relationships in the backend  
//! - Ensure consistent indexing for rebuild and incremental ingestion

use crate::models::storage::{Backend, BookMetadata, StorageBackend};
use crate::utils::file::find_book_files;
use crate::utils::text::tokenize_text;
use regex::Regex;
use std::collections::HashSet;
use std::fs;

fn extract_metadata_from_header(header_content: &str, book_id: u32) -> BookMetadata {
    let title_re = Regex::new(r"(?i)title:\s*(.+)").unwrap();
    let author_re = Regex::new(r"(?i)author:\s*(.+)").unwrap();
    let lang_re = Regex::new(r"(?i)language:\s*(.+)").unwrap();
    let year_re =
        Regex::new(r"(?i)(?:release date|posting date|release|date):\s*.*?(\d{4})").unwrap();

    let title = title_re
        .captures(header_content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    let author = author_re
        .captures(header_content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    let language = lang_re
        .captures(header_content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| "en".to_string());

    let year = year_re
        .captures(header_content)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok());

    BookMetadata {
        book_id,
        title,
        author,
        language,
        year,
        word_count: 0,
        unique_words: 0,
    }
}

pub async fn process_book(
    book_id: u32,
    backend: &Backend,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (header_path, body_path) =
        find_book_files(book_id).ok_or(format!("Book {} files not found", book_id))?;

    let header_content = fs::read_to_string(&header_path)?;
    let body_content = fs::read_to_string(&body_path)?;

    let mut metadata = extract_metadata_from_header(&header_content, book_id);
    let words = tokenize_text(&body_content);
    let title_words = tokenize_text(&metadata.title);

    metadata.word_count = body_content.split_whitespace().count();
    metadata.unique_words = words.len();

    let all_words: HashSet<String> = words.union(&title_words).cloned().collect();

    backend.store_book_metadata(&metadata).await?;

    for word in &all_words {
        backend.add_word_to_index(word, book_id).await?;
    }

    Ok(())
}
