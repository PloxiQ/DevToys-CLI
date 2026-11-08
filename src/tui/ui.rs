use std::time::Duration;

use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;
use unicode_width::UnicodeWidthChar;

use crate::tui::app::{App, Focus, SidebarRow, ToolState, TOOLS};

const BG: Color = Color::Rgb(13, 17, 23);
const PANEL: Color = Color::Rgb(22, 27, 34);
const BORDER: Color = Color::Rgb(48, 54, 61);
const TEXT: Color = Color::Rgb(201, 209, 217);
const MUTED: Color = Color::Rgb(110, 118, 129);
const ACCENT: Color = Color::Rgb(88, 166, 255);
const GREEN: Color = Color::Rgb(63, 185, 80);
const YELLOW: Color = Color::Rgb(210, 153, 34);
const RED: Color = Color::Rgb(248, 81, 73);
const MAGENTA: Color = Color::Rgb(188, 140, 255);

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(BG).fg(TEXT)),
        area,
    );

    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(3),
    ])
    .split(area);

    draw_header(frame, layout[0]);
    draw_body(frame, layout[1], app);
    draw_footer(frame, layout[2], app);

    if app.help_open() {
        draw_help(frame, area);
    }
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let title = Line::from(vec![
        Span::styled(
            "  DEVTOYS",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " CLI",
            Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  offline developer toolbox", Style::default().fg(MUTED)),
    ]);

    let info = Line::from(vec![
        Span::styled(
            format!("  {} tools", TOOLS.len()),
            Style::default().fg(MUTED),
        ),
        Span::styled("  ·  ".to_owned(), Style::default().fg(BORDER)),
        Span::styled("zero network calls", Style::default().fg(MUTED)),
        Span::styled("  ·  ".to_owned(), Style::default().fg(BORDER)),
        Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(MUTED),
        ),
    ]);

    frame.render_widget(Paragraph::new(Text::from(vec![title, info])), inner);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &mut App) {
    let sidebar_width = if area.width < 74 { 26 } else { 32 };
    let chunks = Layout::horizontal([
        Constraint::Length(sidebar_width.min(area.width)),
        Constraint::Min(0),
    ])
    .split(area);

    draw_sidebar(frame, chunks[0], app);
    draw_main(frame, chunks[1], app);
}

fn draw_sidebar(frame: &mut Frame, area: Rect, app: &mut App) {
    let active = app.focus() == Focus::Sidebar;
    let title = Line::from(vec![
        Span::styled(
            " Tools ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{}", TOOLS.len()), Style::default().fg(MUTED)),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if active { ACCENT } else { BORDER }))
        .title(title)
        .style(Style::default().bg(PANEL));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    app.set_sidebar_height(inner.height);
    let width = usize::from(inner.width).saturating_sub(1);
    let selected = app.selected();
    let mut lines = Vec::new();

    for row in app.sidebar_rows() {
        match row {
            SidebarRow::Spacer => lines.push(Line::default()),
            SidebarRow::Header(group) => lines.push(Line::from(Span::styled(
                format!(" {}", group.to_uppercase()),
                Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
            ))),
            SidebarRow::Tool(index) => {
                let tool = TOOLS[*index];
                let is_selected = *index == selected;
                if is_selected {
                    let label = format!(" › {}", tool.name);
                    let padded = format!("{label:<width$}");
                    lines.push(Line::from(Span::styled(
                        padded,
                        Style::default()
                            .fg(BG)
                            .bg(ACCENT)
                            .add_modifier(Modifier::BOLD),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("   {}", tool.name),
                        Style::default().fg(TEXT),
                    )));
                }
            }
        }
    }

    let scroll = u16::try_from(app.sidebar_scroll()).unwrap_or(u16::MAX);
    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), inner);
}

