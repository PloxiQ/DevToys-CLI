use chrono::{DateTime, Duration, Local, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};

use crate::error::{Result, ToolError};

const MILLIS_THRESHOLD: u64 = 100_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpochUnit {
    Seconds,
    Millis,
}

impl EpochUnit {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Seconds => "seconds",
            Self::Millis => "milliseconds",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EpochInfo {
    pub unit: EpochUnit,
    pub seconds: i64,
    pub millis: i64,
    pub utc: DateTime<Utc>,
    pub local: DateTime<Local>,
}

/// Parses a Unix epoch value, auto-detecting milliseconds for large values.
pub fn parse_epoch(input: &str, forced: Option<EpochUnit>) -> Result<EpochInfo> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ToolError::new("No timestamp provided.")
            .with_hint("Pass Unix seconds (1700000000) or milliseconds (1700000000000)."));
    }

    let lower = trimmed.to_ascii_lowercase();
    let (numeric, suffix) = if let Some(value) = lower.strip_suffix("ms") {
        (value, Some(EpochUnit::Millis))
    } else if let Some(value) = lower.strip_suffix('s') {
        (value, Some(EpochUnit::Seconds))
    } else {
        (lower.as_str(), None)
    };

    let value: i64 = numeric.trim().parse().map_err(|_| {
        ToolError::new(format!("`{trimmed}` is not a valid Unix timestamp."))
            .with_hint("Pass a whole number: 1700000000 (seconds) or 1700000000000 (milliseconds).")
    })?;

    let unit = forced.or(suffix).unwrap_or_else(|| {
        if value.unsigned_abs() >= MILLIS_THRESHOLD {
            EpochUnit::Millis
        } else {
            EpochUnit::Seconds
        }
    });

    let (utc, seconds, millis) = match unit {
        EpochUnit::Seconds => {
            let utc = DateTime::from_timestamp(value, 0).ok_or_else(|| out_of_range(value))?;
            let millis = value.checked_mul(1000).ok_or_else(|| out_of_range(value))?;
            (utc, value, millis)
        }
        EpochUnit::Millis => {
            let utc = DateTime::from_timestamp_millis(value).ok_or_else(|| out_of_range(value))?;
            (utc, utc.timestamp(), value)
        }
    };

    Ok(EpochInfo {
        unit,
        seconds,
        millis,
        utc,
        local: utc.with_timezone(&Local),
    })
}

fn out_of_range(value: i64) -> ToolError {
    ToolError::new(format!(
        "Timestamp `{value}` is outside the supported date range."
    ))
    .with_hint("Chrono supports roughly the years 262,000 BC to 262,000 AD.")
}

/// Parses common date formats into a UTC instant.
///
/// Naive values (without an offset) are interpreted as local time unless
/// `naive_as_utc` is set.
pub fn parse_date_to_utc(input: &str, naive_as_utc: bool) -> Result<DateTime<Utc>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ToolError::new("No date provided.").with_hint(
            "Try `2024-01-01`, `2024-01-01 12:30:00`, `2024-01-01T12:30:00Z`, or `now`.",
        ));
    }

    let keyword = trimmed.to_ascii_lowercase();
    let relative_days = match keyword.as_str() {
        "now" => return Ok(Utc::now()),
        "today" => Some(0),
        "yesterday" => Some(-1),
        "tomorrow" => Some(1),
        _ => None,
    };
    if let Some(days) = relative_days {
        let date = Local::now()
            .date_naive()
            .checked_add_signed(Duration::days(days))
            .ok_or_else(|| ToolError::new("The requested date is out of range."))?;
        return local_midnight_to_utc(date);
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Ok(parsed.with_timezone(&Utc));
    }

    const OFFSET_FORMATS: &[&str] = &[
        "%Y-%m-%dT%H:%M:%S%z",
        "%Y-%m-%dT%H:%M:%S%:z",
        "%Y-%m-%d %H:%M:%S%z",
        "%Y-%m-%d %H:%M:%S %z",
        "%Y-%m-%d %H:%M%z",
        "%Y-%m-%d %H:%M %z",
    ];
    for format in OFFSET_FORMATS {
        if let Ok(parsed) = DateTime::parse_from_str(trimmed, format) {
            return Ok(parsed.with_timezone(&Utc));
        }
    }

    if let Some(naive) = parse_naive(trimmed) {
        if naive_as_utc {
            return Ok(Utc.from_utc_datetime(&naive));
        }
        return match Local.from_local_datetime(&naive) {
            LocalResult::Single(value) => Ok(value.with_timezone(&Utc)),
            LocalResult::Ambiguous(_, _) => Err(ToolError::new(
                "This local time is ambiguous because of a DST transition.",
            )
            .with_hint("Add an explicit UTC offset, for example `2024-10-27T02:30:00+02:00`.")),
            LocalResult::None => Err(ToolError::new(
                "This local time does not exist because of a DST transition.",
            )
            .with_hint("Add an explicit UTC offset, for example `2024-03-31T02:30:00+03:00`.")),
        };
    }

    Err(ToolError::new(format!("Could not parse date `{trimmed}`.")).with_hint(
        "Supported: RFC 3339, `YYYY-MM-DD`, `YYYY-MM-DD HH:MM[:SS]`, `DD.MM.YYYY`, and now/today/yesterday/tomorrow.",
    ))
}

fn parse_naive(input: &str) -> Option<NaiveDateTime> {
    const DATETIME_FORMATS: &[&str] = &[
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%d.%m.%Y %H:%M:%S",
        "%d.%m.%Y %H:%M",
    ];
    for format in DATETIME_FORMATS {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(input, format) {
            return Some(parsed);
        }
    }

    const DATE_FORMATS: &[&str] = &["%Y-%m-%d", "%Y/%m/%d", "%d.%m.%Y"];
    for format in DATE_FORMATS {
        if let Ok(date) = NaiveDate::parse_from_str(input, format) {
            return date.and_hms_opt(0, 0, 0);
        }
    }
    None
}

