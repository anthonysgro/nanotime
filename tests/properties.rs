use nanotime::{Elapsed, NanoTime};
use proptest::prelude::*;

fn arb_nanotime() -> impl Strategy<Value = NanoTime> {
    (
        1970u16..2100,
        1u8..=12,
        1u8..=28,
        0u8..=23,
        0u8..=59,
        0u8..=59,
        0u32..=999_999_999,
    )
        .prop_map(|(year, month, day, hour, minute, second, nanosecond)| {
            NanoTime::new(year, month, day, hour, minute, second, nanosecond).unwrap()
        })
}

proptest! {
    #[test]
    fn epoch_to_date_field_validity(secs in 0u64..=4_102_444_800) {
        let nt = NanoTime::from_epoch(secs);
        prop_assert!(nt.hour() <= 23);
        prop_assert!(nt.minute() <= 59);
        prop_assert!(nt.second() <= 59);
        prop_assert!(nt.month() >= 1 && nt.month() <= 12);
        prop_assert!(nt.day() >= 1 && nt.day() <= 31);
    }

    #[test]
    fn display_format_is_hh_mm_ss_mmm(nt in arb_nanotime()) {
        let s = format!("{}", nt);
        // Format: HH:MM:SS.mmm (12 chars)
        prop_assert_eq!(s.len(), 12);
        prop_assert_eq!(s.as_bytes()[2], b':');
        prop_assert_eq!(s.as_bytes()[5], b':');
        prop_assert_eq!(s.as_bytes()[8], b'.');
        prop_assert!(s.chars().enumerate().all(|(i, c)| i == 2 || i == 5 || i == 8 || c.is_ascii_digit()));
    }

    #[test]
    fn date_format_is_yyyy_mm_dd(nt in arb_nanotime()) {
        let s = nt.date();
        prop_assert!(
            s.len() == 10
                && s.as_bytes()[4] == b'-'
                && s.as_bytes()[7] == b'-'
                && s.chars().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()),
        );
    }

    #[test]
    fn datetime_is_date_plus_display(nt in arb_nanotime()) {
        let expected = format!("{} {}", nt.date(), nt);
        prop_assert_eq!(nt.datetime(), expected);
    }

    #[test]
    fn elapsed_is_monotonic(_ in 0..100u32) {
        let timer = Elapsed::start();
        let first = timer.elapsed_ms();
        let second = timer.elapsed_ms();
        prop_assert!(second >= first);
    }

    #[test]
    fn epoch_round_trip(s in 0u64..=4_102_444_800) {
        let nt = NanoTime::from_epoch(s);
        prop_assert_eq!(nt.to_epoch_secs(), s);
    }

    #[test]
    fn from_epoch_round_trip(s in 0u64..=4_102_444_800) {
        prop_assert_eq!(NanoTime::from_epoch(s).to_epoch_secs(), s);
    }

    #[test]
    fn diff_secs_matches_definition(a_secs in 0u64..=4_102_444_800, b_secs in 0u64..=4_102_444_800) {
        let a = NanoTime::from_epoch(a_secs);
        let b = NanoTime::from_epoch(b_secs);
        let expected = a.to_epoch_secs() as i64 - b.to_epoch_secs() as i64;
        prop_assert_eq!(a.diff_secs(&b), expected);
    }

    #[test]
    fn diff_secs_antisymmetry(a_secs in 0u64..=4_102_444_800, b_secs in 0u64..=4_102_444_800) {
        let a = NanoTime::from_epoch(a_secs);
        let b = NanoTime::from_epoch(b_secs);
        prop_assert_eq!(a.diff_secs(&b), -(b.diff_secs(&a)));
    }

    #[test]
    fn relative_to_bucketing(a in 0u64..=4_102_444_800u64, b in 0u64..=4_102_444_800u64) {
        let nt_a = NanoTime::from_epoch(a);
        let nt_b = NanoTime::from_epoch(b);
        let result = nt_a.relative_to(&nt_b);

        let diff = if a <= b { b - a } else { a - b };
        let past = a <= b;

        match diff {
            0 => prop_assert_eq!(result, "just now"),
            1..=59 => {
                let expected = if past { format!("{}s ago", diff) } else { format!("in {}s", diff) };
                prop_assert_eq!(result, expected);
            }
            60..=3599 => {
                let expected = if past { format!("{}m ago", diff / 60) } else { format!("in {}m", diff / 60) };
                prop_assert_eq!(result, expected);
            }
            3600..=86399 => {
                let expected = if past { format!("{}h ago", diff / 3600) } else { format!("in {}h", diff / 3600) };
                prop_assert_eq!(result, expected);
            }
            _ => {
                let expected = if past { format!("{}d ago", diff / 86400) } else { format!("in {}d", diff / 86400) };
                prop_assert_eq!(result, expected);
            }
        }
    }

    /// Nanosecond field is always in [0, 999_999_999] regardless of constructor.
    #[test]
    fn nanosecond_field_range_invariant(
        secs in 0u64..=4_102_444_800u64,
        nanos in 0u128..=(4_102_444_800u128 * 1_000_000_000 + 999_999_999),
        ms in 0u64..=(4_102_444_800u64 * 1_000 + 999),
        us in 0u128..=(4_102_444_800u128 * 1_000_000 + 999_999),
    ) {
        prop_assert!(NanoTime::from_epoch(secs).nanosecond() <= 999_999_999);
        prop_assert!(NanoTime::from_epoch_nanos(nanos).nanosecond() <= 999_999_999);
        prop_assert!(NanoTime::from_epoch_ms(ms).nanosecond() <= 999_999_999);
        prop_assert!(NanoTime::from_epoch_us(us).nanosecond() <= 999_999_999);
    }

    /// millisecond() and microsecond() are derived correctly from nanosecond().
    #[test]
    fn accessor_correctness(n in 0u32..=999_999_999) {
        let nt = NanoTime::new(2000, 1, 1, 0, 0, 0, n).unwrap();
        prop_assert_eq!(nt.millisecond(), (n / 1_000_000) as u16);
        prop_assert_eq!(nt.microsecond(), n / 1_000);
    }

    /// from_epoch() always produces nanosecond == 0.
    #[test]
    fn epoch_secs_constructors_set_nanosecond_zero(s in 0u64..=4_102_444_800) {
        prop_assert_eq!(NanoTime::from_epoch(s).nanosecond(), 0);
        prop_assert_eq!(NanoTime::from_epoch(s).nanosecond(), 0);
    }

    /// from_epoch_nanos(to_epoch_nanos(nt)) == nt.
    #[test]
    fn nanosecond_epoch_round_trip(nt in arb_nanotime()) {
        let round_tripped = NanoTime::from_epoch_nanos(nt.to_epoch_nanos());
        prop_assert_eq!(round_tripped, nt);
    }

    /// Millisecond round-trip truncates sub-millisecond precision.
    #[test]
    fn millisecond_epoch_round_trip(nt in arb_nanotime()) {
        let round_tripped = NanoTime::from_epoch_ms(nt.to_epoch_ms());
        let expected = NanoTime::new(
            nt.year(), nt.month(), nt.day(), nt.hour(), nt.minute(), nt.second(),
            (nt.nanosecond() / 1_000_000) * 1_000_000,
        ).unwrap();
        prop_assert_eq!(round_tripped, expected);
    }

    /// Microsecond round-trip truncates sub-microsecond precision.
    #[test]
    fn microsecond_epoch_round_trip(nt in arb_nanotime()) {
        let round_tripped = NanoTime::from_epoch_us(nt.to_epoch_us());
        let expected = NanoTime::new(
            nt.year(), nt.month(), nt.day(), nt.hour(), nt.minute(), nt.second(),
            (nt.nanosecond() / 1_000) * 1_000,
        ).unwrap();
        prop_assert_eq!(round_tripped, expected);
    }
}

