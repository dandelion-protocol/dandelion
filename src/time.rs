use alloc::fmt;
use core::ops::{
    Add,
    AddAssign,
    Div,
    DivAssign,
    Mul,
    MulAssign,
    Neg,
    Rem,
    RemAssign,
    Sub,
    SubAssign,
};

use dandelion_wire::Printable;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    nanoseconds: i64,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    since_epoch: Duration,
}

const MINUTES_PER_HOUR: i64 = 60;
const HOURS_PER_DAY: i64 = 24;
const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = SECONDS_PER_MINUTE * MINUTES_PER_HOUR;
const SECONDS_PER_DAY: i64 = SECONDS_PER_HOUR * HOURS_PER_DAY;
const NANOS_PER_MICRO: i64 = 1_000;
const NANOS_PER_MILLI: i64 = 1_000_000;
const NANOS_PER_SECOND: i64 = 1_000_000_000;
const NANOS_PER_MINUTE: i64 = NANOS_PER_SECOND * SECONDS_PER_MINUTE;
const NANOS_PER_HOUR: i64 = NANOS_PER_SECOND * SECONDS_PER_HOUR;
const NANOS_PER_DAY: i64 = NANOS_PER_SECOND * SECONDS_PER_DAY;

macro_rules! mul_constructor {
    ($method:ident, $local:ident, $scalar:expr) => {
        pub const fn $method($local: i64) -> Self {
            Self::from_nanoseconds($local.saturating_mul($scalar))
        }
    };
}

macro_rules! div_accessor {
    ($method:ident, $scalar:expr) => {
        pub const fn $method(self) -> i64 {
            if self.is_pos_infinity() {
                i64::MAX
            } else if self.is_neg_infinity() {
                i64::MIN
            } else {
                self.as_nanoseconds().strict_div($scalar)
            }
        }
    };
}

impl Duration {
    pub const ZERO: Self = Self::new(0);
    pub const MAX: Self = Self::new(i64::MAX);
    pub const MIN: Self = Self::new(i64::MIN);

    const fn new(nanoseconds: i64) -> Self {
        Self { nanoseconds }
    }

    pub const fn from_nanoseconds(nanoseconds: i64) -> Self {
        if nanoseconds == -i64::MAX {
            Self::MIN
        } else {
            Self::new(nanoseconds)
        }
    }

    mul_constructor!(from_microseconds, microseconds, NANOS_PER_MICRO);
    mul_constructor!(from_milliseconds, milliseconds, NANOS_PER_MILLI);
    mul_constructor!(from_seconds, seconds, NANOS_PER_SECOND);
    mul_constructor!(from_minutes, minutes, NANOS_PER_MINUTE);
    mul_constructor!(from_hours, hours, NANOS_PER_HOUR);
    mul_constructor!(from_days, days, NANOS_PER_DAY);

    pub const fn is_zero(self) -> bool {
        self.nanoseconds == 0
    }

    pub const fn is_positive(self) -> bool {
        self.nanoseconds > 0
    }

    pub const fn is_negative(self) -> bool {
        self.nanoseconds < 0
    }

    pub const fn is_non_zero(self) -> bool {
        self.nanoseconds != 0
    }

    pub const fn is_non_negative(self) -> bool {
        self.nanoseconds >= 0
    }

    pub const fn is_pos_infinity(self) -> bool {
        self.nanoseconds >= i64::MAX
    }

    pub const fn is_neg_infinity(self) -> bool {
        self.nanoseconds <= -i64::MAX
    }

    pub const fn is_infinity(self) -> bool {
        self.is_pos_infinity() || self.is_neg_infinity()
    }

    pub const fn is_finite(self) -> bool {
        !self.is_infinity()
    }

    pub const fn as_nanoseconds(self) -> i64 {
        self.nanoseconds
    }

    div_accessor!(as_microseconds, NANOS_PER_MICRO);
    div_accessor!(as_milliseconds, NANOS_PER_MILLI);
    div_accessor!(as_seconds, NANOS_PER_SECOND);
    div_accessor!(as_minutes, NANOS_PER_MINUTE);
    div_accessor!(as_hours, NANOS_PER_HOUR);
    div_accessor!(as_days, NANOS_PER_DAY);

