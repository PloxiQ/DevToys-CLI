use std::collections::BTreeSet;

use chrono::{DateTime, Datelike, Duration, Local, LocalResult, NaiveDate, TimeZone, Timelike};

use crate::error::{Result, ToolError};

const MAX_LOOKAHEAD_DAYS: u64 = 366 * 30;
const MONTHS: [(&str, u32); 12] = [
    ("JAN", 1),
    ("FEB", 2),
    ("MAR", 3),
    ("APR", 4),
    ("MAY", 5),
    ("JUN", 6),
    ("JUL", 7),
    ("AUG", 8),
    ("SEP", 9),
    ("OCT", 10),
    ("NOV", 11),
    ("DEC", 12),
];
const WEEKDAYS: [(&str, u32); 7] = [
    ("SUN", 0),
    ("MON", 1),
    ("TUE", 2),
    ("WED", 3),
    ("THU", 4),
    ("FRI", 5),
    ("SAT", 6),
];
const MONTH_LABELS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEKDAY_LABELS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

#[derive(Debug, Clone)]
pub struct CronSchedule {
    raw: String,
    minutes: Vec<u32>,
    hours: Vec<u32>,
    days_of_month: Vec<u32>,
    months: Vec<u32>,
    days_of_week: Vec<u32>,
    dom_star: bool,
    dow_star: bool,
}

struct FieldSpec {
    label: &'static str,
    min: u32,
    max: u32,
    names: &'static [(&'static str, u32)],
    normalize_sunday: bool,
}

const MINUTES: FieldSpec = FieldSpec {
    label: "minute",
    min: 0,
    max: 59,
    names: &[],
    normalize_sunday: false,
};
const HOURS: FieldSpec = FieldSpec {
    label: "hour",
    min: 0,
    max: 23,
    names: &[],
    normalize_sunday: false,
};
const DAYS_OF_MONTH: FieldSpec = FieldSpec {
    label: "day of month",
    min: 1,
    max: 31,
    names: &[],
    normalize_sunday: false,
};
const MONTHS_SPEC: FieldSpec = FieldSpec {
    label: "month",
    min: 1,
    max: 12,
    names: &MONTHS,
    normalize_sunday: false,
};
const DAYS_OF_WEEK: FieldSpec = FieldSpec {
    label: "day of week",
    min: 0,
    max: 7,
    names: &WEEKDAYS,
    normalize_sunday: true,
};

impl CronSchedule {
    /// Parses a classic five-field cron expression or a supported `@macro`.
    pub fn parse(expression: &str) -> Result<Self> {
        let raw = expression.trim();
        if raw.is_empty() {
            return Err(ToolError::new("No cron expression provided."))
                .map_err(|error| error.with_hint("Example: `*/15 * * * *` for every 15 minutes."));
        }

        let expanded = match raw.to_ascii_lowercase().as_str() {
            "@yearly" | "@annually" => "0 0 1 1 *",
            "@monthly" => "0 0 1 * *",
            "@weekly" => "0 0 * * 0",
            "@daily" | "@midnight" => "0 0 * * *",
            "@hourly" => "0 * * * *",
            "@reboot" => {
                return Err(
                    ToolError::new("`@reboot` has no time-based schedule.").with_hint(
                        "DevToys explains calendar schedules; use a five-field cron expression.",
                    ),
                );
            }
            _ => raw,
        };

        let fields: Vec<&str> = expanded.split_whitespace().collect();
        if fields.len() != 5 {
            let hint = if fields.len() == 6 {
                "DevToys supports five-field cron (minute hour day-of-month month day-of-week); a seconds field is not supported."
            } else {
                "Example: `*/15 * * * *` or `0 9 * * 1-5`."
            };
            return Err(
                ToolError::new(format!("Expected 5 cron fields, found {}.", fields.len()))
                    .with_hint(hint),
            );
        }

        let (minutes, _) = parse_field(fields[0], &MINUTES)?;
        let (hours, _) = parse_field(fields[1], &HOURS)?;
        let (days_of_month, dom_star) = parse_field(fields[2], &DAYS_OF_MONTH)?;
        let (months, _) = parse_field(fields[3], &MONTHS_SPEC)?;
        let (days_of_week, dow_star) = parse_field(fields[4], &DAYS_OF_WEEK)?;

        Ok(Self {
            raw: raw.to_owned(),
            minutes,
            hours,
            days_of_month,
            months,
            days_of_week,
            dom_star,
            dow_star,
        })
    }

    #[must_use]
    pub fn expression(&self) -> &str {
        &self.raw
    }

