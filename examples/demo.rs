use nanotime::{Elapsed, NanoTime};

fn main() {
    // Current time
    let local = NanoTime::now();
    println!("Local time: {}", local);
    println!("Local date: {}", local.date());
    println!("Local datetime: {}", local.datetime());

    let utc = NanoTime::now_utc();
    println!("UTC datetime: {}", utc.datetime());

    // Sub-second accessors
    println!("Nanoseconds: {}", utc.nanosecond());
    println!("Microseconds: {}", utc.microsecond());
    println!("Milliseconds: {}", utc.millisecond());

    // Flexible formatting
    let t = NanoTime::new(2026, 2, 22, 14, 30, 5, 123_456_789).unwrap();
    println!("Precision 0: {}", t.datetime_fmt(0));
    println!("Precision 3: {}", t.datetime_fmt(3));
    println!("Precision 6: {}", t.datetime_fmt(6));
    println!("Precision 9: {}", t.datetime_fmt(9));

    // Epoch constructors at different granularities
    let from_ms = NanoTime::from_epoch_ms(1_000_000_000_042);
    let from_us = NanoTime::from_epoch_us(1_000_000_000_042_000);
    let from_ns = NanoTime::from_epoch_nanos(1_000_000_000_042_000_000);
    println!("From ms: {}", from_ms.datetime());
    println!("From µs: {}", from_us.datetime());
    println!("From ns: {}", from_ns.datetime());

    // Relative time
    let past = NanoTime::new(2025, 1, 1, 0, 0, 0, 0).unwrap();
    println!("New Year 2025 was: {}", past.ago());

    let a = NanoTime::new(2026, 2, 22, 12, 0, 0, 0).unwrap();
    let b = NanoTime::new(2026, 2, 22, 14, 30, 0, 0).unwrap();
    println!("a relative to b: {}", a.relative_to(&b));
    println!("b relative to a: {}", b.relative_to(&a));

    // Ordering
    println!("a < b: {}", a < b);

    // Elapsed timer
    let timer = Elapsed::start();
    let mut sum = 0u64;
    for i in 0..1_000_000 {
        sum = sum.wrapping_add(i);
    }
    println!("Crunched {} in {}", sum, timer);
    println!("Elapsed nanos: {}", timer.elapsed_nanos());
}