    pub const fn abs(self) -> Self {
        Self::from_nanoseconds(self.nanoseconds.saturating_abs())
    }

    pub const fn neg(self) -> Self {
        Self::from_nanoseconds(self.nanoseconds.saturating_neg())
    }

    pub const fn add(self, rhs: Self) -> Self {
        Self::from_nanoseconds(self.nanoseconds.saturating_add(rhs.nanoseconds))
    }

    pub const fn sub(self, rhs: Self) -> Self {
        Self::from_nanoseconds(self.nanoseconds.saturating_sub(rhs.nanoseconds))
    }

    pub const fn div(self, rhs: Self) -> i64 {
        let (quo, _) = self.divmod(rhs);
        quo
    }

    pub const fn rem(self, rhs: Self) -> Self {
        let (_, rem) = self.divmod(rhs);
        rem
    }

    pub const fn divmod(self, rhs: Self) -> (i64, Self) {
        let (quo, rem) = divmod_i64(self.nanoseconds, rhs.nanoseconds);
        (quo, Self::from_nanoseconds(rem))
    }

    pub const fn scalar_mul(self, scalar: i64) -> Self {
        Self::from_nanoseconds(self.nanoseconds.saturating_mul(scalar))
    }

    pub const fn scalar_div(self, scalar: i64) -> Self {
        let (quo, _) = self.scalar_divmod(scalar);
        quo
    }
    pub const fn scalar_rem(self, scalar: i64) -> Self {
        let (_, rem) = self.scalar_divmod(scalar);
        rem
    }

    pub const fn scalar_divmod(self, scalar: i64) -> (Self, Self) {
        let (quo, rem) = divmod_i64(self.nanoseconds, scalar);
        (Self::from_nanoseconds(quo), Self::from_nanoseconds(rem))
    }
}

impl Instant {
    pub const ZERO: Self = Self::new(Duration::ZERO);
    pub const MIN: Self = Self::new(Duration::MIN);
    pub const MAX: Self = Self::new(Duration::MAX);

    const fn new(since_epoch: Duration) -> Self {
        Self { since_epoch }
    }

    pub const fn add(self, rhs: Duration) -> Self {
        Self::new(self.since_epoch.add(rhs))
    }

    pub const fn sub(self, rhs: Duration) -> Self {
        Self::new(self.since_epoch.sub(rhs))
    }

    pub const fn diff(self, rhs: Self) -> Duration {
        self.since_epoch.sub(rhs.since_epoch)
    }
}

macro_rules! op {
    ($ty:ty, $trait:ty, $method:ident, out $out:ty) => {
        impl $trait for $ty {
            type Output = $out;
            fn $method(self) -> Self::Output {
                Self::$method(self)
            }
        }
    };
    ($ty:ty, $trait:ident, $method:ident, out $out:ty, rhs $rhs:ty) => {
        impl $trait<$rhs> for $ty {
            type Output = $out;
            fn $method(self, rhs: $rhs) -> Self::Output {
                Self::$method(self, rhs)
            }
        }
    };
    ($ty:ty, $trait:ident, $trait_method:ident as $ty_method:ident, out $out:ty, rhs $rhs:ty) => {
        impl $trait<$rhs> for $ty {
            type Output = $out;
            fn $trait_method(self, rhs: $rhs) -> Self::Output {
                Self::$ty_method(self, rhs)
            }
        }
    };
    ($ty:ty, $trait:ident, $trait_method:ident, assign $ty_method:ident, rhs $rhs:ty) => {
        impl $trait<$rhs> for $ty {
            fn $trait_method(&mut self, rhs: $rhs) {
                *self = self.$ty_method(rhs);
            }
        }
    };
}

