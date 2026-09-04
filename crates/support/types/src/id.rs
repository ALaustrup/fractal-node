//! Identifiers. Formats are fixed by `docs/11 §6` and are part of the public contract.

use std::fmt;
use std::str::FromStr;

/// Crockford base32 alphabet: no I, L, O or U, so an FNID read aloud or copied
/// by hand cannot be confused. This is why we do not use standard base32.
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IdError {
    #[error("expected prefix `{expected}`")]
    BadPrefix { expected: &'static str },
    #[error("invalid character `{0}` — FNIDs use Crockford base32")]
    BadChar(char),
    #[error("wrong length: got {got}, expected {expected}")]
    BadLength { got: usize, expected: usize },
    #[error("checksum mismatch — this identifier was mistyped or truncated")]
    BadChecksum,
    #[error("not a valid ULID")]
    BadUlid,
}

/// A time-sortable unique identifier.
///
/// Wrapped rather than aliased so the `ulid` crate stays swappable and so
/// `IdGen` (`docs/10 §7`) is the only way one is minted in domain code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ulid(ulid::Ulid);

impl Ulid {
    /// Construct from raw 128 bits. Used by `IdGen` implementations and tests only.
    #[must_use]
    pub const fn from_u128(v: u128) -> Self {
        Self(ulid::Ulid(v))
    }

    #[must_use]
    pub const fn to_u128(self) -> u128 {
        self.0 .0
    }

    /// Milliseconds since the Unix epoch, recovered from the ULID's time component.
    #[must_use]
    pub fn timestamp_ms(self) -> u64 {
        self.0.timestamp_ms()
    }
}

impl fmt::Display for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ulid({})", self.0)
    }
}

impl FromStr for Ulid {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ulid::Ulid::from_string(s)
            .map(Self)
            .map_err(|_| IdError::BadUlid)
    }
}

impl serde::Serialize for Ulid {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Ulid {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A cryptographic identity: a Citizen, Society, Agent or Node.
///
/// Format: `fn1` + Crockford-base32(32-byte public key) + 4-character checksum.
/// Self-certifying, human-checkable, and tied to no chain (P11).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fnid([u8; 32]);

impl Fnid {
    pub const PREFIX: &'static str = "fn1";

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// A deterministic FNID for tests and fixtures. Never used in production paths.
    // `i` is bounded by the loop condition and the array is fixed at 32.
    #[allow(clippy::indexing_slicing)]
    #[must_use]
    pub const fn sample(seed: u8) -> Self {
        let mut b = [0u8; 32];
        let mut i: u8 = 0;
        while (i as usize) < 32 {
            b[i as usize] = seed.wrapping_add(i);
            i += 1;
        }
        Self(b)
    }

    /// Non-cryptographic integrity check over the encoded body.
    ///
    /// This catches transcription errors, which is all a checksum is for. It is
    /// deliberately not a hash: the identifier's security comes from the key it
    /// encodes, never from these four characters.
    // Every index below is masked to 0..=31, and CROCKFORD has exactly 32 entries.
    #[allow(clippy::indexing_slicing)]
    fn checksum(body: &[u8]) -> [u8; 4] {
        let mut acc: u32 = 0x811C_9DC5;
        for &b in body {
            acc ^= u32::from(b);
            acc = acc.wrapping_mul(0x0100_0193);
        }
        [
            CROCKFORD[((acc >> 24) & 0x1F) as usize],
            CROCKFORD[((acc >> 16) & 0x1F) as usize],
            CROCKFORD[((acc >> 8) & 0x1F) as usize],
            CROCKFORD[(acc & 0x1F) as usize],
        ]
    }

    // `idx` is masked to 0..=31 before every lookup.
    #[allow(clippy::indexing_slicing)]
    fn encode_body(&self) -> String {
        // 32 bytes -> 52 base32 characters (256 bits / 5, rounded up).
        let mut out = String::with_capacity(52);
        let mut acc: u16 = 0;
        let mut bits: u8 = 0;
        for &byte in &self.0 {
            acc = (acc << 8) | u16::from(byte);
            bits += 8;
            while bits >= 5 {
                bits -= 5;
                let idx = ((acc >> bits) & 0x1F) as usize;
                out.push(char::from(CROCKFORD[idx]));
            }
        }
        if bits > 0 {
            let idx = ((acc << (5 - bits)) & 0x1F) as usize;
            out.push(char::from(CROCKFORD[idx]));
        }
        out
    }
}

impl fmt::Display for Fnid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body = self.encode_body();
        let sum = Self::checksum(body.as_bytes());
        write!(f, "{}{}", Self::PREFIX, body)?;
        for c in sum {
            write!(f, "{}", char::from(c))?;
        }
        Ok(())
    }
}

impl fmt::Debug for Fnid {
    /// Abbreviated on purpose: a full FNID in a log line is noise, and the
    /// abbreviated form is what every surface in `docs/32` renders.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.to_string();
        let head: String = s.chars().take(7).collect();
        let tail: String = s
            .chars()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        write!(f, "{head}…{tail}")
    }
}

