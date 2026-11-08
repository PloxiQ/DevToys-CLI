use crate::error::{Result, ToolError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStyle {
    Lower,
    Upper,
    Title,
    Sentence,
    Camel,
    Pascal,
    Snake,
    Kebab,
    Constant,
    Dot,
    Train,
}

impl CaseStyle {
    pub const ALL: [Self; 11] = [
        Self::Lower,
        Self::Upper,
        Self::Title,
        Self::Sentence,
        Self::Camel,
        Self::Pascal,
        Self::Snake,
        Self::Kebab,
        Self::Constant,
        Self::Dot,
        Self::Train,
    ];

    pub fn parse(value: &str) -> Result<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
        let style = match normalized.as_str() {
            "lower" | "lowercase" | "lower_case" => Self::Lower,
            "upper" | "uppercase" | "upper_case" => Self::Upper,
            "title" | "title_case" | "capital" | "capitalized" => Self::Title,
            "sentence" | "sentence_case" => Self::Sentence,
            "camel" | "camelcase" | "camel_case" => Self::Camel,
            "pascal" | "pascalcase" | "pascal_case" | "studly" => Self::Pascal,
            "snake" | "snakecase" | "snake_case" => Self::Snake,
            "kebab" | "kebabcase" | "kebab_case" | "dash" => Self::Kebab,
            "constant" | "constantcase" | "constant_case" | "screaming_snake" => Self::Constant,
            "dot" | "dotcase" | "dot_case" => Self::Dot,
            "train" | "traincase" | "train_case" => Self::Train,
            _ => {
                return Err(ToolError::new(format!("Unknown case style `{value}`.")).with_hint(
                    "Use one of: lower, upper, title, sentence, camel, pascal, snake, kebab, constant, dot, train.",
                ));
            }
        };
        Ok(style)
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Lower => "lower",
            Self::Upper => "upper",
            Self::Title => "title",
            Self::Sentence => "sentence",
            Self::Camel => "camel",
            Self::Pascal => "pascal",
            Self::Snake => "snake",
            Self::Kebab => "kebab",
            Self::Constant => "constant",
            Self::Dot => "dot",
            Self::Train => "train",
        }
    }
}

/// Converts text between common naming conventions.
#[must_use]
pub fn convert(input: &str, style: CaseStyle) -> String {
    match style {
        CaseStyle::Lower => input.to_lowercase(),
        CaseStyle::Upper => input.to_uppercase(),
        CaseStyle::Title => join_words(input, " ", WordStyle::Capitalized),
        CaseStyle::Sentence => sentence_case(input),
        CaseStyle::Camel => join_words(input, "", WordStyle::Camel),
        CaseStyle::Pascal => join_words(input, "", WordStyle::Capitalized),
        CaseStyle::Snake => join_words(input, "_", WordStyle::Lower),
        CaseStyle::Kebab => join_words(input, "-", WordStyle::Lower),
        CaseStyle::Constant => join_words(input, "_", WordStyle::Upper),
        CaseStyle::Dot => join_words(input, ".", WordStyle::Lower),
        CaseStyle::Train => join_words(input, "-", WordStyle::Capitalized),
    }
}

#[derive(Debug, Clone, Copy)]
enum WordStyle {
    Lower,
    Upper,
    Capitalized,
    Camel,
}

fn join_words(input: &str, separator: &str, style: WordStyle) -> String {
    let words = split_words(input);
    let mut output = String::new();
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            output.push_str(separator);
        }
        match style {
            WordStyle::Lower => output.push_str(&word.to_lowercase()),
            WordStyle::Upper => output.push_str(&word.to_uppercase()),
            WordStyle::Capitalized => output.push_str(&capitalize(word)),
            WordStyle::Camel if index == 0 => output.push_str(&word.to_lowercase()),
            WordStyle::Camel => output.push_str(&capitalize(word)),
        }
    }
    output
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut output = String::new();
    output.extend(first.to_uppercase());
    output.push_str(&chars.as_str().to_lowercase());
    output
}

fn sentence_case(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut capitalize_next = true;
    for character in input.chars() {
        if capitalize_next && character.is_alphabetic() {
            output.extend(character.to_uppercase());
            capitalize_next = false;
        } else {
            output.extend(character.to_lowercase());
        }
        if matches!(character, '.' | '!' | '?') {
            capitalize_next = true;
        }
    }
    output
}

/// Splits identifiers and prose into words without external tokenizers.
#[must_use]
pub fn split_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if !character.is_alphanumeric() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }

        if let Some(previous) = current.chars().last() {
            let next_is_lower = chars.peek().is_some_and(|next| next.is_lowercase());
            let boundary = character.is_uppercase()
                && (previous.is_lowercase()
                    || previous.is_numeric()
                    || (previous.is_uppercase() && next_is_lower));
            if boundary {
                words.push(std::mem::take(&mut current));
            }
        }
        current.push(character);
    }

    if !current.is_empty() {
        words.push(current);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::{convert, split_words, CaseStyle};

    #[test]
    fn splits_prose_and_identifiers() {
        assert_eq!(split_words("Hello, world!"), ["Hello", "world"]);
        assert_eq!(split_words("HTTPServer"), ["HTTP", "Server"]);
        assert_eq!(split_words("XMLHttpRequest"), ["XML", "Http", "Request"]);
        assert_eq!(
            split_words("already_snake_case"),
            ["already", "snake", "case"]
        );
        assert_eq!(split_words("version2Update"), ["version2", "Update"]);
    }

    #[test]
    fn converts_between_styles() {
        assert_eq!(convert("Hello, world!", CaseStyle::Snake), "hello_world");
        assert_eq!(convert("Hello, world!", CaseStyle::Kebab), "hello-world");
        assert_eq!(convert("Hello, world!", CaseStyle::Constant), "HELLO_WORLD");
        assert_eq!(convert("Hello, world!", CaseStyle::Camel), "helloWorld");
        assert_eq!(convert("Hello, world!", CaseStyle::Pascal), "HelloWorld");
        assert_eq!(convert("Hello, world!", CaseStyle::Title), "Hello World");
        assert_eq!(convert("Hello, world!", CaseStyle::Dot), "hello.world");
        assert_eq!(convert("Hello, world!", CaseStyle::Train), "Hello-World");
    }

    #[test]
    fn parses_style_aliases() {
        assert_eq!(
            CaseStyle::parse("snake_case").map(CaseStyle::name),
            Ok("snake")
        );
        assert_eq!(
            CaseStyle::parse("SCREAMING_SNAKE").map(CaseStyle::name),
            Ok("constant")
        );
        assert!(CaseStyle::parse("unknown").is_err());
    }

    #[test]
    fn sentence_case_capitalizes_sentences() {
        assert_eq!(
            convert("hello there. how ARE you?", CaseStyle::Sentence),
            "Hello there. How are you?"
        );
    }
}
