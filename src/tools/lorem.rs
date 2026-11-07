use crate::error::{Result, ToolError};
use crate::tools::rng::SecureRng;

const WORDS: &[&str] = &[
    "lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
    "sed",
    "do",
    "eiusmod",
    "tempor",
    "incididunt",
    "ut",
    "labore",
    "et",
    "dolore",
    "magna",
    "aliqua",
    "enim",
    "ad",
    "minim",
    "veniam",
    "quis",
    "nostrud",
    "exercitation",
    "ullamco",
    "laboris",
    "nisi",
    "aliquip",
    "ex",
    "ea",
    "commodo",
    "consequat",
    "duis",
    "aute",
    "irure",
    "in",
    "reprehenderit",
    "voluptate",
    "velit",
    "esse",
    "cillum",
    "fugiat",
    "nulla",
    "pariatur",
    "excepteur",
    "sint",
    "occaecat",
    "cupidatat",
    "non",
    "proident",
    "sunt",
    "culpa",
    "qui",
    "officia",
    "deserunt",
    "mollit",
    "anim",
    "id",
    "est",
    "laborum",
    "vitae",
    "elementum",
    "sagittis",
    "viverra",
    "accumsan",
    "nisl",
    "porttitor",
    "lacus",
    "facilisis",
    "volutpat",
    "blandit",
    "rhoncus",
    "faucibus",
    "interdum",
    "posuere",
    "ornare",
    "turpis",
    "massa",
    "tincidunt",
    "dui",
    "arcu",
];

/// Generates classic Latin filler text.
pub fn generate(paragraphs: usize, sentences: usize, words: usize) -> Result<String> {
    if !(1..=20).contains(&paragraphs) {
        return Err(ToolError::new("Paragraph count must be between 1 and 20."));
    }
    if !(1..=20).contains(&sentences) {
        return Err(ToolError::new("Sentence count must be between 1 and 20."));
    }
    if !(3..=50).contains(&words) {
        return Err(ToolError::new(
            "Words per sentence must be between 3 and 50.",
        ));
    }

    let mut rng = SecureRng::new();
    let mut paragraphs_out = Vec::with_capacity(paragraphs);

    for paragraph_index in 0..paragraphs {
        let mut sentences_out = Vec::with_capacity(sentences);
        for sentence_index in 0..sentences {
            if paragraph_index == 0 && sentence_index == 0 {
                sentences_out
                    .push("Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_owned());
                continue;
            }
            let jitter = rng.index(5)? as i32 - 2;
            let target = (words as i32 + jitter).clamp(3, 50) as usize;
            let mut chosen = Vec::with_capacity(target);
            for _ in 0..target {
                let index = rng.index(WORDS.len())?;
                chosen.push(WORDS[index]);
            }
            let mut sentence = chosen.join(" ");
            if let Some(first) = sentence.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            sentence.push('.');
            sentences_out.push(sentence);
        }
        paragraphs_out.push(sentences_out.join(" "));
    }

    Ok(paragraphs_out.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::generate;

    #[test]
    fn builds_requested_paragraphs() -> Result<(), Box<dyn std::error::Error>> {
        let text = generate(3, 4, 10)?;
        assert_eq!(text.split("\n\n").count(), 3);
        assert!(text.starts_with("Lorem ipsum dolor sit amet"));
        assert!(text.ends_with('.'));
        Ok(())
    }

    #[test]
    fn validates_limits() {
        assert!(generate(0, 1, 10).is_err());
        assert!(generate(1, 21, 10).is_err());
        assert!(generate(1, 1, 2).is_err());
    }
}