    /// A human-readable description of the schedule.
    #[must_use]
    pub fn describe(&self) -> String {
        let time = self.time_phrase();
        let day = self.day_phrase();
        let month = self.month_phrase();
        format!("{time}{day}{month}.")
    }

    /// Returns the next `count` runs using the system's local timezone.
    pub fn next_runs(&self, count: usize) -> Result<Vec<DateTime<Local>>> {
        self.next_runs_from(Local::now(), count)
    }

    /// Returns upcoming runs strictly after `after`.
    pub fn next_runs_from(
        &self,
        after: DateTime<Local>,
        count: usize,
    ) -> Result<Vec<DateTime<Local>>> {
        if count > 100 {
            return Err(ToolError::new(
                "At most 100 upcoming runs can be requested.",
            ));
        }
        if count == 0 {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(count);
        let start_date = after.date_naive();

        for day_offset in 0..=MAX_LOOKAHEAD_DAYS {
            let Some(date) = start_date.checked_add_signed(Duration::days(day_offset as i64))
            else {
                break;
            };
            if !self.matches_date(date) {
                continue;
            }

            for &hour in &self.hours {
                for &minute in &self.minutes {
                    let Some(naive) = date.and_hms_opt(hour, minute, 0) else {
                        continue;
                    };
                    let Some(candidate) = next_local_instant(naive, after) else {
                        continue;
                    };
                    results.push(candidate);
                    if results.len() == count {
                        return Ok(results);
                    }
                }
            }
        }

        Err(
            ToolError::new("No upcoming runs found within the next 30 years.").with_hint(
                "Some schedules, such as February 30, can never run. Check the day/month fields.",
            ),
        )
    }

    /// Tests whether a local datetime matches this schedule (seconds are ignored).
    #[must_use]
    pub fn matches(&self, when: DateTime<Local>) -> bool {
        let naive = when.naive_local();
        self.matches_date(naive.date())
            && self.hours.contains(&naive.hour())
            && self.minutes.contains(&naive.minute())
    }

    fn matches_date(&self, date: NaiveDate) -> bool {
        if !self.months.contains(&date.month()) {
            return false;
        }
        let dom_match = self.days_of_month.contains(&date.day());
        let dow_match = self
            .days_of_week
            .contains(&date.weekday().num_days_from_sunday());

        if self.dom_star || self.dow_star {
            dom_match && dow_match
        } else {
            // Classic Vixie cron semantics: when both day fields are
            // restricted, either field matching is enough.
            dom_match || dow_match
        }
    }

    fn time_phrase(&self) -> String {
        let hours_full = self.hours.len() == 24;
        let minutes_full = self.minutes.len() == 60;

        if minutes_full && hours_full {
            return "Every minute".to_owned();
        }

        if hours_full && self.minutes.first() == Some(&0) {
            if let Some(step) = step_of(&self.minutes) {
                if step > 1 {
                    return format!("Every {step} minutes");
                }
            }
        }

        if self.minutes.len() == 1 {
            let minute = self.minutes[0];
            if hours_full {
                return format!("At minute {minute:02} of every hour");
            }
            if self.hours.len() == 1 {
                return format!("At {:02}:{:02}", self.hours[0], minute);
            }
            return format!(
                "At minute {minute:02} past hours {}",
                compressed_numbers(&self.hours, 2)
            );
        }

        if hours_full {
            return format!(
                "At minutes {} of every hour",
                compressed_numbers(&self.minutes, 2)
            );
        }

        format!(
            "At minutes {} past hours {}",
            compressed_numbers(&self.minutes, 2),
            compressed_numbers(&self.hours, 2)
        )
    }

    fn day_phrase(&self) -> String {
        let dom_full = self.days_of_month.len() == 31;
        let dow_full = self.days_of_week.len() == 7;
        if dom_full && dow_full {
            return String::new();
        }

        if !dom_full && !dow_full {
            let days = compressed_ordinals(&self.days_of_month);
            let weekdays = compressed_weekdays(&self.days_of_week);
            if self.dom_star || self.dow_star {
                return format!(" on the {days} of the month when it is {weekdays}");
            }
            return format!(" on the {days} of the month or on {weekdays}");
        }

        if !dom_full {
            format!(
                " on the {} of the month",
                compressed_ordinals(&self.days_of_month)
            )
        } else {
            format!(" on {}", compressed_weekdays(&self.days_of_week))
        }
    }

    fn month_phrase(&self) -> String {
        if self.months.len() == 12 {
            return String::new();
        }
        let names: Vec<String> = self
            .months
            .iter()
            .map(|month| {
                MONTH_LABELS
                    .get(month.saturating_sub(1) as usize)
                    .copied()
                    .unwrap_or("?")
                    .to_owned()
            })
            .collect();
        format!(" in {}", join_human(names))
    }
}

fn next_local_instant(
    naive: chrono::NaiveDateTime,
    after: DateTime<Local>,
) -> Option<DateTime<Local>> {
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(value) => (value > after).then_some(value),
        LocalResult::Ambiguous(earliest, latest) => {
            if earliest > after {
                Some(earliest)
            } else if latest > after {
                Some(latest)
            } else {
                None
            }
        }
        LocalResult::None => None,
    }
}