impl FromStr for Fnid {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix(Self::PREFIX).ok_or(IdError::BadPrefix {
            expected: Self::PREFIX,
        })?;
        if rest.len() != 56 {
            return Err(IdError::BadLength {
                got: rest.len(),
                expected: 56,
            });
        }
        let (body, sum) = rest.split_at(52);
        if Self::checksum(body.as_bytes()) != sum.as_bytes() {
            return Err(IdError::BadChecksum);
        }
        let mut bytes = [0u8; 32];
        let mut acc: u16 = 0;
        let mut bits: u8 = 0;
        let mut out = 0usize;
        for ch in body.chars() {
            let upper = ch.to_ascii_uppercase() as u8;
            let idx = CROCKFORD
                .iter()
                .position(|&c| c == upper)
                .ok_or(IdError::BadChar(ch))?;
            acc = (acc << 5) | u16::try_from(idx).map_err(|_| IdError::BadChar(ch))?;
            bits += 5;
            if bits >= 8 {
                bits -= 8;
                if out < 32 {
                    if let Some(slot) = bytes.get_mut(out) {
                        *slot = ((acc >> bits) & 0xFF) as u8;
                    }
                    out += 1;
                }
            }
        }
        Ok(Self(bytes))
    }
}

impl serde::Serialize for Fnid {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Fnid {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A Society identifier: `soc_` + ULID (`docs/11 §6`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SocietyId(Ulid);

impl SocietyId {
    pub const PREFIX: &'static str = "soc_";

    #[must_use]
    pub const fn new(id: Ulid) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn inner(self) -> Ulid {
        self.0
    }
}

impl fmt::Display for SocietyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", Self::PREFIX, self.0)
    }
}

impl fmt::Debug for SocietyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl FromStr for SocietyId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix(Self::PREFIX).ok_or(IdError::BadPrefix {
            expected: Self::PREFIX,
        })?;
        rest.parse().map(Self)
    }
}

impl serde::Serialize for SocietyId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for SocietyId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HandleError {
    #[error("a Handle is 3–24 characters; got {0}")]
    Length(usize),
    #[error("invalid character `{0}` — Handles use a–z, 0–9 and underscore")]
    BadChar(char),
    #[error("a Handle cannot begin or end with an underscore")]
    EdgeUnderscore,
}

/// A globally unique human-readable identifier, `@name`.
///
/// Case-folded on construction: `@Kaya` and `@kaya` are the same Handle, so the
/// uniqueness namespace cannot be gamed with capitalisation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle(String);

impl Handle {
    pub const MIN: usize = 3;
    pub const MAX: usize = 24;

    /// # Errors
    /// Returns [`HandleError`] when the input is the wrong length or shape.
    pub fn parse(raw: &str) -> Result<Self, HandleError> {
        let s = raw.strip_prefix('@').unwrap_or(raw).to_ascii_lowercase();
        let len = s.chars().count();
        if !(Self::MIN..=Self::MAX).contains(&len) {
            return Err(HandleError::Length(len));
        }
        if let Some(bad) = s
            .chars()
            .find(|c| !matches!(c, 'a'..='z' | '0'..='9' | '_'))
        {
            return Err(HandleError::BadChar(bad));
        }
        if s.starts_with('_') || s.ends_with('_') {
            return Err(HandleError::EdgeUnderscore);
        }
        Ok(Self(s))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl serde::Serialize for Handle {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Handle {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn fnid_round_trips() {
        let id = Fnid::sample(9);
        let s = id.to_string();
        assert!(s.starts_with("fn1"));
        assert_eq!(s.len(), 3 + 52 + 4);
        assert_eq!(s.parse::<Fnid>().unwrap(), id);
    }

    #[test]
    fn fnid_rejects_a_single_character_typo() {
        let id = Fnid::sample(4);
        let mut s = id.to_string();
        // Corrupt one character of the body; the checksum must catch it.
        let byte = s.as_bytes()[10];
        let replacement = if byte == b'Z' { '0' } else { 'Z' };
        s.replace_range(10..11, &replacement.to_string());
        assert_eq!(s.parse::<Fnid>().unwrap_err(), IdError::BadChecksum);
    }

    #[test]
    fn fnid_debug_is_abbreviated() {
        // A full 59-character identifier in a log line is noise; docs/32 renders
        // the abbreviated form everywhere.
        let d = format!("{:?}", Fnid::sample(1));
        assert!(d.contains('…'), "got {d}");
        assert!(d.len() < 20, "got {d}");
    }

    #[test]
    fn handles_are_case_folded() {
        assert_eq!(
            Handle::parse("@Kaya").unwrap(),
            Handle::parse("kaya").unwrap()
        );
        assert_eq!(Handle::parse("KAYA").unwrap().to_string(), "@kaya");
    }

    #[test]
    fn handle_rules_are_enforced() {
        assert_eq!(Handle::parse("ab").unwrap_err(), HandleError::Length(2));
        assert_eq!(
            Handle::parse("_kaya").unwrap_err(),
            HandleError::EdgeUnderscore
        );
        assert_eq!(
            Handle::parse("ka ya").unwrap_err(),
            HandleError::BadChar(' ')
        );
        assert!(Handle::parse("oracle_hall").is_ok());
    }

    #[test]
    fn society_id_displays_with_its_prefix() {
        let s = SocietyId::new(Ulid::from_u128(1));
        assert!(s.to_string().starts_with("soc_"));
        assert_eq!(s.to_string().parse::<SocietyId>().unwrap(), s);
    }
}
