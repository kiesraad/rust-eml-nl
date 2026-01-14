use std::str::FromStr;

use chrono::{
    DateTime, FixedOffset, MappedLocalTime, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc,
};

use crate::utils::StringValueData;

/// Represents an `xs:date`.
///
/// These kinds of dates may optionally contain timezone information using a
/// fixed offset from UTC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XsDate {
    /// The date part of the `xs:date`.
    pub date: NaiveDate,
    /// The optional timezone offset from UTC of the `xs:date`.
    pub tz: Option<FixedOffset>,
}

impl XsDate {
    /// Create a new `XsDate` without timezone information.
    pub fn new(date: NaiveDate) -> Self {
        XsDate { date, tz: None }
    }

    /// Create a new `XsDate` with timezone information.
    pub fn new_with_tz<O: Offset>(date: NaiveDate, tz: O) -> Self {
        XsDate {
            date,
            tz: Some(tz.fix()),
        }
    }
}

impl FromStr for XsDate {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // count the number of '-' and '+' in the string to determine if there's a timezone
        let sep_count = s.chars().filter(|&c| c == '-' || c == '+').count();
        if sep_count > 2
            && let Some(pos) = s.rfind(['+', '-'])
        {
            // The string should be of the form YYYY-MM-DD±HH:MM
            let (date_str, tz_str) = s.split_at(pos);
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
            let tz = tz_str.parse::<FixedOffset>()?;
            Ok(XsDate { date, tz: Some(tz) })
        } else if s.ends_with('Z') || s.ends_with('z') {
            // The string should be of the form YYYY-MM-DDZ
            let date_str = &s[..s.len() - 1];
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;

            Ok(XsDate {
                date,
                tz: Some(Utc.fix()),
            })
        } else {
            // There is no timezone info, just a YYYY-MM-DD date
            let date = NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
            Ok(XsDate { date, tz: None })
        }
    }
}

impl StringValueData for XsDate {
    type Error = chrono::ParseError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        s.parse()
    }

    fn to_raw_value(&self) -> String {
        match self.tz {
            Some(tz) => format!("{}{}", self.date.format("%Y-%m-%d"), tz),
            None => self.date.format("%Y-%m-%d").to_string(),
        }
    }
}

/// Represents an `xs:dateTime`.
///
/// These kinds of date-times may optionally contain timezone information using
/// a fixed offset from UTC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XsDateTime {
    /// The naive date-time. This information does not reflect a specific point
    /// in time without considering timezone information. If a specific point in
    /// time needs to be referenced use the [`Self::datetime_utc`] or [`Self::datetime_tz`]
    /// methods.
    pub naive_date_time: NaiveDateTime,
    /// The timezone offset, if specified. If [`None`], the date-time needs external
    /// context to determine the actual point in time it represents.
    pub tz: Option<FixedOffset>,
}

impl XsDateTime {
    /// Create a new `XsDateTime` from a `DateTime` with timezone information.
    pub fn new<T: TimeZone>(date_time: DateTime<T>) -> Self {
        XsDateTime {
            naive_date_time: date_time.naive_utc(),
            tz: Some(date_time.offset().fix()),
        }
    }

    /// Create a new `XsDateTime` without timezone information.
    pub fn new_without_tz(naive_date_time: NaiveDateTime) -> Self {
        XsDateTime {
            naive_date_time,
            tz: None,
        }
    }

    /// Converts this [`XsDateTime`] to a [`DateTime<Utc>`].
    ///
    /// If the DateTime did not contain timezone information, it is assumed to be in UTC.
    pub fn datetime_utc(&self) -> DateTime<Utc> {
        match self.tz {
            Some(tz) => {
                DateTime::<FixedOffset>::from_naive_utc_and_offset(self.naive_date_time, tz)
                    .to_utc()
            }
            None => DateTime::<Utc>::from_naive_utc_and_offset(self.naive_date_time, Utc),
        }
    }

    /// Converts this [`XsDateTime`] to a [`DateTime`] with the specified timezone.
    ///
    /// If the [`XsDateTime`] did not contain timezone information, it is assumed it was a local time for the specified timezone.
    /// This does mean that some local date-times might be ambiguous or invalid (e.g. during daylight saving time transitions).
    pub fn datetime_tz<Tz: TimeZone>(&self, tz: &Tz) -> MappedLocalTime<DateTime<Tz>> {
        match self.tz {
            Some(original_tz) => MappedLocalTime::Single(
                DateTime::<FixedOffset>::from_naive_utc_and_offset(
                    self.naive_date_time,
                    original_tz,
                )
                .with_timezone(tz),
            ),
            None => tz.from_local_datetime(&self.naive_date_time),
        }
    }
}