op!(Duration, Neg, neg, out Self);
op!(Duration, Add, add, out Self, rhs Self);
op!(Duration, Sub, sub, out Self, rhs Self);
op!(Duration, Mul, mul as scalar_mul, out Self, rhs i64);
op!(Duration, Div, div as scalar_div, out Self, rhs i64);
op!(Duration, Rem, rem as scalar_rem, out Self, rhs i64);
op!(Duration, Div, div, out i64, rhs Self);
op!(Duration, Rem, rem, out Self, rhs Self);
op!(Duration, AddAssign, add_assign, assign add, rhs Self);
op!(Duration, SubAssign, sub_assign, assign sub, rhs Self);
op!(Duration, MulAssign, mul_assign, assign scalar_mul, rhs i64);
op!(Duration, DivAssign, div_assign, assign scalar_div, rhs i64);
op!(Duration, RemAssign, rem_assign, assign scalar_rem, rhs i64);
op!(Duration, RemAssign, rem_assign, assign rem, rhs Self);

op!(Instant, Add, add, out Self, rhs Duration);
op!(Instant, Sub, sub, out Self, rhs Duration);
op!(Instant, Sub, sub as diff, out Duration, rhs Self);
op!(Instant, AddAssign, add_assign, assign add, rhs Duration);
op!(Instant, SubAssign, sub_assign, assign sub, rhs Duration);

impl Printable for Duration {
    fn print(&self, writer: &mut dyn fmt::Write) -> fmt::Result {
        if self.is_zero() {
            writer.write_str("0")
        } else if self.is_pos_infinity() {
            writer.write_str("+infinity")
        } else if self.is_neg_infinity() {
            writer.write_str("-infinity")
        } else {
            let ns = self.nanoseconds.unsigned_abs();
            let (s, ns) = divmod_u64(ns, NANOS_PER_SECOND as u64);
            let (m, s) = divmod_u64(s, SECONDS_PER_MINUTE as u64);
            let (h, m) = divmod_u64(m, MINUTES_PER_HOUR as u64);
            let (d, h) = divmod_u64(h, HOURS_PER_DAY as u64);
            let (ms, ms_ns) = divmod_u64(ns, NANOS_PER_MILLI as u64);
            let (us, us_ns) = divmod_u64(ns, NANOS_PER_MICRO as u64);

            if self.is_negative() {
                writer.write_char('-')?;
                writer.write_char('[')?;
            }

            let mut is_first = true;
            macro_rules! field {
                ( $format:literal, $value:ident ) => {
                    if $value != 0 {
                        if !is_first {
                            writer.write_char(' ')?;
                        }
                        writer.write_fmt(format_args!($format, $value))?;
                        is_first = false;
                    }
                };
            }
            field!("{}d", d);
            field!("{}h", h);
            field!("{}m", m);
            field!("{}s", s);
            if ns != 0 {
                if !is_first {
                    writer.write_char(' ')?;
                }
                if ms_ns == 0 {
                    writer.write_fmt(format_args!("{}ms", ms))?;
                } else if us_ns == 0 {
                    writer.write_fmt(format_args!("{}µs", us))?;
                } else {
                    writer.write_fmt(format_args!("{}ns", ns))?;
                }
            }

            if self.is_negative() {
                writer.write_char(']')?;
            }
            Ok(())
        }
    }
}

impl Printable for Instant {
    fn print(&self, writer: &mut dyn fmt::Write) -> fmt::Result {
        self.since_epoch.print(writer)?;
        writer.write_str(" since epoch")
    }
}

impl_serializable_for_struct!(Duration { nanoseconds: i64 }, fixed size);
impl_debug_for_printable!(Duration);
impl_display_for_printable!(Duration);

impl_serializable_for_struct!(Instant { since_epoch: Duration }, fixed size);
impl_debug_for_printable!(Instant);
impl_display_for_printable!(Instant);

const fn divmod_u64(lhs: u64, rhs: u64) -> (u64, u64) {
    (lhs.strict_div_euclid(rhs), lhs.strict_rem_euclid(rhs))
}

