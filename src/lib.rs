use std::fmt;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// --- Platform FFI for local time ---

#[cfg(unix)]
mod platform {
    use super::NanoTime;

    #[repr(C)]
    struct Tm {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        _rest: [i32; 3],
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        _extra: [i64; 2],
    }

    #[repr(C)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }

    const CLOCK_REALTIME: i32 = 0;

    extern "C" {
        fn localtime_r(time: *const i64, result: *mut Tm) -> *mut Tm;
        fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
    }

    pub fn now() -> NanoTime {
        unsafe {
            let mut ts = std::mem::zeroed::<Timespec>();
            if clock_gettime(CLOCK_REALTIME, &mut ts) != 0 {
                return NanoTime {
                    year: 0,
                    month: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    nanosecond: 0,
                };
            }
            let mut tm = std::mem::zeroed::<Tm>();
            let result = localtime_r(&ts.tv_sec, &mut tm);
            if result.is_null() {
                return NanoTime {
                    year: 0,
                    month: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    nanosecond: 0,
                };
            }
            NanoTime {
                year: (tm.tm_year + 1900) as u16,
                month: (tm.tm_mon + 1) as u8,
                day: tm.tm_mday as u8,
                hour: tm.tm_hour as u8,
                minute: tm.tm_min as u8,
                second: tm.tm_sec as u8,
                nanosecond: ts.tv_nsec as u32,
            }
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::NanoTime;

    #[repr(C)]
    struct SYSTEMTIME {
        w_year: u16,
        w_month: u16,
        w_day_of_week: u16,
        w_day: u16,
        w_hour: u16,
        w_minute: u16,
        w_second: u16,
        w_milliseconds: u16,
    }

    extern "system" {
        fn GetLocalTime(lp_system_time: *mut SYSTEMTIME);
    }

    pub fn now() -> NanoTime {
        unsafe {
            let mut st = std::mem::zeroed::<SYSTEMTIME>();
            GetLocalTime(&mut st as *mut SYSTEMTIME);
            NanoTime {
                year: st.w_year,
                month: st.w_month as u8,
                day: st.w_day as u8,
                hour: st.w_hour as u8,
                minute: st.w_minute as u8,
                second: st.w_second as u8,
                nanosecond: st.w_milliseconds as u32 * 1_000_000,
            }
        }
    }
}

fn is_leap_year(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Converts Unix epoch seconds to a NanoTime using Howard Hinnant's civil_from_days algorithm.
fn epoch_to_date(secs: u64) -> NanoTime {
    let day_secs = secs % 86400;
    let hour = (day_secs / 3600) as u8;
    let minute = ((day_secs % 3600) / 60) as u8;
    let second = (day_secs % 60) as u8;

    // Days since 1970-01-01
    let z = (secs / 86400) as i64 + 719468; // shift epoch to 0000-03-01
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month proxy [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    let year = (if month <= 2 { y + 1 } else { y }) as u16;

    NanoTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        nanosecond: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NanoTime {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
}

impl NanoTime {
    pub fn new(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }
        if day < 1 || day > days_in_month(year, month) {
            return None;
        }
        if hour > 23 {
            return None;
        }
        if minute > 59 {
            return None;
        }
        if second > 59 {
            return None;
        }
        if nanosecond > 999_999_999 {
            return None;
        }
        Some(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
        })
    }

    pub fn year(&self) -> u16 {
        self.year
    }
    pub fn month(&self) -> u8 {
        self.month
    }
    pub fn day(&self) -> u8 {
        self.day
    }
    pub fn hour(&self) -> u8 {
        self.hour
    }
    pub fn minute(&self) -> u8 {
        self.minute
    }
    pub fn second(&self) -> u8 {
        self.second
    }
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }

    /// Returns the millisecond component (0–999), derived from nanosecond.
    pub fn millisecond(&self) -> u16 {
        (self.nanosecond / 1_000_000) as u16
    }

    /// Returns the microsecond component (0–999_999), derived from nanosecond.
    pub fn microsecond(&self) -> u32 {
        self.nanosecond / 1_000
    }

    /// Returns current local time via platform FFI.
    pub fn now() -> Self {
        platform::now()
    }

    /// Returns current UTC time via SystemTime + calendar math.
    pub fn now_utc() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        let mut nt = epoch_to_date(secs);
        nt.nanosecond = nanos;
        nt
    }

    /// Constructs a NanoTime from Unix epoch seconds.
    /// Ergonomic wrapper around `epoch_to_date`.
    pub fn from_epoch(secs: u64) -> Self {
        epoch_to_date(secs)
    }

    /// Constructs from total nanoseconds since Unix epoch.
    pub fn from_epoch_nanos(nanos: u128) -> Self {
        let secs = (nanos / 1_000_000_000) as u64;
        let sub_nanos = (nanos % 1_000_000_000) as u32;
        let mut nt = epoch_to_date(secs);
        nt.nanosecond = sub_nanos;
        nt
    }

    /// Constructs from total milliseconds since Unix epoch.
    pub fn from_epoch_ms(ms: u64) -> Self {
        let secs = ms / 1_000;
        let sub_ms = (ms % 1_000) as u32;
        let mut nt = epoch_to_date(secs);
        nt.nanosecond = sub_ms * 1_000_000;
        nt
    }

    /// Constructs from total microseconds since Unix epoch.
    pub fn from_epoch_us(us: u128) -> Self {
        let secs = (us / 1_000_000) as u64;
        let sub_us = (us % 1_000_000) as u32;
        let mut nt = epoch_to_date(secs);
        nt.nanosecond = sub_us * 1_000;
        nt
    }

    /// Returns the signed difference in seconds between self and other.
    /// Positive means self is after other; negative means self is before.
    pub fn diff_secs(&self, other: &NanoTime) -> i64 {
        self.to_epoch_secs() as i64 - other.to_epoch_secs() as i64
    }

    /// Signed difference in nanoseconds (self - other).
    pub fn diff_nanos(&self, other: &NanoTime) -> i128 {
        self.to_epoch_nanos() as i128 - other.to_epoch_nanos() as i128
    }

    /// Signed difference in milliseconds (self - other).
    pub fn diff_ms(&self, other: &NanoTime) -> i64 {
        self.to_epoch_ms() as i64 - other.to_epoch_ms() as i64
    }

    /// Signed difference in microseconds (self - other).
    pub fn diff_us(&self, other: &NanoTime) -> i128 {
        self.to_epoch_us() as i128 - other.to_epoch_us() as i128
    }

    /// Formats as "YYYY-MM-DD".
    pub fn date(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Formats as "YYYY-MM-DD HH:MM:SS.mmm".
    pub fn datetime(&self) -> String {
        format!("{} {}", self.date(), self)
    }

    /// Formats as "YYYY-MM-DD HH:MM:SS" with `precision` fractional digits (0–9).
    /// Values above 9 are clamped to 9.
    pub fn datetime_fmt(&self, precision: u8) -> String {
        let p = precision.min(9) as usize;
        if p == 0 {
            format!(
                "{} {:02}:{:02}:{:02}",
                self.date(),
                self.hour,
                self.minute,
                self.second
            )
        } else {
            let nanos_str = format!("{:09}", self.nanosecond);
            format!(
                "{} {:02}:{:02}:{:02}.{}",
                self.date(),
                self.hour,
                self.minute,
                self.second,
                &nanos_str[..p]
            )
        }
    }

    /// Converts this NanoTime back to Unix epoch seconds.
    /// Reverse of `epoch_to_date` using Hinnant's `days_from_civil` algorithm.
    pub fn to_epoch_secs(&self) -> u64 {
        let y = if self.month <= 2 {
            self.year as i64 - 1
        } else {
            self.year as i64
        };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = (y - era * 400) as u32;
        let m = self.month as u32;
        let d = self.day as u32;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        let days = era * 146097 + doe as i64 - 719468;
        (days as u64) * 86400
            + self.hour as u64 * 3600
            + self.minute as u64 * 60
            + self.second as u64
    }

    /// Returns total nanoseconds since Unix epoch.
    pub fn to_epoch_nanos(&self) -> u128 {
        self.to_epoch_secs() as u128 * 1_000_000_000 + self.nanosecond as u128
    }

    /// Returns total milliseconds since Unix epoch.
    pub fn to_epoch_ms(&self) -> u64 {
        self.to_epoch_secs() * 1_000 + (self.nanosecond / 1_000_000) as u64
    }

    /// Returns total microseconds since Unix epoch.
    pub fn to_epoch_us(&self) -> u128 {
        self.to_epoch_secs() as u128 * 1_000_000 + self.microsecond() as u128
    }

    /// Returns a human-friendly relative time string compared to `other`.
    /// e.g., "3s ago", "2m ago", "in 1h", "just now"
    pub fn relative_to(&self, other: &NanoTime) -> String {
        let self_secs = self.to_epoch_secs();
        let other_secs = other.to_epoch_secs();

        let (diff, past) = if self_secs <= other_secs {
            (other_secs - self_secs, true)
        } else {
            (self_secs - other_secs, false)
        };

        let label = match diff {
            0 => return "just now".to_string(),
            1..=59 => format!("{}s", diff),
            60..=3599 => format!("{}m", diff / 60),
            3600..=86399 => format!("{}h", diff / 3600),
            _ => format!("{}d", diff / 86400),
        };

        if past {
            format!("{} ago", label)
        } else {
            format!("in {}", label)
        }
    }

    /// Returns a human-friendly relative time string compared to now (UTC).
    /// Convenience wrapper around `relative_to`.
    pub fn ago(&self) -> String {
        self.relative_to(&NanoTime::now_utc())
    }
}

impl fmt::Display for NanoTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{:03}",
            self.hour,
            self.minute,
            self.second,
            self.millisecond()
        )
    }
}