impl FromStr for XsDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to parse as RFC3339 first, if that fails, try without timezone info
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            Ok(XsDateTime {
                naive_date_time: dt.naive_utc(),
                tz: Some(dt.offset().to_owned()),
            })
        } else {
            // Fallback to parsing without timezone info
            let naive_dt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")?;
            Ok(XsDateTime {
                naive_date_time: naive_dt,
                tz: None,
            })
        }
    }
}

impl StringValueData for XsDateTime {
    type Error = chrono::ParseError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        s.parse()
    }

    fn to_raw_value(&self) -> String {
        match self.tz {
            Some(tz) => {
                let dt_with_tz =
                    DateTime::<FixedOffset>::from_naive_utc_and_offset(self.naive_date_time, tz);
                dt_with_tz.to_rfc3339()
            }
            None => self
                .naive_date_time
                .format("%Y-%m-%dT%H:%M:%S%.f")
                .to_string(),
        }
    }
}

/// Represents either an `xs:date` or an `xs:dateTime`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XsDateOrDateTime {
    /// An `xs:date` value.
    Date(XsDate),
    /// An `xs:dateTime` value.
    DateTime(XsDateTime),
}

impl XsDateOrDateTime {
    /// Returns the date of the [`XsDate`] or [`XsDateTime`].
    ///
    /// If the value is an [`XsDateTime`] and the timezone is unknown, the date-time is assumed to be
    /// in the specified timezone. In these cases the resulting date may be ambiguous or non-existent.
    pub fn date<Tz: TimeZone>(&self, tz: &Tz) -> MappedLocalTime<NaiveDate> {
        match self {
            // for XsDate, we just return the date and ignore the timezone (no clear way to map a date to a timezone)
            XsDateOrDateTime::Date(d) => MappedLocalTime::Single(d.date),
            // for XsDateTime, we convert to the specified timezone and extract the date
            XsDateOrDateTime::DateTime(dt) => {
                let dt = dt.datetime_tz(tz);
                match dt {
                    MappedLocalTime::Single(dt) => MappedLocalTime::Single(dt.date_naive()),
                    MappedLocalTime::None => MappedLocalTime::None,
                    MappedLocalTime::Ambiguous(first, second) => {
                        // If the first and second are on the same date, the date is not ambiguous for our purposes
                        // DST transitions typically do not happen at midnight, so this is usually the case
                        if first.date_naive() == second.date_naive() {
                            MappedLocalTime::Single(first.date_naive())
                        } else {
                            MappedLocalTime::Ambiguous(first.date_naive(), second.date_naive())
                        }
                    }
                }
            }
        }
    }
}

impl FromStr for XsDateOrDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('T') {
            let date_time = s.parse::<XsDateTime>()?;
            Ok(XsDateOrDateTime::DateTime(date_time))
        } else {
            let date = s.parse::<XsDate>()?;
            Ok(XsDateOrDateTime::Date(date))
        }
    }
}

