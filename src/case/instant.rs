//! The UTC instant recorded into immutable case evidence.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::TerminalFailure;

const SECONDS_PER_DAY: i64 = 86_400;

/// The accepted `recorded_at` field renders exactly four year digits, so the
/// writer and the reader agree only within this range.
const MIN_YEAR: i64 = 0;
const MAX_YEAR: i64 = 9_999;

/// One UTC instant as recorded in a case event's `recorded_at` field.
///
/// Case recording takes this value as an argument and never reads a clock, so
/// the instant written into immutable evidence is chosen by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordedInstant {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
}

impl RecordedInstant {
    /// Reads the system clock.
    ///
    /// This is the only clock-reading adapter in the crate. Case recording
    /// receives the resulting value; it never calls this itself.
    ///
    /// # Errors
    ///
    /// Fails when the clock reports an instant this field shape cannot record.
    pub fn now() -> Result<Self, TerminalFailure> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| {
                TerminalFailure::unsafe_failure(format!(
                    "system clock cannot supply the case recording timestamp: {error}"
                ))
            })?
            .as_secs();
        let seconds = i64::try_from(seconds).map_err(|error| {
            TerminalFailure::unsafe_failure(format!(
                "case recording timestamp is outside the supported range: {error}"
            ))
        })?;
        Self::from_unix_seconds(seconds)
    }

    /// Derives the instant that many seconds after the Unix epoch.
    ///
    /// # Errors
    ///
    /// Fails when the instant falls outside the four-digit-year range the
    /// accepted `recorded_at` field shape can record.
    pub fn from_unix_seconds(seconds: i64) -> Result<Self, TerminalFailure> {
        let days = seconds.div_euclid(SECONDS_PER_DAY);
        let seconds_of_day = seconds.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = civil_date_from_unix_days(days);
        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            return Err(TerminalFailure::unsafe_failure(format!(
                "case recording timestamp is outside the supported range: year {year} cannot be recorded as a four-digit UTC year"
            )));
        }
        Ok(Self {
            year,
            month,
            day,
            hour: seconds_of_day / 3_600,
            minute: (seconds_of_day % 3_600) / 60,
            second: seconds_of_day % 60,
        })
    }

    /// Reads an instant already recorded in a case event or a prepared proposal.
    pub(super) fn parse(value: &str) -> Result<Self, MalformedInstant> {
        let bytes = value.as_bytes();
        let shaped = bytes.len() == 20
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b'T'
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[19] == b'Z'
            && bytes.iter().enumerate().all(|(index, byte)| {
                matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
            });
        if !shaped {
            return Err(MalformedInstant::NotRfc3339);
        }
        let component = |range: std::ops::Range<usize>| {
            value[range]
                .parse::<i64>()
                .expect("validated ASCII digits should parse")
        };
        let year = component(0..4);
        let month = component(5..7);
        let day = component(8..10);
        let hour = component(11..13);
        let minute = component(14..16);
        let second = component(17..19);
        if day == 0 || day > days_in_month(year, month) || hour > 23 || minute > 59 || second > 59 {
            return Err(MalformedInstant::NotAValidInstant);
        }
        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }
}

/// Why a recorded `recorded_at` value cannot be read as an instant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MalformedInstant {
    /// The value is not the accepted `NNNN-NN-NNTNN:NN:NNZ` shape.
    NotRfc3339,
    /// The value is correctly shaped but names no real UTC instant.
    NotAValidInstant,
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i64) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

impl fmt::Display for RecordedInstant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } = *self;
        write!(
            formatter,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
        )
    }
}

