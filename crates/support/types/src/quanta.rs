//! Fraction amounts.
//!
//! Ruling `docs/61` X-Q: the integer type is **`i64`, signed**.
//!
//! Signed is not a convenience — the `EmissionAccount` holds the *negative* of
//! total supply (`docs/11 §2.6`), so total supply is a directly queryable number
//! rather than an estimate. An unsigned type cannot represent it, which is why
//! `u128` was rejected despite being the wider type.
//!
//! Headroom: 1e9 lifetime FRC × 1e9 quanta = 1e18, against `i64::MAX` ≈ 9.22e18.
//! That is 9.2× headroom, asserted at compile time below.
//!
//! Wire format is a **decimal string**, never a JSON number: 1e18 is three orders
//! of magnitude past IEEE-754's exactly-representable range, so a JSON number
//! would silently round balances in every JavaScript client.

use std::fmt;
use std::str::FromStr;

/// Quanta per Fraction. `1 FRC = 1_000_000_000 quanta` (`docs/01 §4`).
pub const FRC: i64 = 1_000_000_000;

/// The maximum lifetime supply the economy is permitted to reach, in FRC.
///
/// Published, because P12 requires emission to be bounded, measured and public.
/// The const assertion below is a compile-time proof that this bound fits the
/// chosen integer type with room to spare.
pub const MAX_LIFETIME_FRC: i64 = 1_000_000_000;

// Compile-time proof that the headroom argument above actually holds.
const _: () = assert!(MAX_LIFETIME_FRC.checked_mul(FRC).is_some());

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum QuantaError {
    #[error("arithmetic overflow — an amount left the representable range")]
    Overflow,
    #[error("`{0}` is not a decimal amount")]
    NotANumber(String),
    #[error("more than 9 decimal places: a quantum is the smallest unit that exists")]
    TooPrecise,
}

/// An amount of Fraction, in quanta.
///
/// Arithmetic is **checked**, never saturating. A saturating ledger silently
/// invents or destroys money; a checked one refuses and says so.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Quanta(i64);

impl Quanta {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn from_raw(v: i64) -> Self {
        Self(v)
    }

    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }

    /// Construct from whole Fraction.
    ///
    /// # Errors
    /// [`QuantaError::Overflow`] if the amount exceeds the representable range.
    pub const fn from_frc(frc: i64) -> Result<Self, QuantaError> {
        match frc.checked_mul(FRC) {
            Some(v) => Ok(Self(v)),
            None => Err(QuantaError::Overflow),
        }
    }

    /// # Errors
    /// [`QuantaError::Overflow`] on overflow.
    pub const fn checked_add(self, other: Self) -> Result<Self, QuantaError> {
        match self.0.checked_add(other.0) {
            Some(v) => Ok(Self(v)),
            None => Err(QuantaError::Overflow),
        }
    }

    /// # Errors
    /// [`QuantaError::Overflow`] on overflow.
    pub const fn checked_sub(self, other: Self) -> Result<Self, QuantaError> {
        match self.0.checked_sub(other.0) {
            Some(v) => Ok(Self(v)),
            None => Err(QuantaError::Overflow),
        }
    }

    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Quanta {
    /// Always nine decimal places. A wallet balance that changes width as it
    /// updates reads as instability, which is the wrong signal for money
    /// (`docs/33 §3.4`).
    // Integer division by FRC is the point: quanta are integers and the split
    // between whole and fractional part is exact by construction.
    #[allow(clippy::integer_division)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        let whole = abs / (FRC as u64);
        let frac = abs % (FRC as u64);
        write!(f, "{sign}{whole}.{frac:09}")
    }
}

impl fmt::Debug for Quanta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self} FRC")
    }
}

impl FromStr for Quanta {
    type Err = QuantaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (neg, digits) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (whole_str, frac_str) = match digits.split_once('.') {
            Some((w, f)) => (w, f),
            None => (digits, ""),
        };
        if whole_str.is_empty() && frac_str.is_empty() {
            return Err(QuantaError::NotANumber(s.to_owned()));
        }
        if frac_str.len() > 9 {
            return Err(QuantaError::TooPrecise);
        }
        let parse = |part: &str| -> Result<i64, QuantaError> {
            if part.is_empty() {
                return Ok(0);
            }
            part.parse::<i64>()
                .map_err(|_| QuantaError::NotANumber(s.to_owned()))
        };
        let whole = parse(whole_str)?;
        let frac_digits = parse(frac_str)?;
        let scale = 10i64
            .checked_pow(u32::try_from(9 - frac_str.len()).map_err(|_| QuantaError::TooPrecise)?)
            .ok_or(QuantaError::Overflow)?;
        let total = whole
            .checked_mul(FRC)
            .and_then(|w| {
                frac_digits
                    .checked_mul(scale)
                    .and_then(|f| w.checked_add(f))
            })
            .ok_or(QuantaError::Overflow)?;
        Ok(Self(if neg { -total } else { total }))
    }
}

impl serde::Serialize for Quanta {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Decimal STRING. See the module docs: a JSON number rounds here.
        s.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Quanta {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn display_is_always_nine_places() {
        assert_eq!(Quanta::from_frc(1).unwrap().to_string(), "1.000000000");
        assert_eq!(Quanta::from_raw(1).to_string(), "0.000000001");
        assert_eq!(Quanta::from_raw(-1).to_string(), "-0.000000001");
    }

    #[test]
    fn round_trips_through_string() {
        for s in [
            "0.000000000",
            "1.000000000",
            "1204.482913000",
            "-84.000000000",
        ] {
            let q: Quanta = s.parse().unwrap();
            assert_eq!(q.to_string(), s, "round trip failed for {s}");
        }
    }

    #[test]
    fn rejects_sub_quantum_precision() {
        assert_eq!(
            "1.0000000001".parse::<Quanta>().unwrap_err(),
            QuantaError::TooPrecise
        );
    }

    #[test]
    fn arithmetic_is_checked_not_saturating() {
        let max = Quanta::from_raw(i64::MAX);
        assert_eq!(
            max.checked_add(Quanta::from_raw(1)).unwrap_err(),
            QuantaError::Overflow
        );
    }

    #[test]
    fn negative_balances_are_representable() {
        // The EmissionAccount holds -(total supply). This is the whole reason
        // the type is signed; if this test fails, docs/11 §7 invariant 4 is unprovable.
        let emission = Quanta::from_frc(-5000).unwrap();
        assert!(emission.is_negative());
        assert_eq!(emission.to_string(), "-5000.000000000");
    }

    #[test]
    fn serialises_as_a_string_not_a_number() {
        let json = serde_json::to_string(&Quanta::from_frc(1204).unwrap()).unwrap();
        assert!(json.starts_with('"'), "must be a JSON string, got {json}");
    }
}