fn draw_main(frame: &mut Frame, area: Rect, app: &App) {
    let tool = app.current_tool();
    let state = app.current_state();
    let secondary_height = if tool.secondary.is_some() { 5 } else { 0 };

    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Percentage(35),
        Constraint::Length(secondary_height),
        Constraint::Min(5),
    ])
    .split(area);

    let title = Line::from(vec![
        Span::styled(
            format!("  {}", tool.name),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}", tool.group.to_uppercase()),
            Style::default().fg(MUTED),
        ),
    ]);
    let description = Line::from(Span::styled(
        format!("  {}", tool.description),
        Style::default().fg(MUTED),
    ));
    frame.render_widget(
        Paragraph::new(Text::from(vec![title, description])),
        chunks[0],
    );

    draw_editor(
        frame,
        chunks[1],
        tool.input.label,
        tool.input.placeholder,
        &state.input,
        state.cursor,
        app.focus() == Focus::Primary,
    );

    if let Some(secondary) = tool.secondary {
        draw_editor(
            frame,
            chunks[2],
            secondary.label,
            secondary.placeholder,
            &state.secondary,
            state.secondary_cursor,
            app.focus() == Focus::Secondary,
        );
    }

    draw_output(frame, chunks[3], state, tool.auto_run);
}

fn draw_editor(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    placeholder: &str,
    input: &str,
    cursor: usize,
    active: bool,
) {
    if area.height < 3 || area.width < 4 {
        return;
    }
    let border = if active { ACCENT } else { BORDER };
    let title = Line::from(Span::styled(
        format!(" {label} "),
        Style::default()
            .fg(if active { ACCENT } else { MUTED })
            .add_modifier(Modifier::BOLD),
    ));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(title)
        .style(Style::default().bg(PANEL));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let lines = if input.is_empty() {
        vec![Line::from(Span::styled(
            placeholder.to_owned(),
            Style::default().fg(MUTED).add_modifier(Modifier::ITALIC),
        ))]
    } else {
        input
            .split('\n')
            .map(|line| {
                Line::from(Span::styled(
                    line.replace('\t', "    "),
                    Style::default().fg(TEXT),
                ))
            })
            .collect()
    };

    if !active {
        frame.render_widget(Paragraph::new(lines), inner);
        return;
    }

    let (caret_line, prefix) = cursor_line(input, cursor);
    let caret_cell = display_width(prefix);
    let inner_height = usize::from(inner.height);
    let inner_width = usize::from(inner.width);
    let vertical = caret_line.saturating_sub(inner_height.saturating_sub(1));
    let horizontal = caret_cell.saturating_sub(inner_width.saturating_sub(1));
    let summary_offset = u16::try_from(vertical).unwrap_or(u16::MAX);
    let horizontal_offset = u16::try_from(horizontal).unwrap_or(u16::MAX);

    frame.render_widget(
        Paragraph::new(lines).scroll((summary_offset, horizontal_offset)),
        inner,
    );

    let x = inner
        .x
        .saturating_add(u16::try_from(caret_cell.saturating_sub(horizontal)).unwrap_or(0));
    let y = inner
        .y
        .saturating_add(u16::try_from(caret_line.saturating_sub(vertical)).unwrap_or(0));
    let x = x.min(inner.right().saturating_sub(1));
    let y = y.min(inner.bottom().saturating_sub(1));
    frame.set_cursor_position(Position::new(x, y));
}