fn parse_field(raw: &str, spec: &FieldSpec) -> Result<(Vec<u32>, bool)> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(field_error(spec, raw, "the field is empty"));
    }

    let star = raw.starts_with('*') || raw.starts_with('?');
    let mut values = BTreeSet::new();

    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(field_error(spec, raw, "empty list item"));
        }
        if part.matches('/').count() > 1 {
            return Err(field_error(spec, raw, "too many step separators"));
        }

        let (base, step) = match part.split_once('/') {
            Some((base, step_text)) => {
                let step: u32 = step_text
                    .trim()
                    .parse()
                    .map_err(|_| field_error(spec, raw, &format!("invalid step `{step_text}`")))?;
                if step == 0 {
                    return Err(field_error(spec, raw, "step must be at least 1"));
                }
                (base.trim(), Some(step))
            }
            None => (part, None),
        };

        if base.is_empty() {
            return Err(field_error(spec, raw, "missing value before `/`"));
        }

        if base == "*" || base == "?" {
            for value in (spec.min..=spec.max).step_by(step.unwrap_or(1) as usize) {
                values.insert(normalize(value, spec));
            }
        } else if let Some((start, end)) = base.split_once('-') {
            let start = parse_value(start, spec, raw)?;
            let end = parse_value(end, spec, raw)?;
            if start > end {
                return Err(field_error(
                    spec,
                    raw,
                    &format!("range `{start}-{end}` is descending"),
                ));
            }
            for value in (start..=end).step_by(step.unwrap_or(1) as usize) {
                values.insert(normalize(value, spec));
            }
        } else {
            let start = parse_value(base, spec, raw)?;
            match step {
                Some(step) => {
                    for value in (start..=spec.max).step_by(step as usize) {
                        values.insert(normalize(value, spec));
                    }
                }
                None => {
                    values.insert(normalize(start, spec));
                }
            }
        }
    }

    if values.is_empty() {
        return Err(field_error(spec, raw, "no values matched"));
    }
    Ok((values.into_iter().collect(), star))
}

fn normalize(value: u32, spec: &FieldSpec) -> u32 {
    if spec.normalize_sunday && value == 7 {
        0
    } else {
        value
    }
}

fn parse_value(text: &str, spec: &FieldSpec, raw: &str) -> Result<u32> {
    let text = text.trim();
    if text.is_empty() {
        return Err(field_error(spec, raw, "missing range endpoint"));
    }
    if let Ok(value) = text.parse::<u32>() {
        if value < spec.min || value > spec.max {
            return Err(field_error(
                spec,
                raw,
                &format!("value `{value}` is outside {}-{}", spec.min, spec.max),
            ));
        }
        return Ok(value);
    }
    for (name, value) in spec.names {
        if text.eq_ignore_ascii_case(name) {
            return Ok(*value);
        }
    }
    Err(field_error(spec, raw, &format!("unknown value `{text}`")))
}

fn field_error(spec: &FieldSpec, raw: &str, detail: &str) -> ToolError {
    ToolError::new(format!("Invalid {} field `{raw}`: {detail}.", spec.label))
        .with_hint("Use `*`, lists `1,2`, ranges `1-5`, steps `*/5`, or names like MON/JAN.")
}

fn step_of(values: &[u32]) -> Option<u32> {
    if values.len() < 2 {
        return None;
    }
    let step = values[1].checked_sub(values[0])?;
    if step == 0 {
        return None;
    }
    values
        .windows(2)
        .all(|pair| pair[1].checked_sub(pair[0]) == Some(step))
        .then_some(step)
}

fn compressed_numbers(values: &[u32], width: usize) -> String {
    let parts = compress(values, |value| format!("{value:0width$}"));
    join_human(parts)
}

fn compressed_ordinals(values: &[u32]) -> String {
    let parts = compress(values, ordinal);
    join_human(parts)
}

fn compressed_weekdays(values: &[u32]) -> String {
    let parts = compress(values, |value| {
        WEEKDAY_LABELS
            .get(value as usize)
            .copied()
            .unwrap_or("?")
            .to_owned()
    });
    join_human(parts)
}

