// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Clock abstraction: the only place in the core that touches `std::time::Instant` / `SystemTime`.
//!
//! `Instant::now()` and `SystemTime::now()` PANIC on `wasm32-unknown-unknown` ("time not implemented on this
//! platform"). Every time measurement in the core (time budgets of `export-fit`, durations in `ci` reports, the
//! training time of the face detector, commit time stamps, scratch-name entropy) therefore goes through this module:
//!
//! * on every target except `wasm32-unknown-unknown` it is a thin wrapper over the standard library;
//! * on `wasm32-unknown-unknown` it never panics: time comes from the host, which may call [`set_host_clock`]
//!   (the wasm module exports it as `arm_set_clock`, the JavaScript wrapper feeds it `performance.now()` and
//!   `Date.now()` before every call); a host that never does leaves the clock at 0, so durations are 0 and time
//!   budgets never expire (documented in docs/WASM.md).
//!
//! The host-supplied path is compiled on every target so that its logic is unit-tested natively.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static HOST_MONO_MS: AtomicU64 = AtomicU64::new(0);
static HOST_UNIX_MS: AtomicU64 = AtomicU64::new(0);

/// Tells the clock what time it is (for targets without a system clock). `mono_ms`: a monotonic millisecond counter
/// (`performance.now()`); `unix_ms`: milliseconds since 1970-01-01 UTC (`Date.now()`). Non-finite or negative values
/// are ignored. Used by [`Stamp::from_host`] and, on `wasm32-unknown-unknown`, by every other function here.
pub fn set_host_clock(mono_ms: f64, unix_ms: f64) {
    if mono_ms.is_finite() && mono_ms >= 0.0 {
        HOST_MONO_MS.store(mono_ms.to_bits(), Ordering::Relaxed);
    }
    if unix_ms.is_finite() && unix_ms >= 0.0 {
        HOST_UNIX_MS.store(unix_ms.to_bits(), Ordering::Relaxed);
    }
}

fn host_mono_ms() -> f64 {
    f64::from_bits(HOST_MONO_MS.load(Ordering::Relaxed))
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn host_unix_ms() -> f64 {
    f64::from_bits(HOST_UNIX_MS.load(Ordering::Relaxed))
}

/// Duration between two millisecond readings; never negative, never panics.
fn span_ms(start: f64, now: f64) -> Duration {
    let d = now - start;
    if d.is_finite() && d > 0.0 {
        Duration::from_secs_f64(d / 1000.0)
    } else {
        Duration::ZERO
    }
}

/// A point in time to measure from (the replacement for `std::time::Instant`).
#[derive(Clone, Copy, Debug)]
pub struct Stamp {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    t: std::time::Instant,
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    ms: f64,
    /// Used by [`Stamp::from_host`] on targets that have a real clock.
    host_ms: Option<f64>,
}

impl Stamp {
    /// Now, from the system clock (or the host clock on `wasm32-unknown-unknown`).
    pub fn now() -> Stamp {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        {
            Stamp { t: std::time::Instant::now(), host_ms: None }
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            Stamp { ms: host_mono_ms(), host_ms: None }
        }
    }

    /// Now, from the host clock set with [`set_host_clock`] on every target (what `now` does on wasm32).
    pub fn from_host() -> Stamp {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        {
            Stamp { t: std::time::Instant::now(), host_ms: Some(host_mono_ms()) }
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            Stamp { ms: host_mono_ms(), host_ms: Some(host_mono_ms()) }
        }
    }

    /// Time since this stamp was taken (0 when the host clock never moved).
    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.host_ms {
            return span_ms(start, host_mono_ms());
        }
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        {
            self.t.elapsed()
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            span_ms(self.ms, host_mono_ms())
        }
    }
}

/// Seconds since 1970-01-01 UTC (0 if the clock is before it, or on wasm32 when the host never set the clock).
pub fn unix_seconds() -> u64 {
    (unix_nanos() / 1_000_000_000) as u64
}

/// Nanoseconds since 1970-01-01 UTC; see [`unix_seconds`].
pub fn unix_nanos() -> u128 {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
    }
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        (host_unix_ms() * 1_000_000.0) as u128
    }
}

/// The process id, or 0 where there are no processes (`std::process::id()` panics on `wasm32-unknown-unknown`).
pub fn process_id() -> u32 {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        std::process::id()
    }
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_are_clamped_and_converted() {
        assert_eq!(span_ms(10.0, 1510.0), Duration::from_millis(1500));
        assert_eq!(span_ms(10.0, 10.0), Duration::ZERO);
        assert_eq!(span_ms(10.0, 5.0), Duration::ZERO, "a clock that went back gives 0, not a panic");
        assert_eq!(span_ms(f64::NAN, 5.0), Duration::ZERO);
        assert_eq!(span_ms(0.0, f64::INFINITY), Duration::ZERO);
    }

    /// The host-clock path (the only one on wasm32-unknown-unknown) works on any target.
    #[test]
    fn host_clock_drives_stamps_and_unix_time_is_sane() {
        set_host_clock(1000.0, 1_700_000_000_000.0);
        let s = Stamp::from_host();
        assert_eq!(s.elapsed(), Duration::ZERO);
        set_host_clock(3500.0, 1_700_000_002_500.0);
        assert_eq!(s.elapsed(), Duration::from_millis(2500));
        // invalid readings are ignored
        set_host_clock(f64::NAN, -1.0);
        assert_eq!(s.elapsed(), Duration::from_millis(2500));
        // the system clock is used on this target and is after 2020
        assert!(unix_seconds() > 1_577_836_800 || cfg!(all(target_arch = "wasm32", target_os = "unknown")));
        let a = Stamp::now();
        assert!(a.elapsed() < Duration::from_secs(5));
    }
}