fn civil_date_from_unix_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_piece = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_piece + 2) / 5 + 1;
    let month = month_piece + if month_piece < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExitMeaning;

    fn rendered(seconds: i64) -> String {
        RecordedInstant::from_unix_seconds(seconds)
            .expect("the instant should be representable")
            .to_string()
    }

    #[test]
    fn unix_epoch_seconds_render_the_epoch_instant() {
        assert_eq!(
            RecordedInstant::from_unix_seconds(0)
                .expect("the epoch is a representable instant")
                .to_string(),
            "1970-01-01T00:00:00Z"
        );
    }

    #[test]
    fn a_four_hundred_year_century_keeps_its_leap_day() {
        // A `year % 4 == 0 && year % 100 != 0` rule would deny 2000 a leap day
        // and render this instant as 2000-03-01.
        assert_eq!(rendered(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rendered(951_868_800), "2000-03-01T00:00:00Z");
    }

    #[test]
    fn a_skipped_century_has_no_leap_day() {
        // A bare `year % 4 == 0` rule would grant 1900 a leap day and render
        // this instant as 1900-02-29.
        assert_eq!(rendered(-2_203_891_200), "1900-03-01T00:00:00Z");
        assert_eq!(rendered(-2_208_988_800), "1900-01-01T00:00:00Z");
    }

    #[test]
    fn an_ordinary_leap_year_keeps_its_leap_day() {
        assert_eq!(rendered(1_709_164_800), "2024-02-29T00:00:00Z");
    }

    #[test]
    fn the_last_second_of_a_day_renders_its_time_components() {
        assert_eq!(rendered(SECONDS_PER_DAY - 1), "1970-01-01T23:59:59Z");
    }

    #[test]
    fn the_representable_range_ends_at_the_last_four_digit_year() {
        assert_eq!(rendered(253_402_300_799), "9999-12-31T23:59:59Z");
        let failure = RecordedInstant::from_unix_seconds(253_402_300_800)
            .expect_err("year 10000 cannot be recorded in the accepted field shape");
        assert_eq!(failure.meaning(), ExitMeaning::UnsafeFailure);
    }

    #[test]
    fn every_written_instant_is_read_back_as_the_same_instant() {
        for seconds in [
            0,
            SECONDS_PER_DAY - 1,
            951_782_400,
            951_868_800,
            -2_203_891_200,
            1_709_164_800,
            253_402_300_799,
            -62_167_219_200,
        ] {
            let written =
                RecordedInstant::from_unix_seconds(seconds).expect("the instant is representable");
            let read = RecordedInstant::parse(&written.to_string())
                .expect("the writer must emit only what the reader accepts");
            assert_eq!(read, written, "disagreement at {seconds} seconds");
        }
    }

    #[test]
    fn a_shaped_value_naming_no_real_date_is_refused() {
        // A bare `year % 4 == 0` reader would admit 1900-02-29; a reader
        // missing the four-hundred-year clause would reject 2000-02-29.
        assert_eq!(
            RecordedInstant::parse("1900-02-29T00:00:00Z"),
            Err(MalformedInstant::NotAValidInstant)
        );
        assert_eq!(
            RecordedInstant::parse("2000-02-29T00:00:00Z")
                .expect("2000 is a leap year")
                .to_string(),
            "2000-02-29T00:00:00Z"
        );
        for impossible in [
            "2023-02-29T00:00:00Z",
            "2024-13-01T00:00:00Z",
            "2024-00-01T00:00:00Z",
            "2024-01-00T00:00:00Z",
            "2024-04-31T00:00:00Z",
            "2024-01-01T24:00:00Z",
            "2024-01-01T00:60:00Z",
            "2024-01-01T00:00:60Z",
        ] {
            assert_eq!(
                RecordedInstant::parse(impossible),
                Err(MalformedInstant::NotAValidInstant),
                "{impossible} names no real UTC instant"
            );
        }
    }

    #[test]
    fn a_value_outside_the_accepted_shape_is_refused_before_the_calendar() {
        for misshapen in [
            "",
            "2024-01-01T00:00:00",
            "2024-01-01T00:00:00+00:00",
            "2024-01-01t00:00:00Z",
            "2024-01-01T00:00:00z",
            "2024/01/01T00:00:00Z",
            "24-01-01T00:00:00Z",
            "2024-01-01T00:00:0AZ",
            " 2024-01-01T00:00:00Z",
        ] {
            assert_eq!(
                RecordedInstant::parse(misshapen),
                Err(MalformedInstant::NotRfc3339),
                "{misshapen:?} is not the accepted shape"
            );
        }
    }

    #[test]
    fn the_representable_range_begins_at_the_first_four_digit_year() {
        assert_eq!(rendered(-62_167_219_200), "0000-01-01T00:00:00Z");
        let failure = RecordedInstant::from_unix_seconds(-62_167_219_201)
            .expect_err("a year before 0000 cannot be recorded in the accepted field shape");
        assert_eq!(failure.meaning(), ExitMeaning::UnsafeFailure);
    }
}