impl StringValueData for XsDateOrDateTime {
    type Error = chrono::ParseError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        s.parse()
    }

    fn to_raw_value(&self) -> String {
        match self {
            XsDateOrDateTime::Date(d) => d.to_raw_value(),
            XsDateOrDateTime::DateTime(dt) => dt.to_raw_value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike as _, Timelike as _};

    use super::*;

    #[test]
    fn test_xs_date_parse() {
        let d1: XsDate = "2025-10-05".parse().unwrap();
        assert_eq!(d1.date, NaiveDate::from_ymd_opt(2025, 10, 5).unwrap());
        assert!(d1.tz.is_none());

        let d2: XsDate = "2025-10-05+02:00".parse().unwrap();
        assert_eq!(d2.date, NaiveDate::from_ymd_opt(2025, 10, 5).unwrap());
        assert_eq!(d2.tz.unwrap(), FixedOffset::east_opt(2 * 3600).unwrap());

        let d3: XsDate = "2025-10-05Z".parse().unwrap();
        assert_eq!(d3.date, NaiveDate::from_ymd_opt(2025, 10, 5).unwrap());
        assert_eq!(d3.tz.unwrap(), Utc.fix());

        let d4: XsDate = "2025-10-05-05:00".parse().unwrap();
        assert_eq!(d4.date, NaiveDate::from_ymd_opt(2025, 10, 5).unwrap());
        assert_eq!(d4.tz.unwrap(), FixedOffset::west_opt(5 * 3600).unwrap());
    }

    #[test]
    fn test_xs_date_time_parse() {
        let dt1: XsDateTime = "2025-10-05T14:30:00".parse().unwrap();
        assert_eq!(
            dt1.naive_date_time,
            NaiveDate::from_ymd_opt(2025, 10, 5)
                .unwrap()
                .and_hms_opt(14, 30, 0)
                .unwrap()
        );
        assert!(dt1.tz.is_none());

        let dt2: XsDateTime = "2025-10-05T14:30:00+02:00".parse().unwrap();
        assert_eq!(
            dt2.naive_date_time,
            NaiveDate::from_ymd_opt(2025, 10, 5)
                .unwrap()
                .and_hms_opt(12, 30, 0)
                .unwrap()
        );
        assert_eq!(dt2.tz.unwrap(), FixedOffset::east_opt(2 * 3600).unwrap());

        let dt3: XsDateTime = "2025-10-05T14:30:00Z".parse().unwrap();
        assert_eq!(
            dt3.naive_date_time,
            NaiveDate::from_ymd_opt(2025, 10, 5)
                .unwrap()
                .and_hms_opt(14, 30, 0)
                .unwrap()
        );
        assert_eq!(dt3.tz.unwrap(), Utc.fix());

        let dt4: XsDateTime = "2025-10-05T14:30:00.123456".parse().unwrap();
        assert_eq!(
            dt4.naive_date_time,
            NaiveDate::from_ymd_opt(2025, 10, 5)
                .unwrap()
                .and_hms_micro_opt(14, 30, 0, 123456)
                .unwrap()
        );
        assert!(dt4.tz.is_none());

        let dt5: XsDateTime = "2025-10-05T14:30:00.123456-02:00".parse().unwrap();
        assert_eq!(
            dt5.naive_date_time,
            NaiveDate::from_ymd_opt(2025, 10, 5)
                .unwrap()
                .and_hms_micro_opt(16, 30, 0, 123456)
                .unwrap()
        );
        assert_eq!(dt5.tz.unwrap(), FixedOffset::west_opt(2 * 3600).unwrap());
    }

    #[test]
    fn test_xs_date_time_to_datetime_utc() {
        let dt1: XsDateTime = "2025-10-05T14:30:00+02:00".parse().unwrap();
        let utc_dt1 = dt1.datetime_utc();
        assert_eq!(utc_dt1.year(), 2025);
        assert_eq!(utc_dt1.month(), 10);
        assert_eq!(utc_dt1.day(), 5);
        assert_eq!(utc_dt1.hour(), 12);
        assert_eq!(utc_dt1.minute(), 30);

        let dt2: XsDateTime = "2025-10-05T14:30:00".parse().unwrap();
        let utc_dt2 = dt2.datetime_utc();
        assert_eq!(utc_dt2.year(), 2025);
        assert_eq!(utc_dt2.month(), 10);
        assert_eq!(utc_dt2.day(), 5);
        assert_eq!(utc_dt2.hour(), 14);
        assert_eq!(utc_dt2.minute(), 30);
    }

    #[test]
    fn test_xs_date_time_to_datetime_tz() {
        let dt1: XsDateTime = "2025-10-05T14:30:00+02:00".parse().unwrap();
        let tz = FixedOffset::east_opt(3600).unwrap();
        let dt1_in_tz = dt1.datetime_tz(&tz).single().unwrap();
        assert_eq!(dt1_in_tz.year(), 2025);
        assert_eq!(dt1_in_tz.month(), 10);
        assert_eq!(dt1_in_tz.day(), 5);
        assert_eq!(dt1_in_tz.hour(), 13);
        assert_eq!(dt1_in_tz.minute(), 30);

        let dt2: XsDateTime = "2025-10-05T14:30:00".parse().unwrap();
        let dt2_in_tz = dt2.datetime_tz(&tz).single().unwrap();
        assert_eq!(dt2_in_tz.year(), 2025);
        assert_eq!(dt2_in_tz.month(), 10);
        assert_eq!(dt2_in_tz.day(), 5);
        assert_eq!(dt2_in_tz.hour(), 14);
        assert_eq!(dt2_in_tz.minute(), 30);
    }

    #[test]
    fn test_xs_date_or_date_time_parse() {
        let d: XsDateOrDateTime = "2025-10-05".parse().unwrap();
        match d {
            XsDateOrDateTime::Date(date) => {
                assert_eq!(date.date, NaiveDate::from_ymd_opt(2025, 10, 5).unwrap());
                assert!(date.tz.is_none());
            }
            _ => panic!("Expected XsDate variant"),
        }

        let dt: XsDateOrDateTime = "2025-10-05T14:30:00+02:00".parse().unwrap();
        match dt {
            XsDateOrDateTime::DateTime(date_time) => {
                assert_eq!(
                    date_time.naive_date_time,
                    NaiveDate::from_ymd_opt(2025, 10, 5)
                        .unwrap()
                        .and_hms_opt(12, 30, 0)
                        .unwrap()
                );
                assert_eq!(
                    date_time.tz.unwrap(),
                    FixedOffset::east_opt(2 * 3600).unwrap()
                );
            }
            _ => panic!("Expected XsDateTime variant"),
        }
    }
}