const fn divmod_i64(lhs: i64, rhs: i64) -> (i64, i64) {
    // NB: saturating_{div,rem}_euclid don't exist, but this is what they would do
    if lhs == i64::MIN && rhs == -1 {
        (i64::MAX, 0)
    } else {
        (lhs.strict_div_euclid(rhs), lhs.strict_rem_euclid(rhs))
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::*;

    #[test]
    fn duration_print() {
        let mut buffer = String::with_capacity(64);

        macro_rules! duration_write_into_test_case {
            ( $ctor:expr, $expect:expr ) => {
                let value = $ctor;
                value.print(&mut buffer).unwrap();
                assert_eq!($expect, &mut buffer);
                buffer.clear();
            };
            ( $ctor:expr, $pos:expr, $neg:expr ) => {
                let value = $ctor;
                value.print(&mut buffer).unwrap();
                assert_eq!($pos, &mut buffer);
                buffer.clear();
                let value = value.neg();
                value.print(&mut buffer).unwrap();
                assert_eq!($neg, &mut buffer);
                buffer.clear();
            };
        }

        duration_write_into_test_case!(Duration::ZERO, "0");
        duration_write_into_test_case!(Duration::MAX, "+infinity", "-infinity");
        duration_write_into_test_case!(Duration::MIN, "-infinity", "+infinity");
        duration_write_into_test_case!(-Duration::MAX, "-infinity", "+infinity");
        duration_write_into_test_case!(Duration::from_nanoseconds(-1), "-[1ns]", "1ns");
        duration_write_into_test_case!(Duration::from_nanoseconds(1), "1ns", "-[1ns]");
        duration_write_into_test_case!(Duration::from_nanoseconds(37), "37ns", "-[37ns]");
        duration_write_into_test_case!(Duration::from_nanoseconds(131), "131ns", "-[131ns]");
        duration_write_into_test_case!(Duration::from_nanoseconds(997), "997ns", "-[997ns]");
        duration_write_into_test_case!(
            Duration::from_nanoseconds(10000079),
            "10000079ns",
            "-[10000079ns]"
        );
        duration_write_into_test_case!(Duration::from_microseconds(1), "1µs", "-[1µs]");
        duration_write_into_test_case!(Duration::from_microseconds(37), "37µs", "-[37µs]");
        duration_write_into_test_case!(Duration::from_microseconds(131), "131µs", "-[131µs]");
        duration_write_into_test_case!(Duration::from_microseconds(997), "997µs", "-[997µs]");
        duration_write_into_test_case!(Duration::from_milliseconds(1), "1ms", "-[1ms]");
        duration_write_into_test_case!(Duration::from_milliseconds(37), "37ms", "-[37ms]");
        duration_write_into_test_case!(Duration::from_milliseconds(131), "131ms", "-[131ms]");
        duration_write_into_test_case!(Duration::from_milliseconds(997), "997ms", "-[997ms]");
        duration_write_into_test_case!(Duration::from_seconds(1), "1s", "-[1s]");
        duration_write_into_test_case!(Duration::from_seconds(37), "37s", "-[37s]");
        duration_write_into_test_case!(Duration::from_seconds(131), "2m 11s", "-[2m 11s]");
        duration_write_into_test_case!(Duration::from_seconds(997), "16m 37s", "-[16m 37s]");
        duration_write_into_test_case!(Duration::from_seconds(8423), "2h 20m 23s", "-[2h 20m 23s]");
        duration_write_into_test_case!(
            Duration::from_seconds(999809),
            "11d 13h 43m 29s",
            "-[11d 13h 43m 29s]"
        );
        duration_write_into_test_case!(Duration::from_minutes(1), "1m", "-[1m]");
        duration_write_into_test_case!(Duration::from_minutes(37), "37m", "-[37m]");
        duration_write_into_test_case!(Duration::from_minutes(131), "2h 11m", "-[2h 11m]");
        duration_write_into_test_case!(Duration::from_minutes(997), "16h 37m", "-[16h 37m]");
        duration_write_into_test_case!(Duration::from_hours(1), "1h", "-[1h]");
        duration_write_into_test_case!(Duration::from_hours(37), "1d 13h", "-[1d 13h]");
        duration_write_into_test_case!(Duration::from_hours(131), "5d 11h", "-[5d 11h]");
        duration_write_into_test_case!(Duration::from_hours(997), "41d 13h", "-[41d 13h]");
        duration_write_into_test_case!(Duration::from_days(1), "1d", "-[1d]");
        duration_write_into_test_case!(Duration::from_days(37), "37d", "-[37d]");
        duration_write_into_test_case!(Duration::from_days(131), "131d", "-[131d]");
        duration_write_into_test_case!(Duration::from_days(997), "997d", "-[997d]");
    }
}