fn local_midnight_to_utc(date: NaiveDate) -> Result<DateTime<Utc>> {
    let naive = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| ToolError::new("Could not build a midnight timestamp."))?;
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(value) => Ok(value.with_timezone(&Utc)),
        LocalResult::Ambiguous(earliest, _) => Ok(earliest.with_timezone(&Utc)),
        LocalResult::None => Err(ToolError::new(
            "Midnight does not exist on this date in your timezone.",
        )),
    }
}

/// Returns a detailed, human-friendly breakdown of an epoch value.
#[must_use]
pub fn describe_epoch(info: &EpochInfo) -> String {
    let human = info.utc.format("%A, %B %d, %Y at %H:%M:%S UTC").to_string();
    let weekday = info.utc.format("%A").to_string();
    format!(
        "Detected unit : {}\nUnix seconds  : {}\nUnix millis   : {}\nUTC           : {}\nLocal         : {}\nHuman (UTC)   : {}\nWeekday       : {}\nRelative      : {}",
        info.unit.name(),
        info.seconds,
        info.millis,
        info.utc.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
        info.local.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, false),
        human,
        weekday,
        relative(info.utc, Utc::now()),
    )
}

/// Returns a detailed breakdown for a UTC date.
#[must_use]
pub fn describe_date(utc: DateTime<Utc>) -> String {
    let local = utc.with_timezone(&Local);
    let human = utc.format("%A, %B %d, %Y at %H:%M:%S UTC").to_string();
    let weekday = utc.format("%A").to_string();
    format!(
        "Unix seconds  : {}\nUnix millis   : {}\nUTC           : {}\nLocal         : {}\nHuman (UTC)   : {}\nWeekday       : {}\nRelative      : {}",
        utc.timestamp(),
        utc.timestamp_millis(),
        utc.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
        local.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, false),
        human,
        weekday,
        relative(utc, Utc::now()),
    )
}

/// A compact relative time such as `in 3 days` or `2 hours ago`.
#[must_use]
pub fn relative(target: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = target.signed_duration_since(now).num_seconds();
    let future = seconds >= 0;
    let absolute = seconds.unsigned_abs();

    let (value, unit) = if absolute < 5 {
        return "just now".to_owned();
    } else if absolute < 60 {
        (absolute, "second")
    } else if absolute < 3_600 {
        (absolute / 60, "minute")
    } else if absolute < 86_400 {
        (absolute / 3_600, "hour")
    } else if absolute < 2_592_000 {
        (absolute / 86_400, "day")
    } else if absolute < 31_536_000 {
        (absolute / 2_592_000, "month")
    } else {
        (absolute / 31_536_000, "year")
    };

    let plural = if value == 1 { "" } else { "s" };
    if future {
        format!("in {value} {unit}{plural}")
    } else {
        format!("{value} {unit}{plural} ago")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        describe_date, describe_epoch, parse_date_to_utc, parse_epoch, relative, EpochUnit,
    };
    use chrono::DateTime;

    #[test]
    fn parses_seconds_and_auto_detects_millis() -> Result<(), Box<dyn std::error::Error>> {
        let seconds = parse_epoch("1700000000", None)?;
        assert_eq!(seconds.unit, EpochUnit::Seconds);
        assert_eq!(seconds.seconds, 1_700_000_000);
        assert_eq!(seconds.millis, 1_700_000_000_000);

        let millis = parse_epoch("1700000000000", None)?;
        assert_eq!(millis.unit, EpochUnit::Millis);
        assert_eq!(millis.seconds, 1_700_000_000);
        Ok(())
    }

    #[test]
    fn accepts_suffixes_and_forced_units() -> Result<(), Box<dyn std::error::Error>> {
        let value = parse_epoch("1700000000s", Some(EpochUnit::Millis))?;
        // An explicit --ms flag wins over the suffix.
        assert_eq!(value.unit, EpochUnit::Millis);
        let value = parse_epoch("-100ms", None)?;
        assert_eq!(value.millis, -100);
        Ok(())
    }

    #[test]
    fn parses_dates_as_utc() -> Result<(), Box<dyn std::error::Error>> {
        let value = parse_date_to_utc("2024-01-01T00:00:00Z", false)?;
        assert_eq!(value.timestamp(), 1_704_067_200);
        let value = parse_date_to_utc("2024-01-01 00:00:00", true)?;
        assert_eq!(value.timestamp(), 1_704_067_200);
        Ok(())
    }

    #[test]
    fn describes_epoch() -> Result<(), Box<dyn std::error::Error>> {
        let value = parse_epoch("0", None)?;
        let text = describe_epoch(&value);
        assert!(text.contains("1970-01-01T00:00:00Z"));
        Ok(())
    }

    #[test]
    fn formats_relative_time() -> Result<(), Box<dyn std::error::Error>> {
        let now = DateTime::from_timestamp(1_700_000_000, 0).ok_or("invalid timestamp")?;
        let target = now + chrono::Duration::days(3);
        assert_eq!(relative(target, now), "in 3 days");
        Ok(())
    }

    #[test]
    fn date_description_works() -> Result<(), Box<dyn std::error::Error>> {
        let utc = DateTime::from_timestamp(0, 0).ok_or("invalid timestamp")?;
        assert!(describe_date(utc).contains("Unix seconds  : 0"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(parse_epoch("", None).is_err());
        assert!(parse_epoch("tomorrow", None).is_err());
        assert!(parse_date_to_utc("not a date", false).is_err());
        assert!(parse_date_to_utc("2024-13-01", true).is_err());
    }
}