proptest! {
    /// Display is HH:MM:SS.mmm; datetime() is date() + " " + Display.
    #[test]
    fn display_and_datetime_formatting(nt in arb_nanotime()) {
        let display = format!("{}", nt);
        let expected_ms = format!("{:03}", nt.millisecond());
        // Display format: HH:MM:SS.mmm
        let parts: Vec<&str> = display.split('.').collect();
        prop_assert_eq!(parts.len(), 2);
        prop_assert_eq!(parts[1], expected_ms.as_str());

        // datetime format: YYYY-MM-DD HH:MM:SS.mmm
        let dt = nt.datetime();
        let expected_dt = format!("{} {}", nt.date(), display);
        prop_assert_eq!(&dt, &expected_dt);
        // Verify the mmm portion in datetime
        let dt_parts: Vec<&str> = dt.split('.').collect();
        prop_assert_eq!(dt_parts.len(), 2);
        prop_assert_eq!(dt_parts[1], expected_ms.as_str());
    }

    /// datetime_fmt(p) produces the correct fractional digits, clamped at 9.
    #[test]
    fn datetime_fmt_correctness(nt in arb_nanotime(), precision in 0u8..=15) {
        let result = nt.datetime_fmt(precision);
        let nanos_str = format!("{:09}", nt.nanosecond());
        let p = precision.min(9) as usize;

        if p == 0 {
            let expected = format!("{} {:02}:{:02}:{:02}", nt.date(), nt.hour(), nt.minute(), nt.second());
            prop_assert_eq!(&result, &expected);
            prop_assert!(!result.contains('.'));
        } else {
            let expected = format!("{} {:02}:{:02}:{:02}.{}", nt.date(), nt.hour(), nt.minute(), nt.second(), &nanos_str[..p]);
            prop_assert_eq!(&result, &expected);
        }

        // Values > 9 should produce same output as precision 9
        if precision > 9 {
            prop_assert_eq!(result, nt.datetime_fmt(9));
        }
    }

    /// diff_nanos, diff_ms, diff_us equal the difference of the corresponding epoch conversions.
    #[test]
    fn sub_second_diff_methods(a in arb_nanotime(), b in arb_nanotime()) {
        prop_assert_eq!(a.diff_nanos(&b), a.to_epoch_nanos() as i128 - b.to_epoch_nanos() as i128);
        prop_assert_eq!(a.diff_ms(&b), a.to_epoch_ms() as i64 - b.to_epoch_ms() as i64);
        prop_assert_eq!(a.diff_us(&b), a.to_epoch_us() as i128 - b.to_epoch_us() as i128);
    }

    /// to_epoch_secs and diff_secs ignore the nanosecond field.
    #[test]
    fn second_granularity_truncates_nanoseconds(
        nt in arb_nanotime(),
        n1 in 0u32..=999_999_999u32,
        n2 in 0u32..=999_999_999u32,
    ) {
        let a = NanoTime::new(nt.year(), nt.month(), nt.day(), nt.hour(), nt.minute(), nt.second(), n1).unwrap();
        let b = NanoTime::new(nt.year(), nt.month(), nt.day(), nt.hour(), nt.minute(), nt.second(), n2).unwrap();
        // to_epoch_secs ignores nanosecond field
        prop_assert_eq!(a.to_epoch_secs(), b.to_epoch_secs());
        // diff_secs between two NanoTimes differing only in nanosecond should be 0
        prop_assert_eq!(a.diff_secs(&b), 0);
    }
}

