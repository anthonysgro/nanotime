# ⏱ nanotime

[![Crates.io](https://img.shields.io/crates/v/nanotime)](https://crates.io/crates/nanotime)
[![Docs.rs](https://docs.rs/nanotime/badge.svg)](https://docs.rs/nanotime/latest/nanotime/)

A minimal, zero-dependency time utility crate for Rust CLI applications.

Part of the nano crate family — minimal, zero-dependency building blocks for CLI apps in Rust:

- [nanocolor](https://github.com/anthonysgro/nanocolor) — terminal colors and styles
- [nanospinner](https://github.com/anthonysgro/nanospinner) — terminal spinners
- [nanoprogress](https://github.com/anthonysgro/nanoprogress) — progress bars
- [nanolog](https://github.com/anthonysgro/nanolog) — minimal logger
- [nanotime](https://github.com/anthonysgro/nanotime) — time utilities

Local and UTC time retrieval, nanosecond-precision timestamps, human-readable formatting, relative time strings, and lightweight elapsed duration measurement — no heavy crates, no transitive dependencies.

## Motivation

Most Rust time crates (like `chrono` or `time`) are feature-rich but pull in dependencies or offer far more than you need. If all you want is the current time and a simple way to measure durations, those crates are overkill.

`nanotime` solves this by providing the essentials and nothing more:

- Zero external dependencies (only `std` + raw FFI)
- Nanosecond-precision timestamps with millisecond and microsecond accessors
- Local time via platform FFI (`clock_gettime` on Unix, `GetLocalTime` on Windows)
- UTC time via `SystemTime` + manual calendar arithmetic
- Elapsed duration measurement via `std::time::Instant`
- Human-readable formatting out of the box
- Validated constructor — no invalid dates
- Implements `Ord`, `Hash`, `Eq`, `Clone`, `Copy`, `Debug`

## Comparison

nanotime is intentionally minimal. If you need timezones, parsing, arithmetic, or `no_std`, use `chrono` or `time` — they're great crates.

nanotime is for when you just want to know what time it is and how long something took.

| Feature | `nanotime` | `chrono` | `time` |
|---------|:----------:|:--------:|:------:|
| Zero dependencies | ✓ | ✗ | ✗ |
| Current local time | ✓ | ✓ | ✓ |
| Single file | ✓ | ✗ | ✗ |
| Clean build (release) | ~0.12s | ~1.6s | ~1.6s |
| Current UTC time | ✓ | ✓ | ✓ |
| Nanosecond precision | ✓ | ✓ | ✓ |
| Elapsed measurement | ✓ | ✗ | ✗ |
| Relative time ("3s ago") | ✓ | ✓ | ✓ |
| Human-readable display | ✓ | ✓ | ✓ |
| Timezone conversion | ✗ | ✓ | ✓ |
| Date/time parsing | ✗ | ✓ | ✓ |
| Date/time arithmetic | ✗ | ✓ | ✓ |
| `no_std` support | ✗ | ✗ | ✓ |

## Quick Start

```toml
[dependencies]
nanotime = "0.1.0"
```

```rust
use nanotime::{NanoTime, Elapsed};

fn main() {
    let now = NanoTime::now();
    println!("Local time: {}", now);            // "14:32:07.042"
    println!("Local datetime: {}", now.datetime()); // "2026-02-22 14:32:07.042"

    let utc = NanoTime::now_utc();
    println!("UTC: {}", utc.datetime());

    // Relative time
    let past = NanoTime::new(2025, 1, 1, 0, 0, 0, 0).unwrap();
    println!("{}", past.ago()); // e.g. "419d ago"

    // Elapsed timer
    let timer = Elapsed::start();
    // ... do some work ...
    println!("Took {}", timer); // "42ms" or "1.23s"
}
```

## Usage

### Get the current time

```rust
use nanotime::NanoTime;

let local = NanoTime::now();       // local time via platform FFI
let utc = NanoTime::now_utc();     // UTC via SystemTime

println!("{}", local);             // "14:32:07.042"
println!("{}", local.date());      // "2026-02-22"
println!("{}", local.datetime());  // "2026-02-22 14:32:07.042"
```

### Create timestamps

```rust
use nanotime::NanoTime;

// Validated constructor (returns None for invalid dates)
let t = NanoTime::new(2026, 2, 22, 14, 30, 0, 500_000_000).unwrap();

// From epoch values
let from_secs = NanoTime::from_epoch(1_000_000_000);
let from_ms = NanoTime::from_epoch_ms(1_000_000_000_042);
let from_us = NanoTime::from_epoch_us(1_000_000_000_042_000);
let from_ns = NanoTime::from_epoch_nanos(1_000_000_000_042_000_000);

// Invalid dates return None
assert!(NanoTime::new(2025, 2, 29, 0, 0, 0, 0).is_none()); // not a leap year
assert!(NanoTime::new(2026, 13, 1, 0, 0, 0, 0).is_none());  // invalid month
```

### Access fields

```rust
use nanotime::NanoTime;

let now = NanoTime::now();
println!("Year: {}, Month: {}, Day: {}", now.year(), now.month(), now.day());
println!("{}:{}:{}", now.hour(), now.minute(), now.second());

// Sub-second accessors (derived from nanosecond field)
println!("Nanoseconds: {}", now.nanosecond());   // 0–999,999,999
println!("Microseconds: {}", now.microsecond());  // 0–999,999
println!("Milliseconds: {}", now.millisecond());  // 0–999
```

### Formatting

```rust
use nanotime::NanoTime;

let t = NanoTime::new(2026, 2, 22, 14, 30, 5, 123_456_789).unwrap();

// Default formatting includes milliseconds
println!("{}", t);              // "14:30:05.123"
println!("{}", t.date());       // "2026-02-22"
println!("{}", t.datetime());   // "2026-02-22 14:30:05.123"

// Flexible precision with datetime_fmt (0–9 fractional digits)
println!("{}", t.datetime_fmt(0)); // "2026-02-22 14:30:05"
println!("{}", t.datetime_fmt(3)); // "2026-02-22 14:30:05.123"
println!("{}", t.datetime_fmt(6)); // "2026-02-22 14:30:05.123456"
println!("{}", t.datetime_fmt(9)); // "2026-02-22 14:30:05.123456789"
```

### Epoch conversions

```rust
use nanotime::NanoTime;

let t = NanoTime::new(2026, 2, 22, 14, 30, 0, 500_000_000).unwrap();

// Convert to epoch at any granularity
let secs = t.to_epoch_secs();    // u64
let ms = t.to_epoch_ms();        // u64
let us = t.to_epoch_us();        // u128
let nanos = t.to_epoch_nanos();  // u128
```

### Time differences

```rust
use nanotime::NanoTime;

let a = NanoTime::from_epoch(1_000_100);
let b = NanoTime::from_epoch(1_000_000);

// At any granularity
println!("{}", a.diff_secs(&b));   //  100
println!("{}", a.diff_ms(&b));     //  100000
println!("{}", a.diff_us(&b));     //  100000000
println!("{}", a.diff_nanos(&b));  //  100000000000
```

### Relative time

```rust
use nanotime::NanoTime;

let past = NanoTime::new(2025, 1, 1, 0, 0, 0, 0).unwrap();
println!("{}", past.ago()); // e.g. "419d ago"

let a = NanoTime::new(2026, 2, 22, 12, 0, 0, 0).unwrap();
let b = NanoTime::new(2026, 2, 22, 12, 5, 30, 0).unwrap();
println!("{}", a.relative_to(&b)); // "5m ago"
println!("{}", b.relative_to(&a)); // "in 5m"
```

Buckets: `just now`, `Xs`, `Xm`, `Xh`, `Xd` — with "ago" or "in" prefix/suffix for direction.

### Measure elapsed time

```rust
use nanotime::Elapsed;

let timer = Elapsed::start();

// ... do some work ...

println!("Elapsed: {}", timer);                   // "42ms" or "1.23s"
println!("Seconds: {:.3}", timer.elapsed_secs()); // "0.042"
println!("Milliseconds: {}", timer.elapsed_ms()); // "42"
```

The `Display` impl automatically picks the right unit:
- Under 1 second: `Xms` (e.g. `450ms`)
- 1 second or more: `X.XXs` (e.g. `1.23s`)

## API Reference

### `NanoTime`

#### Constructors

| Method | Returns | Description |
|--------|---------|-------------|
| `NanoTime::new(year, month, day, hour, minute, second, nanosecond)` | `Option<NanoTime>` | Validated constructor. Returns `None` for invalid dates. |
| `NanoTime::now()` | `NanoTime` | Current local time via platform FFI |
| `NanoTime::now_utc()` | `NanoTime` | Current UTC time via `SystemTime` |
| `NanoTime::from_epoch(secs)` | `NanoTime` | From Unix epoch seconds |
| `NanoTime::from_epoch_ms(ms)` | `NanoTime` | From Unix epoch milliseconds |
| `NanoTime::from_epoch_us(us)` | `NanoTime` | From Unix epoch microseconds |
| `NanoTime::from_epoch_nanos(nanos)` | `NanoTime` | From Unix epoch nanoseconds |

#### Getters

| Method | Returns | Description |
|--------|---------|-------------|
| `.year()` | `u16` | Calendar year |
| `.month()` | `u8` | Month (1–12) |
| `.day()` | `u8` | Day of month (1–31) |
| `.hour()` | `u8` | Hour (0–23) |
| `.minute()` | `u8` | Minute (0–59) |
| `.second()` | `u8` | Second (0–59) |
| `.nanosecond()` | `u32` | Nanosecond (0–999,999,999) |
| `.millisecond()` | `u16` | Derived: nanosecond / 1,000,000 |
| `.microsecond()` | `u32` | Derived: nanosecond / 1,000 |

#### Formatting

| Method | Returns | Example |
|--------|---------|---------|
| `Display` (format!) | — | `14:30:05.123` |
| `.date()` | `String` | `2026-02-22` |
| `.datetime()` | `String` | `2026-02-22 14:30:05.123` |
| `.datetime_fmt(precision)` | `String` | `2026-02-22 14:30:05.123456` (precision=6) |

#### Epoch Conversions

| Method | Returns | Description |
|--------|---------|-------------|
| `.to_epoch_secs()` | `u64` | Unix epoch seconds |
| `.to_epoch_ms()` | `u64` | Unix epoch milliseconds |
| `.to_epoch_us()` | `u128` | Unix epoch microseconds |
| `.to_epoch_nanos()` | `u128` | Unix epoch nanoseconds |

#### Differences

| Method | Returns | Description |
|--------|---------|-------------|
| `.diff_secs(&other)` | `i64` | Signed difference in seconds |
| `.diff_ms(&other)` | `i64` | Signed difference in milliseconds |
| `.diff_us(&other)` | `i128` | Signed difference in microseconds |
| `.diff_nanos(&other)` | `i128` | Signed difference in nanoseconds |

#### Relative Time

| Method | Returns | Description |
|--------|---------|-------------|
| `.relative_to(&other)` | `String` | e.g. "3s ago", "in 2h" |
| `.ago()` | `String` | Relative to now (UTC) |

### `Elapsed`

| Method | Returns | Description |
|--------|---------|-------------|
| `Elapsed::start()` | `Elapsed` | Capture current instant |
| `.elapsed_secs()` | `f64` | Elapsed seconds |
| `.elapsed_ms()` | `u128` | Elapsed milliseconds |
| `.elapsed_us()` | `u128` | Elapsed microseconds |
| `.elapsed_nanos()` | `u128` | Elapsed nanoseconds |
| `Display` | — | `Xms` or `X.XXs` |

## Contributing

Contributions are welcome. To get started:

1. Fork the repository
2. Create a feature branch (`git checkout -b my-feature`)
3. Make your changes
4. Run the tests: `cargo test`
5. Submit a pull request

Please keep changes minimal and focused. This crate's goal is to stay small and dependency-free.

## License

This project is licensed under the [MIT License](LICENSE).