fn compress<F>(values: &[u32], render: F) -> Vec<String>
where
    F: Fn(u32) -> String,
{
    let mut parts = Vec::new();
    let mut index = 0usize;
    while index < values.len() {
        let start = values[index];
        let mut end = start;
        let mut next = index + 1;
        while next < values.len() && values[next] == end + 1 {
            end = values[next];
            next += 1;
        }
        let run = next - index;
        if run >= 3 {
            parts.push(format!("{}-{}", render(start), render(end)));
        } else {
            for value in &values[index..next] {
                parts.push(render(*value));
            }
        }
        index = next;
    }
    parts
}

fn ordinal(value: u32) -> String {
    let suffix = match (value % 10, value % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{value}{suffix}")
}

fn join_human(parts: Vec<String>) -> String {
    match parts.len() {
        0 => String::new(),
        1 => parts[0].clone(),
        2 => format!("{} and {}", parts[0], parts[1]),
        _ => {
            let last = parts.len() - 1;
            format!("{}, and {}", parts[..last].join(", "), parts[last])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CronSchedule;
    use chrono::{Datelike, Local, NaiveDate, TimeZone};

    #[test]
    fn parses_and_describes_simple_schedules() -> Result<(), Box<dyn std::error::Error>> {
        let schedule = CronSchedule::parse("*/15 * * * *")?;
        assert_eq!(schedule.describe(), "Every 15 minutes.");
        let schedule = CronSchedule::parse("0 9 * * 1-5")?;
        assert_eq!(schedule.describe(), "At 09:00 on Mon-Fri.");
        let schedule = CronSchedule::parse("@daily")?;
        assert_eq!(schedule.describe(), "At 00:00.");
        Ok(())
    }

    #[test]
    fn parses_names_and_ordinals() -> Result<(), Box<dyn std::error::Error>> {
        let schedule = CronSchedule::parse("0 0 1,15 JAN,MAR *")?;
        let text = schedule.describe();
        assert!(text.contains("1st and 15th"));
        assert!(text.contains("Jan and Mar"));
        Ok(())
    }

    #[test]
    fn computes_next_runs_in_local_time() -> Result<(), Box<dyn std::error::Error>> {
        let schedule = CronSchedule::parse("0 0 * * *")?;
        let after = match Local.with_ymd_and_hms(2026, 1, 1, 12, 0, 0) {
            chrono::LocalResult::Single(value) => value,
            _ => return Err("test time is ambiguous in this timezone".into()),
        };
        let runs = schedule.next_runs_from(after, 2)?;
        assert_eq!(runs.len(), 2);
        assert_eq!(
            runs[0].format("%Y-%m-%d %H:%M").to_string(),
            "2026-01-02 00:00"
        );
        assert_eq!(
            runs[1].format("%Y-%m-%d %H:%M").to_string(),
            "2026-01-03 00:00"
        );
        Ok(())
    }

    #[test]
    fn day_fields_use_or_semantics_when_both_restricted() -> Result<(), Box<dyn std::error::Error>>
    {
        let schedule = CronSchedule::parse("0 0 13 * 5")?;
        let friday = NaiveDate::from_ymd_opt(2026, 11, 6).ok_or("invalid date")?;
        let thirteenth = NaiveDate::from_ymd_opt(2026, 12, 13).ok_or("invalid date")?;
        assert_eq!(friday.weekday(), chrono::Weekday::Fri);
        assert_eq!(thirteenth.day(), 13);
        assert!(schedule.matches_date(friday));
        assert!(schedule.matches_date(thirteenth));
        Ok(())
    }

    #[test]
    fn rejects_invalid_expressions() {
        assert!(CronSchedule::parse("").is_err());
        assert!(CronSchedule::parse("* * * *").is_err());
        assert!(CronSchedule::parse("* * * * * *").is_err());
        assert!(CronSchedule::parse("*/0 * * * *").is_err());
        assert!(CronSchedule::parse("60 * * * *").is_err());
        assert!(CronSchedule::parse("0 0 * * 8").is_err());
        assert!(CronSchedule::parse("5-1 * * * *").is_err());
        assert!(CronSchedule::parse("@reboot").is_err());
    }

    #[test]
    fn impossible_schedule_returns_error() -> Result<(), Box<dyn std::error::Error>> {
        let schedule = CronSchedule::parse("0 0 31 2 *")?;
        let after = match Local.with_ymd_and_hms(2026, 1, 1, 0, 0, 0) {
            chrono::LocalResult::Single(value) => value,
            _ => return Err("test time is ambiguous in this timezone".into()),
        };
        assert!(schedule.next_runs_from(after, 1).is_err());
        Ok(())
    }
}