/// Helper: days in month (mirrors private helper in lib.rs)
fn test_days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

proptest! {
    /// NanoTime::new followed by getters returns the original values.
    #[test]
    fn constructor_getter_round_trip(
        year in 0u16..=9999,
        month in 1u8..=12,
        day_offset in 0u8..=27,
        hour in 0u8..=23,
        minute in 0u8..=59,
        second in 0u8..=59,
        nanosecond in 0u32..=999_999_999,
    ) {
        let max_day = test_days_in_month(year, month);
        let day = (day_offset % max_day) + 1;
        let result = NanoTime::new(year, month, day, hour, minute, second, nanosecond);
        prop_assert!(result.is_some());
        let nt = result.unwrap();
        prop_assert_eq!(nt.year(), year);
        prop_assert_eq!(nt.month(), month);
        prop_assert_eq!(nt.day(), day);
        prop_assert_eq!(nt.hour(), hour);
        prop_assert_eq!(nt.minute(), minute);
        prop_assert_eq!(nt.second(), second);
        prop_assert_eq!(nt.nanosecond(), nanosecond);
        prop_assert_eq!(nt.millisecond(), (nanosecond / 1_000_000) as u16);
        prop_assert_eq!(nt.microsecond(), nanosecond / 1_000);
    }

    /// NanoTime::new returns None when any field is out of range.
    #[test]
    fn invalid_input_rejection(
        year in 0u16..=9999,
        month in 0u8..=255u8,
        day in 0u8..=255u8,
        hour in 0u8..=255u8,
        minute in 0u8..=255u8,
        second in 0u8..=255u8,
        nanosecond in 0u32..=u32::MAX,
    ) {
        // At least one field must be invalid for this test
        let month_invalid = month == 0 || month > 12;
        let day_invalid = if month >= 1 && month <= 12 {
            day == 0 || day > test_days_in_month(year, month)
        } else {
            false // can't check day validity if month is already invalid
        };
        let hour_invalid = hour > 23;
        let minute_invalid = minute > 59;
        let second_invalid = second > 59;
        let nano_invalid = nanosecond > 999_999_999;

        let any_invalid = month_invalid || day_invalid || hour_invalid || minute_invalid || second_invalid || nano_invalid;

        // Only assert None when we know at least one field is invalid
        prop_assume!(any_invalid);
        prop_assert!(NanoTime::new(year, month, day, hour, minute, second, nanosecond).is_none());
    }
}
