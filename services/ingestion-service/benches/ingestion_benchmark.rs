use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn header_body_split(text: &str) -> (String, String) {
    let start_marker = "*** START OF THE PROJECT GUTENBERG EBOOK";
    let end_marker = "*** END OF THE PROJECT GUTENBERG EBOOK";

    if let Some(start_pos) = text.find(start_marker) {
        let header = text[..start_pos].to_string();

        if let Some(end_pos) = text.find(end_marker) {
            let body_start = text[start_pos..].find('\n').map(|pos| start_pos + pos + 1).unwrap_or(start_pos);
            let body = text[body_start..end_pos].to_string();
            return (header, body);
        }
    }

    (text.to_string(), String::new())
}

fn benchmark_header_body_split(c: &mut Criterion) {
    let sample_text = format!(
        "Project Gutenberg EBook\nTitle: Test Book\nAuthor: Test Author\n\n{}\nThis is the main content of the book.\nLorem ipsum dolor sit amet.\n{}\nEnd of book.",
        "*** START OF THE PROJECT GUTENBERG EBOOK",
        "*** END OF THE PROJECT GUTENBERG EBOOK"
    );

    c.bench_function("header_body_split", |b| {
        b.iter(|| header_body_split(black_box(&sample_text)))
    });
}

fn benchmark_header_body_split_large(c: &mut Criterion) {
    let large_content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(10000);
    let sample_text = format!(
        "Project Gutenberg EBook\nTitle: Large Test Book\nAuthor: Test Author\n\n{}\n{}\n{}\nEnd of book.",
        "*** START OF THE PROJECT GUTENBERG EBOOK",
        large_content,
        "*** END OF THE PROJECT GUTENBERG EBOOK"
    );

    c.bench_function("header_body_split_large", |b| {
        b.iter(|| header_body_split(black_box(&sample_text)))
    });
}

criterion_group!(benches, benchmark_header_body_split, benchmark_header_body_split_large);
criterion_main!(benches);