fn draw_output(frame: &mut Frame, area: Rect, state: &ToolState, auto_run: bool) {
    if area.height < 3 || area.width < 4 {
        return;
    }

    let (badge, color) = if state.error.is_some() {
        ("ERROR", RED)
    } else if state.output.is_empty() {
        ("IDLE", MUTED)
    } else {
        ("OK", GREEN)
    };
    let elapsed = format_elapsed(state.elapsed);
    let title = Line::from(vec![
        Span::styled(
            " Output ",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {badge} "),
            Style::default()
                .fg(BG)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {elapsed}"), Style::default().fg(MUTED)),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .title(title)
        .style(Style::default().bg(PANEL));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let lines = if let Some(error) = &state.error {
        let mut lines = vec![
            Line::from(Span::styled(
                "Error",
                Style::default().fg(RED).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(error.clone(), Style::default().fg(TEXT))),
        ];
        if let Some(hint) = &state.hint {
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                format!("Hint: {hint}"),
                Style::default().fg(YELLOW),
            )));
        }
        lines
    } else if state.output.is_empty() {
        let text = if auto_run {
            "This tool runs automatically as you type."
        } else {
            "Press Enter to generate a value."
        };
        vec![Line::from(Span::styled(text, Style::default().fg(MUTED)))]
    } else {
        state
            .output
            .lines()
            .map(|line| Line::from(Span::styled(line.to_owned(), Style::default().fg(TEXT))))
            .collect()
    };

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((state.output_scroll, 0)),
        inner,
    );
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let hints = if app.help_open() {
        " Esc close help   ·   all tools run locally"
    } else {
        match app.focus() {
            Focus::Sidebar => " Up/Down navigate   Enter edit   ? help   q quit",
            Focus::Primary | Focus::Secondary => {
                " Enter run   Tab switch field   Ctrl+J newline   PgUp/PgDn scroll   Esc tools"
            }
        }
    };

    let first = Line::from(Span::styled(
        format!(" {hints}"),
        Style::default().fg(MUTED),
    ));
    let second = Line::from(vec![
        Span::styled(
            format!(" {} tools", TOOLS.len()),
            Style::default().fg(ACCENT),
        ),
        Span::styled("  ·  offline by design", Style::default().fg(MUTED)),
        Span::styled("  ·  MIT", Style::default().fg(MUTED)),
        Span::styled(
            format!("  ·  v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(MUTED),
        ),
    ]);
    frame.render_widget(Paragraph::new(Text::from(vec![first, second])), inner);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let width = area.width.saturating_sub(6).min(68).max(30.min(area.width));
    let height = area.height.saturating_sub(4).min(21);
    if width == 0 || height == 0 {
        return;
    }
    let popup = Rect::new(
        area.x + (area.width.saturating_sub(width)) / 2,
        area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            " DevToys CLI",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " Keyboard-first. Offline. No telemetry.",
            Style::default().fg(MUTED),
        )),
        Line::default(),
        help_line(" Up / Down", "select a tool"),
        help_line(" Enter", "edit input or run the current tool"),
        help_line(" Tab", "switch between input fields"),
        help_line(" Ctrl+J", "insert a newline"),
        help_line(" Ctrl+U / Ctrl+W", "clear the field / delete a word"),
        help_line(" PgUp / PgDn", "scroll the output panel"),
        help_line(" Esc", "return to the tool list"),
        help_line(" ? / F1", "open or close this help"),
        help_line(" Ctrl+C", "quit DevToys CLI"),
        Line::default(),
        Line::from(Span::styled(
            " Try: JSON Format, JWT Decoder, Regex Tester, Cron Explainer.",
            Style::default().fg(TEXT),
        )),
        Line::from(Span::styled(
            " Tip: pipe data in CLI mode, e.g. `echo '{\"a\":1}' | devtoys json format`.",
            Style::default().fg(MUTED),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .title(Line::from(Span::styled(
            " Help ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )))
        .style(Style::default().bg(PANEL));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn help_line(key: &'static str, description: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            key,
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {description}"), Style::default().fg(TEXT)),
    ])
}

fn cursor_line(input: &str, cursor: usize) -> (usize, &str) {
    let before = &input[..cursor.min(input.len())];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let prefix = before.rsplit('\n').next().unwrap_or("");
    (line, prefix)
}

fn display_width(text: &str) -> usize {
    text.chars()
        .map(|character| {
            if character == '\t' {
                4
            } else {
                UnicodeWidthChar::width(character).unwrap_or(0)
            }
        })
        .sum()
}

fn format_elapsed(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{micros} µs")
    } else {
        format!("{:.2} ms", duration.as_secs_f64() * 1_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::draw;
    use crate::tui::app::{App, ToolId};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render(app: &mut App) -> Result<String, Box<dyn std::error::Error>> {
        let backend = TestBackend::new(110, 32);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|frame| draw(frame, app))?;
        let buffer = terminal.backend().buffer();
        Ok(buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>())
    }

    #[test]
    fn renders_the_main_screen() -> Result<(), Box<dyn std::error::Error>> {
        let mut app = App::new();
        let screen = render(&mut app)?;
        assert!(screen.contains("DEVTOYS"));
        assert!(screen.contains("JSON Format"));
        assert!(screen.contains("Output"));
        assert!(screen.contains("offline by design"));
        Ok(())
    }

    #[test]
    fn renders_help_and_secondary_fields() -> Result<(), Box<dyn std::error::Error>> {
        let mut app = App::new();
        app.handle_key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
        let help = render(&mut app)?;
        assert!(help.contains("Keyboard-first"));
        assert!(help.contains("select a tool"));

        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        while app.current_tool().id != ToolId::RegexTest {
            app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        }
        let regex = render(&mut app)?;
        assert!(regex.contains("Regex Tester"));
        assert!(regex.contains("Test string"));
        assert!(regex.contains("Matches: 2"));
        Ok(())
    }
}