pub struct Elapsed {
    start: Instant,
}

impl Elapsed {
    /// Captures the current instant.
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Returns elapsed time in seconds as f64.
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Returns elapsed time in milliseconds as u128.
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    /// Returns elapsed time in microseconds as u128.
    pub fn elapsed_us(&self) -> u128 {
        self.start.elapsed().as_micros()
    }

    /// Returns elapsed time in nanoseconds as u128.
    pub fn elapsed_nanos(&self) -> u128 {
        self.start.elapsed().as_nanos()
    }
}

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ms = self.start.elapsed().as_millis();
        if ms < 1000 {
            write!(f, "{}ms", ms)
        } else {
            let secs = self.start.elapsed().as_secs_f64();
            write!(f, "{:.2}s", secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_zero() {
        let nt = NanoTime::from_epoch(0);
        assert_eq!(nt, NanoTime::new(1970, 1, 1, 0, 0, 0, 0).unwrap());
    }

    #[test]
    fn test_epoch_one_billion() {
        let nt = NanoTime::from_epoch(1_000_000_000);
        assert_eq!(nt, NanoTime::new(2001, 9, 9, 1, 46, 40, 0).unwrap());
    }

    #[test]
    fn test_epoch_leap_year_feb29() {
        let nt = NanoTime::from_epoch(951_782_400);
        assert_eq!(nt.year(), 2000);
        assert_eq!(nt.month(), 2);
        assert_eq!(nt.day(), 29);
    }

    #[test]
    fn test_display_zero_padding() {
        let nt = NanoTime::new(2026, 2, 22, 9, 5, 3, 0).unwrap();
        assert_eq!(format!("{}", nt), "09:05:03.000");
    }

    #[test]
    fn test_date_formatting() {
        let nt = NanoTime::new(2026, 2, 22, 0, 0, 0, 0).unwrap();
        assert_eq!(nt.date(), "2026-02-22");
    }

    #[test]
    fn test_datetime_formatting() {
        let nt = NanoTime::new(2026, 2, 22, 9, 5, 3, 0).unwrap();
        assert_eq!(nt.datetime(), "2026-02-22 09:05:03.000");
    }

    #[test]
    fn test_elapsed_ms_nonnegative() {
        let timer = Elapsed::start();
        let ms = timer.elapsed_ms();
        assert!(
            ms < 1000,
            "elapsed_ms() returned {} immediately after start",
            ms
        );
    }

    #[test]
    fn test_elapsed_secs_nonnegative() {
        let timer = Elapsed::start();
        let s = timer.elapsed_secs();
        assert!(s >= 0.0);
        assert!(
            s < 1.0,
            "elapsed_secs() returned {} immediately after start",
            s
        );
    }

    #[test]
    fn test_elapsed_display_ms_format() {
        let timer = Elapsed::start();
        let display = format!("{}", timer);
        assert!(
            display.ends_with("ms"),
            "expected ms format, got '{}'",
            display
        );
    }

    #[test]
    fn test_to_epoch_secs_zero() {
        let nt = NanoTime::from_epoch(0);
        assert_eq!(nt.to_epoch_secs(), 0);
    }

    #[test]
    fn test_to_epoch_secs_one_billion() {
        let nt = NanoTime::from_epoch(1_000_000_000);
        assert_eq!(nt.to_epoch_secs(), 1_000_000_000);
    }

    #[test]
    fn test_to_epoch_secs_leap_year() {
        let nt = NanoTime::from_epoch(951_782_400);
        assert_eq!(nt.to_epoch_secs(), 951_782_400);
    }

    #[test]
    fn test_now_returns_valid_ranges() {
        let nt = NanoTime::now();
        assert!(nt.hour() <= 23);
        assert!(nt.minute() <= 59);
        assert!(nt.second() <= 59);
        assert!(nt.month() >= 1 && nt.month() <= 12);
        assert!(nt.day() >= 1 && nt.day() <= 31);
        assert!(nt.year() >= 1970);
    }

    #[test]
    fn test_relative_to_just_now() {
        let t = NanoTime::from_epoch(1_000_000);
        assert_eq!(t.relative_to(&t), "just now");
    }

    #[test]
    fn test_relative_to_seconds_ago() {
        let t1 = NanoTime::from_epoch(1_000_000);
        let t2 = NanoTime::from_epoch(1_000_030);
        assert_eq!(t1.relative_to(&t2), "30s ago");
    }

    #[test]
    fn test_relative_to_minutes_ago() {
        let t1 = NanoTime::from_epoch(1_000_000);
        let t2 = NanoTime::from_epoch(1_000_150);
        assert_eq!(t1.relative_to(&t2), "2m ago");
    }

    #[test]
    fn test_relative_to_hours_ago() {
        let t1 = NanoTime::from_epoch(1_000_000);
        let t2 = NanoTime::from_epoch(1_007_200);
        assert_eq!(t1.relative_to(&t2), "2h ago");
    }

    #[test]
    fn test_relative_to_days_ago() {
        let t1 = NanoTime::from_epoch(1_000_000);
        let t2 = NanoTime::from_epoch(1_172_800);
        assert_eq!(t1.relative_to(&t2), "2d ago");
    }

    #[test]
    fn test_relative_to_future() {
        let base = NanoTime::from_epoch(1_000_000);
        assert_eq!(NanoTime::from_epoch(1_000_030).relative_to(&base), "in 30s");
        assert_eq!(NanoTime::from_epoch(1_000_150).relative_to(&base), "in 2m");
        assert_eq!(NanoTime::from_epoch(1_007_200).relative_to(&base), "in 2h");
        assert_eq!(NanoTime::from_epoch(1_172_800).relative_to(&base), "in 2d");
    }

    #[test]
    fn test_relative_to_bucket_boundaries() {
        let base = NanoTime::from_epoch(1_000_000);
        assert_eq!(
            NanoTime::from_epoch(1_000_000 - 59).relative_to(&base),
            "59s ago"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 - 60).relative_to(&base),
            "1m ago"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 - 3599).relative_to(&base),
            "59m ago"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 - 3600).relative_to(&base),
            "1h ago"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 - 86399).relative_to(&base),
            "23h ago"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 - 86400).relative_to(&base),
            "1d ago"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 + 59).relative_to(&base),
            "in 59s"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 + 60).relative_to(&base),
            "in 1m"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 + 3600).relative_to(&base),
            "in 1h"
        );
        assert_eq!(
            NanoTime::from_epoch(1_000_000 + 86400).relative_to(&base),
            "in 1d"
        );
    }

    #[test]
    fn test_from_epoch_zero() {
        let nt = NanoTime::from_epoch(0);
        assert_eq!(nt, NanoTime::new(1970, 1, 1, 0, 0, 0, 0).unwrap());
    }

    #[test]
    fn test_diff_secs_identical() {
        let a = NanoTime::from_epoch(1_000_000);
        assert_eq!(a.diff_secs(&a), 0);
    }

    #[test]
    fn test_diff_secs_known_values() {
        let a = NanoTime::from_epoch(1_000_100);
        let b = NanoTime::from_epoch(1_000_000);
        assert_eq!(a.diff_secs(&b), 100);
        assert_eq!(b.diff_secs(&a), -100);
    }

    // --- NanoTime::new() validation ---

    #[test]
    fn test_new_valid() {
        assert!(NanoTime::new(2026, 2, 22, 14, 30, 0, 0).is_some());
    }

    #[test]
    fn test_new_invalid_month_zero() {
        assert!(NanoTime::new(2026, 0, 1, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn test_new_invalid_month_13() {
        assert!(NanoTime::new(2026, 13, 1, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn test_new_invalid_day_zero() {
        assert!(NanoTime::new(2026, 1, 0, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn test_new_invalid_day_32() {
        assert!(NanoTime::new(2026, 1, 32, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn test_new_feb29_leap_year() {
        assert!(NanoTime::new(2024, 2, 29, 0, 0, 0, 0).is_some());
    }

    #[test]
    fn test_new_feb29_non_leap_year() {
        assert!(NanoTime::new(2025, 2, 29, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn test_new_feb29_century_leap() {
        assert!(NanoTime::new(2000, 2, 29, 0, 0, 0, 0).is_some());
    }

    #[test]
    fn test_new_feb29_century_non_leap() {
        assert!(NanoTime::new(1900, 2, 29, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn test_new_invalid_hour() {
        assert!(NanoTime::new(2026, 1, 1, 24, 0, 0, 0).is_none());
    }

    #[test]
    fn test_new_invalid_minute() {
        assert!(NanoTime::new(2026, 1, 1, 0, 60, 0, 0).is_none());
    }

    #[test]
    fn test_new_invalid_second() {
        assert!(NanoTime::new(2026, 1, 1, 0, 0, 60, 0).is_none());
    }

    #[test]
    fn test_new_invalid_nanosecond() {
        assert!(NanoTime::new(2026, 1, 1, 0, 0, 0, 1_000_000_000).is_none());
    }

    #[test]
    fn test_new_max_valid_nanosecond() {
        assert!(NanoTime::new(2026, 1, 1, 0, 0, 0, 999_999_999).is_some());
    }

    // --- Sub-second accessors ---

    #[test]
    fn test_millisecond() {
        let nt = NanoTime::new(2026, 1, 1, 0, 0, 0, 123_456_789).unwrap();
        assert_eq!(nt.millisecond(), 123);
    }

    #[test]
    fn test_microsecond() {
        let nt = NanoTime::new(2026, 1, 1, 0, 0, 0, 123_456_789).unwrap();
        assert_eq!(nt.microsecond(), 123_456);
    }

    // --- Sub-second epoch constructors ---

    #[test]
    fn test_from_epoch_ms_known() {
        let nt = NanoTime::from_epoch_ms(1_000_000_000_042);
        assert_eq!(nt.to_epoch_secs(), 1_000_000_000);
        assert_eq!(nt.millisecond(), 42);
    }

    #[test]
    fn test_from_epoch_us_known() {
        let nt = NanoTime::from_epoch_us(1_000_000_000_042_000);
        assert_eq!(nt.to_epoch_secs(), 1_000_000_000);
        assert_eq!(nt.microsecond(), 42_000);
    }

    #[test]
    fn test_from_epoch_nanos_known() {
        let nt = NanoTime::from_epoch_nanos(1_000_000_000_123_456_789);
        assert_eq!(nt.to_epoch_secs(), 1_000_000_000);
        assert_eq!(nt.nanosecond(), 123_456_789);
    }

    // --- Sub-second epoch converters ---

    #[test]
    fn test_to_epoch_ms() {
        let nt = NanoTime::new(2001, 9, 9, 1, 46, 40, 42_000_000).unwrap();
        assert_eq!(nt.to_epoch_ms(), 1_000_000_000_042);
    }

    #[test]
    fn test_to_epoch_us() {
        let nt = NanoTime::new(2001, 9, 9, 1, 46, 40, 42_000_000).unwrap();
        assert_eq!(nt.to_epoch_us(), 1_000_000_000_042_000);
    }

    #[test]
    fn test_to_epoch_nanos() {
        let nt = NanoTime::new(2001, 9, 9, 1, 46, 40, 123_456_789).unwrap();
        assert_eq!(nt.to_epoch_nanos(), 1_000_000_000_123_456_789);
    }

    // --- Sub-second diffs ---

    #[test]
    fn test_diff_ms_known() {
        let a = NanoTime::new(2001, 9, 9, 1, 46, 40, 100_000_000).unwrap();
        let b = NanoTime::new(2001, 9, 9, 1, 46, 40, 0).unwrap();
        assert_eq!(a.diff_ms(&b), 100);
        assert_eq!(b.diff_ms(&a), -100);
    }

    #[test]
    fn test_diff_us_known() {
        let a = NanoTime::new(2001, 9, 9, 1, 46, 40, 100_000_000).unwrap();
        let b = NanoTime::new(2001, 9, 9, 1, 46, 40, 0).unwrap();
        assert_eq!(a.diff_us(&b), 100_000);
    }

    #[test]
    fn test_diff_nanos_known() {
        let a = NanoTime::new(2001, 9, 9, 1, 46, 40, 100_000_000).unwrap();
        let b = NanoTime::new(2001, 9, 9, 1, 46, 40, 0).unwrap();
        assert_eq!(a.diff_nanos(&b), 100_000_000);
    }

    // --- datetime_fmt ---

    #[test]
    fn test_datetime_fmt_precision_0() {
        let nt = NanoTime::new(2026, 2, 22, 14, 30, 5, 123_456_789).unwrap();
        assert_eq!(nt.datetime_fmt(0), "2026-02-22 14:30:05");
    }

    #[test]
    fn test_datetime_fmt_precision_3() {
        let nt = NanoTime::new(2026, 2, 22, 14, 30, 5, 123_456_789).unwrap();
        assert_eq!(nt.datetime_fmt(3), "2026-02-22 14:30:05.123");
    }

    #[test]
    fn test_datetime_fmt_precision_6() {
        let nt = NanoTime::new(2026, 2, 22, 14, 30, 5, 123_456_789).unwrap();
        assert_eq!(nt.datetime_fmt(6), "2026-02-22 14:30:05.123456");
    }

    #[test]
    fn test_datetime_fmt_precision_9() {
        let nt = NanoTime::new(2026, 2, 22, 14, 30, 5, 123_456_789).unwrap();
        assert_eq!(nt.datetime_fmt(9), "2026-02-22 14:30:05.123456789");
    }

    #[test]
    fn test_datetime_fmt_precision_clamped() {
        let nt = NanoTime::new(2026, 2, 22, 14, 30, 5, 123_456_789).unwrap();
        assert_eq!(nt.datetime_fmt(15), nt.datetime_fmt(9));
    }

    // --- Ordering ---

    #[test]
    fn test_ordering() {
        let a = NanoTime::new(2026, 1, 1, 0, 0, 0, 0).unwrap();
        let b = NanoTime::new(2026, 1, 1, 0, 0, 1, 0).unwrap();
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_ordering_nanoseconds() {
        let a = NanoTime::new(2026, 1, 1, 0, 0, 0, 100).unwrap();
        let b = NanoTime::new(2026, 1, 1, 0, 0, 0, 200).unwrap();
        assert!(a < b);
    }
}
