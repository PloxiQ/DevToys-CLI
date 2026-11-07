use std::time::{Duration, Instant};

use chrono::SecondsFormat;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::error::{Result, ToolError};
use crate::tools::hash::{self, Algorithm};
use crate::tools::random::{self, Charset, PasswordOptions};
use crate::tools::{
    base64, cron::CronSchedule, json, jwt, lorem, regex as regex_tool, timestamp, url, uuid,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolId {
    JsonFormat,
    JsonMinify,
    Base64Encode,
    Base64Decode,
    Base64EncodeHex,
    Base64DecodeHex,
    UrlEncode,
    UrlDecode,
    TimestampToDate,
    TimestampToEpoch,
    CronExplain,
    Uuid,
    HashMd5,
    HashSha256,
    HashSha512,
    Lorem,
    RandomString,
    Password,
    JwtDecode,
    RegexTest,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldDef {
    pub label: &'static str,
    pub placeholder: &'static str,
    pub sample: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolDef {
    pub id: ToolId,
    pub name: &'static str,
    pub group: &'static str,
    pub description: &'static str,
    pub input: FieldDef,
    pub secondary: Option<FieldDef>,
    pub auto_run: bool,
}

const fn field(label: &'static str, placeholder: &'static str, sample: &'static str) -> FieldDef {
    FieldDef {
        label,
        placeholder,
        sample,
    }
}

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        id: ToolId::JsonFormat,
        name: "JSON Format",
        group: "Format & Convert",
        description: "Pretty-print JSON and preserve key order.",
        input: field("JSON input", "Paste JSON here...", r#"{"name":"DevToys CLI","stars":1337,"offline":true,"tools":["json","jwt","regex"]}"#),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::JsonMinify,
        name: "JSON Minify",
        group: "Format & Convert",
        description: "Remove insignificant whitespace for compact payloads.",
        input: field("JSON input", "Paste JSON here...", "{\n  \"name\": \"DevToys CLI\",\n  \"offline\": true\n}"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::Base64Encode,
        name: "Base64 Encode",
        group: "Format & Convert",
        description: "Encode UTF-8 text as standard Base64.",
        input: field("Text", "Text to encode...", "Hello, DevToys!"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::Base64Decode,
        name: "Base64 Decode",
        group: "Format & Convert",
        description: "Decode Base64 to UTF-8 text.",
        input: field("Base64", "Base64 to decode...", "SGVsbG8sIERldlRveXMh"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::Base64EncodeHex,
        name: "Base64 Encode HEX",
        group: "Format & Convert",
        description: "Encode raw hex bytes as Base64.",
        input: field("Hex input", "Hex bytes, e.g. 48656c6c6f", "48656c6c6f2c20446576546f797321"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::Base64DecodeHex,
        name: "Base64 Decode HEX",
        group: "Format & Convert",
        description: "Inspect Base64 payload as lowercase hex bytes.",
        input: field("Base64", "Base64 to inspect...", "SGVsbG8sIERldlRveXMh"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::UrlEncode,
        name: "URL Encode",
        group: "Format & Convert",
        description: "Percent-encode text for safe use in URLs.",
        input: field("Text", "Text or URL to encode...", "https://github.com/PloxiQ/DevToys-CLI?q=rust cli&lang=en"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::UrlDecode,
        name: "URL Decode",
        group: "Format & Convert",
        description: "Decode percent escapes; plus becomes a space.",
        input: field("Encoded URL", "Encoded URL to decode...", "https%3A%2F%2Fexample.com%2F%3Fq%3Drust+cli"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::TimestampToDate,
        name: "Timestamp to Date",
        group: "Format & Convert",
        description: "Convert Unix seconds/milliseconds to a full date breakdown.",
        input: field("Unix timestamp", "e.g. 1700000000 or 1700000000000", "1700000000"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::TimestampToEpoch,
        name: "Date to Timestamp",
        group: "Format & Convert",
        description: "Parse a date and show Unix seconds and milliseconds.",
        input: field("Date / time", "e.g. 2024-01-01 12:00:00 or now", "2024-01-01 12:00:00"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::CronExplain,
        name: "Cron Explainer",
        group: "Format & Convert",
        description: "Explain a five-field cron schedule and preview local runs.",
        input: field("Cron expression", "e.g. */15 * * * *", "*/15 * * * *"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::Uuid,
        name: "UUID v4",
        group: "Generate",
        description: "Generate RFC 4122 v4 UUIDs from OS entropy.",
        input: field("How many? (1-1000)", "1", "1"),
        secondary: None,
        auto_run: false,
    },
    ToolDef {
        id: ToolId::HashMd5,
        name: "Hash MD5",
        group: "Generate",
        description: "Compute an MD5 digest (legacy checksums only).",
        input: field("Text to hash", "Text...", "hello world"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::HashSha256,
        name: "Hash SHA-256",
        group: "Generate",
        description: "Compute a SHA-256 digest.",
        input: field("Text to hash", "Text...", "hello world"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::HashSha512,
        name: "Hash SHA-512",
        group: "Generate",
        description: "Compute a SHA-512 digest.",
        input: field("Text to hash", "Text...", "hello world"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::Lorem,
        name: "Lorem Ipsum",
        group: "Generate",
        description: "Generate classic Latin filler paragraphs.",
        input: field("Paragraphs (1-20)", "1", "1"),
        secondary: None,
        auto_run: false,
    },
    ToolDef {
        id: ToolId::RandomString,
        name: "Random String",
        group: "Generate",
        description: "Generate a cryptographically secure alphanumeric string.",
        input: field("Length (1-4096)", "32", "32"),
        secondary: None,
        auto_run: false,
    },
    ToolDef {
        id: ToolId::Password,
        name: "Password",
        group: "Generate",
        description: "Generate a password with every character class.",
        input: field("Length (4-256)", "24", "24"),
        secondary: None,
        auto_run: false,
    },
    ToolDef {
        id: ToolId::JwtDecode,
        name: "JWT Decoder",
        group: "Inspect",
        description: "Decode JWT header, payload and registered claims. No verification.",
        input: field("JWT", "Paste a compact JWT...", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"),
        secondary: None,
        auto_run: true,
    },
    ToolDef {
        id: ToolId::RegexTest,
        name: "Regex Tester",
        group: "Inspect",
        description: "Find captures and preview replacements with Rust regex.",
        input: field("Pattern", "Regular expression...", r"\b\w+@\w+\.\w+\b"),
        secondary: Some(field(
            "Test string",
            "Text to test...",
            "Contact dev@example.com or support@devtoys.dev",
        )),
        auto_run: true,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarRow {
    Spacer,
    Header(&'static str),
    Tool(usize),
}

#[derive(Debug, Clone)]
pub struct ToolState {
    pub input: String,
    pub cursor: usize,
    pub secondary: String,
    pub secondary_cursor: usize,
    pub output: String,
    pub error: Option<String>,
    pub hint: Option<String>,
    pub elapsed: Duration,
    pub output_scroll: u16,
}

impl ToolState {
    fn from_tool(tool: &ToolDef) -> Self {
        Self {
            input: tool.input.sample.to_owned(),
            cursor: tool.input.sample.len(),
            secondary: tool
                .secondary
                .map_or_else(String::new, |field| field.sample.to_owned()),
            secondary_cursor: tool.secondary.map_or(0, |field| field.sample.len()),
            output: String::new(),
            error: None,
            hint: None,
            elapsed: Duration::ZERO,
            output_scroll: 0,
        }
    }
}

pub struct App {
    selected: usize,
    states: Vec<ToolState>,
    focus: Focus,
    help_open: bool,
    sidebar_rows: Vec<SidebarRow>,
    selected_row: usize,
    sidebar_scroll: usize,
    sidebar_height: u16,
}

impl App {
    pub fn new() -> Self {
        let states = TOOLS.iter().map(ToolState::from_tool).collect();
        let sidebar_rows = build_sidebar_rows();
        let selected_row = find_selected_row(&sidebar_rows, 0);
        let mut app = Self {
            selected: 0,
            states,
            focus: Focus::Sidebar,
            help_open: false,
            sidebar_rows,
            selected_row,
            sidebar_scroll: 0,
            sidebar_height: 20,
        };
        app.run_current();
        app
    }

    pub fn current_tool(&self) -> ToolDef {
        TOOLS[self.selected]
    }

    pub fn current_state(&self) -> &ToolState {
        &self.states[self.selected]
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn help_open(&self) -> bool {
        self.help_open
    }

    pub fn sidebar_rows(&self) -> &[SidebarRow] {
        &self.sidebar_rows
    }

    pub fn sidebar_scroll(&self) -> usize {
        self.sidebar_scroll
    }

    pub fn set_sidebar_height(&mut self, height: u16) {
        self.sidebar_height = height;
        self.ensure_selected_visible();
    }

    /// Returns `true` when the application should quit.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            return true;
        }

        if self.help_open {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::F(1)
            ) {
                self.help_open = false;
            }
            return false;
        }

        if key.code == KeyCode::F(1) {
            self.help_open = true;
            return false;
        }

        match self.focus {
            Focus::Sidebar => self.handle_sidebar_key(key),
            Focus::Primary | Focus::Secondary => {
                self.handle_editor_key(key);
                false
            }
        }
    }

    pub fn handle_paste(&mut self, text: &str) {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        if let Some((input, cursor)) = self.active_field_mut() {
            input.insert_str(*cursor, &normalized);
            *cursor += normalized.len();
            self.after_edit();
        }
    }

    fn handle_sidebar_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.select_offset(-1),
            KeyCode::Down | KeyCode::Char('j') => self.select_offset(1),
            KeyCode::PageUp => self.select_offset(-5),
            KeyCode::PageDown => self.select_offset(5),
            KeyCode::Home | KeyCode::Char('g') => self.select_index(0),
            KeyCode::End | KeyCode::Char('G') => self.select_index(TOOLS.len() - 1),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                self.focus = Focus::Primary;
            }
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.help_open = true,
            _ => {}
        }
        false
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Enter {
            self.insert_text("\n");
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('j') => {
                    self.insert_text("\n");
                }
                KeyCode::Char('u') => self.clear_field(),
                KeyCode::Char('w') => self.delete_previous_word(),
                KeyCode::PageUp => self.scroll_output(5),
                KeyCode::PageDown => self.scroll_output(-5),
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc => self.focus = Focus::Sidebar,
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.insert_text("\n");
                } else {
                    self.run_current();
                }
            }
            KeyCode::Char(character) => {
                self.insert_text(&character.to_string());
            }
            KeyCode::Backspace => self.edit_field(|input, cursor| {
                if *cursor > 0 {
                    let previous = previous_char_boundary(input, *cursor);
                    input.replace_range(previous..*cursor, "");
                    *cursor = previous;
                }
            }),
            KeyCode::Delete => self.edit_field(|input, cursor| {
                if *cursor < input.len() {
                    let next = next_char_boundary(input, *cursor);
                    input.replace_range(*cursor..next, "");
                }
            }),
            KeyCode::Left => self.move_cursor(|input, cursor| {
                if *cursor > 0 {
                    *cursor = previous_char_boundary(input, *cursor);
                }
            }),
            KeyCode::Right => self.move_cursor(|input, cursor| {
                if *cursor < input.len() {
                    *cursor = next_char_boundary(input, *cursor);
                }
            }),
            KeyCode::Up => self.move_cursor(|input, cursor| {
                *cursor = vertical_cursor(input, *cursor, true);
            }),
            KeyCode::Down => self.move_cursor(|input, cursor| {
                *cursor = vertical_cursor(input, *cursor, false);
            }),
            KeyCode::Home => self.move_cursor(|input, cursor| {
                *cursor = line_bounds(input, *cursor).0;
            }),
            KeyCode::End => self.move_cursor(|input, cursor| {
                *cursor = line_bounds(input, *cursor).1;
            }),
            KeyCode::PageUp => self.scroll_output(5),
            KeyCode::PageDown => self.scroll_output(-5),
            _ => {}
        }
    }

    fn select_offset(&mut self, offset: isize) {
        let len = TOOLS.len() as isize;
        let next = (self.selected as isize + offset).rem_euclid(len);
        self.select_index(next as usize);
    }

    fn select_index(&mut self, index: usize) {
        if index >= TOOLS.len() || index == self.selected {
            return;
        }
        self.selected = index;
        self.selected_row = find_selected_row(&self.sidebar_rows, index);
        self.run_current();
        self.ensure_selected_visible();
    }

    fn ensure_selected_visible(&mut self) {
        let height = usize::from(self.sidebar_height.max(1));
        if self.selected_row < self.sidebar_scroll {
            self.sidebar_scroll = self.selected_row;
        } else if self.selected_row >= self.sidebar_scroll.saturating_add(height) {
            self.sidebar_scroll = self.selected_row + 1 - height;
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Primary if self.current_tool().secondary.is_some() => Focus::Secondary,
            Focus::Primary => Focus::Sidebar,
            Focus::Secondary => Focus::Primary,
            Focus::Sidebar => Focus::Primary,
        };
    }

    fn run_current(&mut self) {
        let tool = self.current_tool();
        let state = &mut self.states[self.selected];
        let started = Instant::now();
        let result = run_tool(tool.id, &state.input, &state.secondary);
        state.elapsed = started.elapsed();
        state.output_scroll = 0;
        match result {
            Ok(output) => {
                state.output = output;
                state.error = None;
                state.hint = None;
            }
            Err(error) => {
                state.output.clear();
                state.error = Some(error.message().to_owned());
                state.hint = error.hint().map(str::to_owned);
            }
        }
    }

    fn after_edit(&mut self) {
        if self.current_tool().auto_run {
            self.run_current();
        }
    }

    fn active_field_mut(&mut self) -> Option<(&mut String, &mut usize)> {
        let has_secondary = self.current_tool().secondary.is_some();
        match self.focus {
            Focus::Primary => {
                let state = &mut self.states[self.selected];
                Some((&mut state.input, &mut state.cursor))
            }
            Focus::Secondary if has_secondary => {
                let state = &mut self.states[self.selected];
                Some((&mut state.secondary, &mut state.secondary_cursor))
            }
            _ => None,
        }
    }

    fn edit_field<F>(&mut self, edit: F)
    where
        F: FnOnce(&mut String, &mut usize),
    {
        if let Some((input, cursor)) = self.active_field_mut() {
            edit(input, cursor);
        }
        self.after_edit();
    }

    fn move_cursor<F>(&mut self, movement: F)
    where
        F: FnOnce(&str, &mut usize),
    {
        if let Some((input, cursor)) = self.active_field_mut() {
            movement(input, cursor);
        }
    }

    fn insert_text(&mut self, text: &str) {
        self.edit_field(|input, cursor| {
            input.insert_str(*cursor, text);
            *cursor += text.len();
        });
    }

    fn clear_field(&mut self) {
        self.edit_field(|input, cursor| {
            input.clear();
            *cursor = 0;
        });
    }

    fn delete_previous_word(&mut self) {
        self.edit_field(|input, cursor| {
            let mut start = *cursor;
            while start > 0 {
                let previous = previous_char_boundary(input, start);
                let is_space = input[previous..start]
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace);
                if !is_space {
                    break;
                }
                start = previous;
            }
            while start > 0 {
                let previous = previous_char_boundary(input, start);
                let is_space = input[previous..start]
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace);
                if is_space {
                    break;
                }
                start = previous;
            }
            input.replace_range(start..*cursor, "");
            *cursor = start;
        });
    }

    fn scroll_output(&mut self, delta: i16) {
        let state = &mut self.states[self.selected];
        if delta >= 0 {
            state.output_scroll = state.output_scroll.saturating_add(delta as u16);
        } else {
            state.output_scroll = state.output_scroll.saturating_sub(delta.unsigned_abs());
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn build_sidebar_rows() -> Vec<SidebarRow> {
    let mut rows = Vec::new();
    let mut group = "";
    for (index, tool) in TOOLS.iter().enumerate() {
        if tool.group != group {
            if !group.is_empty() {
                rows.push(SidebarRow::Spacer);
            }
            group = tool.group;
            rows.push(SidebarRow::Header(group));
        }
        rows.push(SidebarRow::Tool(index));
    }
    rows
}

fn find_selected_row(rows: &[SidebarRow], tool_index: usize) -> usize {
    rows.iter()
        .position(|row| *row == SidebarRow::Tool(tool_index))
        .unwrap_or(0)
}

fn run_tool(id: ToolId, input: &str, secondary: &str) -> Result<String> {
    match id {
        ToolId::JsonFormat => json::format(input, 2, false),
        ToolId::JsonMinify => json::minify(input),
        ToolId::Base64Encode => Ok(base64::encode_text(input)),
        ToolId::Base64Decode => base64::decode_text(input),
        ToolId::Base64EncodeHex => base64::encode_hex(input),
        ToolId::Base64DecodeHex => base64::decode_hex(input),
        ToolId::UrlEncode => Ok(url::encode(input)),
        ToolId::UrlDecode => url::decode(input),
        ToolId::TimestampToDate => {
            timestamp::parse_epoch(input, None).map(|info| timestamp::describe_epoch(&info))
        }
        ToolId::TimestampToEpoch => {
            timestamp::parse_date_to_utc(input, false).map(timestamp::describe_date)
        }
        ToolId::CronExplain => run_cron(input),
        ToolId::Uuid => {
            let count = number_option(input, 1, 1, uuid::MAX_COUNT, "UUID count")?;
            Ok(uuid::generate(count, false, true)?.join("\n"))
        }
        ToolId::HashMd5 => Ok(hash::hash(Algorithm::Md5, input)),
        ToolId::HashSha256 => Ok(hash::hash(Algorithm::Sha256, input)),
        ToolId::HashSha512 => Ok(hash::hash(Algorithm::Sha512, input)),
        ToolId::Lorem => {
            let paragraphs = number_option(input, 1, 1, 20, "Paragraph count")?;
            lorem::generate(paragraphs, 4, 12)
        }
        ToolId::RandomString => {
            let length = number_option(input, 32, 1, random::MAX_LENGTH, "Length")?;
            Ok(random::generate_strings(length, Charset::Alnum, 1)?.join("\n"))
        }
        ToolId::Password => {
            let length = number_option(input, 20, 4, 256, "Length")?;
            let options = PasswordOptions {
                length,
                ..PasswordOptions::default()
            };
            Ok(random::generate_passwords(options)?.join("\n"))
        }
        ToolId::JwtDecode => run_jwt(input),
        ToolId::RegexTest => run_regex(input, secondary),
    }
}

fn number_option(
    input: &str,
    default: usize,
    min: usize,
    max: usize,
    label: &str,
) -> Result<usize> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(default);
    }
    let value: usize = trimmed.parse().map_err(|_| {
        ToolError::new(format!("{label} must be a whole number."))
            .with_hint(format!("Enter a number between {min} and {max}."))
    })?;
    if !(min..=max).contains(&value) {
        return Err(ToolError::new(format!(
            "{label} must be between {min} and {max}."
        )));
    }
    Ok(value)
}

fn run_cron(input: &str) -> Result<String> {
    let schedule = CronSchedule::parse(input)?;
    let runs = schedule.next_runs(5)?;
    let mut output = format!(
        "Expression : {}\nDescription: {}\n\nNext runs (local):\n",
        schedule.expression(),
        schedule.describe()
    );
    for (index, run) in runs.iter().enumerate() {
        output.push_str(&format!(
            "  {:>2}. {} ({})\n",
            index + 1,
            run.to_rfc3339_opts(SecondsFormat::Secs, false),
            run.format("%a")
        ));
    }
    Ok(output.trim_end().to_owned())
}

fn run_jwt(input: &str) -> Result<String> {
    let decoded = jwt::decode(input)?;
    let mut output = String::new();
    output.push_str("HEADER\n");
    output.push_str(&pretty_json(&decoded.header_raw));
    output.push_str("\n\nPAYLOAD\n");
    output.push_str(&pretty_json(&decoded.payload_raw));
    let claims = decoded.claims();
    if !claims.is_empty() {
        output.push_str("\n\nCLAIMS\n");
        for (key, value) in claims {
            output.push_str(&format!("{key:<5} {value}\n"));
        }
    }
    output.push('\n');
    output.push_str(jwt::SIGNATURE_WARNING);
    output.push('\n');
    output.push_str(&format!(
        "Signature: {} base64url characters.",
        decoded.signature_length
    ));
    Ok(output)
}

fn pretty_json(raw: &str) -> String {
    json::format(raw, 2, false).unwrap_or_else(|_| raw.to_owned())
}

fn run_regex(pattern: &str, text: &str) -> Result<String> {
    let report = regex_tool::test(pattern, text, &regex_tool::RegexOptions::default())?;
    if report.matches.is_empty() {
        let mut output = "No matches found.".to_owned();
        if let Some(replaced) = &report.replaced {
            output.push_str("\n\nReplacement:\n");
            output.push_str(replaced);
        }
        return Ok(output);
    }

    let mut output = format!("Matches: {}\n", report.matches.len());
    for (index, found) in report.matches.iter().enumerate() {
        let (line, column) = regex_tool::line_column(text, found.start);
        output.push_str(&format!(
            "\n[{}] \"{}\"\n    line {line}, column {column}, bytes {}..{}\n",
            index + 1,
            visible_text(&found.text),
            found.start,
            found.end
        ));
        for (group_index, group) in found.groups.iter().enumerate() {
            output.push_str(&format!(
                "    group {}: {}\n",
                group_index + 1,
                group
                    .as_deref()
                    .map_or_else(|| "(no match)".to_owned(), visible_text)
            ));
        }
        for (name, group) in &found.named_groups {
            output.push_str(&format!(
                "    {name}: {}\n",
                group
                    .as_deref()
                    .map_or_else(|| "(no match)".to_owned(), visible_text)
            ));
        }
    }
    if let Some(replaced) = &report.replaced {
        output.push_str("\nReplacement:\n");
        output.push_str(replaced);
    }
    Ok(output.trim_end().to_owned())
}

fn visible_text(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    if escaped.chars().count() > 80 {
        let mut truncated: String = escaped.chars().take(77).collect();
        truncated.push_str("...");
        truncated
    } else {
        escaped
    }
}

fn previous_char_boundary(input: &str, cursor: usize) -> usize {
    input[..cursor]
        .char_indices()
        .next_back()
        .map_or(0, |(index, _)| index)
}

fn next_char_boundary(input: &str, cursor: usize) -> usize {
    input[cursor..]
        .chars()
        .next()
        .map_or(cursor, |character| cursor + character.len_utf8())
}

fn line_bounds(input: &str, cursor: usize) -> (usize, usize) {
    let start = input[..cursor].rfind('\n').map_or(0, |index| index + 1);
    let end = input[cursor..]
        .find('\n')
        .map_or(input.len(), |index| cursor + index);
    (start, end)
}

fn cursor_at_column(input: &str, start: usize, end: usize, column: usize) -> usize {
    let mut position = start;
    for _ in 0..column {
        let Some(character) = input[position..end].chars().next() else {
            break;
        };
        position += character.len_utf8();
    }
    position
}

fn vertical_cursor(input: &str, cursor: usize, up: bool) -> usize {
    let (line_start, line_end) = line_bounds(input, cursor);
    let column = input[line_start..cursor].chars().count();
    if up {
        if line_start == 0 {
            return cursor;
        }
        let previous_end = line_start - 1;
        let previous_start = input[..previous_end]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        cursor_at_column(input, previous_start, previous_end, column)
    } else {
        if line_end >= input.len() {
            return cursor;
        }
        let next_start = line_end + 1;
        let next_end = input[next_start..]
            .find('\n')
            .map_or(input.len(), |index| next_start + index);
        cursor_at_column(input, next_start, next_end, column)
    }
}

#[cfg(test)]
mod tests {
    use super::{App, Focus, ToolId};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn starts_with_json_and_runs_it() {
        let app = App::new();
        assert_eq!(app.current_tool().id, ToolId::JsonFormat);
        assert!(app.current_state().output.contains("\"name\""));
        assert!(app.current_state().error.is_none());
    }

    #[test]
    fn navigates_and_edits_with_auto_run() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.current_tool().id, ToolId::JsonMinify);

        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.focus(), Focus::Primary);
        app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert!(app.current_state().input.is_empty());

        app.handle_paste(r#"{"a": 1}"#);
        let output = &app.current_state().output;
        assert_eq!(output.trim(), r#"{"a":1}"#);
    }

    #[test]
    fn generates_passwords_on_demand() {
        let mut app = App::new();
        app.select_index(17);
        assert_eq!(app.current_tool().id, ToolId::Password);
        assert_eq!(
            app.current_state().output.lines().next().map(str::len),
            Some(24)
        );

        app.handle_key(key(KeyCode::Enter));
        app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        app.handle_key(key(KeyCode::Char('1')));
        app.handle_key(key(KeyCode::Char('2')));
        assert_eq!(app.current_state().input, "12");
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(
            app.current_state().output.lines().next().map(str::len),
            Some(12)
        );
    }

    #[test]
    fn help_and_quit_shortcuts_work() {
        let mut app = App::new();
        assert!(!app.help_open());
        app.handle_key(key(KeyCode::Char('?')));
        assert!(app.help_open());
        app.handle_key(key(KeyCode::Esc));
        assert!(!app.help_open());
        assert!(app.handle_key(key(KeyCode::Char('q'))));
    }

    #[test]
    fn regex_tool_has_two_fields() {
        let mut app = App::new();
        app.select_index(19);
        assert_eq!(app.current_tool().id, ToolId::RegexTest);
        assert!(app.current_tool().secondary.is_some());
        assert!(app.current_state().output.contains("Matches: 2"));
    }
}
