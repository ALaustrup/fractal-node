//! Seeded generators for simulation.
//!
//! Deliberately in the `support` layer and deliberately ignorant of the domain:
//! this produces strings, choices and weighted coin flips, and the harness that
//! knows what a Society is composes them. Keeping it that way is what lets the
//! same generators drive every boundary as they arrive, rather than each one
//! growing its own.

use fractal_ports::Rng;

/// A seeded source of test inputs.
///
/// Every method draws from the same `Rng`, so a whole generated history is a
/// pure function of one seed. That is the property the entire simulation rests
/// on: a failure is not "it broke sometimes", it is "it breaks at seed 41,
/// step 380", which is a thing you can debug.
pub struct Gen<'a> {
    rng: &'a dyn Rng,
}

impl core::fmt::Debug for Gen<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Gen").finish_non_exhaustive()
    }
}

impl<'a> Gen<'a> {
    #[must_use]
    pub fn new(rng: &'a dyn Rng) -> Self {
        Self { rng }
    }

    /// A number in `0..n`. Returns 0 when `n` is 0.
    #[must_use]
    pub fn below(&self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        self.rng.next_u64() % n
    }

    /// True with probability `percent`.
    #[must_use]
    pub fn chance(&self, percent: u64) -> bool {
        self.below(100) < percent
    }

    /// One of `options`. Panics only if `options` is empty, which is a caller bug.
    ///
    /// # Panics
    /// If `options` is empty.
    #[must_use]
    pub fn pick<'b, T>(&self, options: &'b [T]) -> &'b T {
        let i = usize::try_from(self.below(options.len() as u64)).unwrap_or(0);
        options
            .get(i)
            .unwrap_or_else(|| unreachable!("pick from an empty slice"))
    }

    /// A handle-shaped string: `[a-z0-9_]`, 3–24 characters, no edge underscore.
    ///
    /// Generates *valid* handles by construction. Invalid ones are produced
    /// deliberately by [`Self::bad_handle`], so a test that expects a rejection
    /// says so rather than relying on chance.
    #[must_use]
    pub fn handle(&self) -> String {
        const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789_";
        let len = 3 + self.below(22);
        let mut out = String::new();
        for i in 0..len {
            let idx = usize::try_from(self.below(ALPHABET.len() as u64)).unwrap_or(0);
            let mut c = char::from(*ALPHABET.get(idx).unwrap_or(&b'a'));
            // No leading or trailing underscore.
            if c == '_' && (i == 0 || i == len - 1) {
                c = 'a';
            }
            out.push(c);
        }
        out
    }

    /// A handle that must be rejected, and the reason it must be.
    #[must_use]
    pub fn bad_handle(&self) -> (String, &'static str) {
        match self.below(4) {
            0 => (String::new(), "empty"),
            1 => ("ab".to_owned(), "too short"),
            2 => ("_leading".to_owned(), "leading underscore"),
            _ => ("has space".to_owned(), "illegal character"),
        }
    }

    /// A plausible Society name.
    #[must_use]
    pub fn name(&self) -> String {
        const FIRST: &[&str] = &[
            "Oracle", "Signal", "Long", "Quiet", "Deep", "Bright", "Iron", "Salt",
        ];
        const SECOND: &[&str] = &[
            "Hall", "Lab", "Winter", "Commons", "Field", "Works", "Circle", "Relay",
        ];
        alloc_format(self.pick(FIRST), self.pick(SECOND))
    }

    /// A name that must be rejected, and the reason.
    #[must_use]
    pub fn bad_name(&self) -> (String, &'static str) {
        if self.chance(50) {
            ("   ".to_owned(), "blank")
        } else {
            ("x".repeat(65), "too long")
        }
    }
}

fn alloc_format(a: &str, b: &str) -> String {
    let mut s = String::with_capacity(a.len() + b.len() + 1);
    s.push_str(a);
    s.push(' ');
    s.push_str(b);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SeededRng;

    #[test]
    fn generated_handles_are_always_valid() {
        // If this generator can emit an invalid handle, every "unexpected
        // rejection" in the simulation becomes ambiguous — is it the domain or
        // the generator? So it is checked directly.
        let rng = SeededRng::new(7);
        let g = Gen::new(&rng);
        for _ in 0..2_000 {
            let h = g.handle();
            let len = h.chars().count();
            assert!((3..=24).contains(&len), "length {len}: {h}");
            assert!(
                h.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_')),
                "{h}"
            );
            assert!(!h.starts_with('_') && !h.ends_with('_'), "{h}");
        }
    }

    #[test]
    fn the_same_seed_generates_the_same_inputs() {
        let a = SeededRng::new(11);
        let b = SeededRng::new(11);
        let (ga, gb) = (Gen::new(&a), Gen::new(&b));
        for _ in 0..100 {
            assert_eq!(ga.handle(), gb.handle());
            assert_eq!(ga.name(), gb.name());
        }
    }

    #[test]
    fn bad_inputs_are_actually_bad() {
        let rng = SeededRng::new(3);
        let g = Gen::new(&rng);
        for _ in 0..200 {
            let (h, _) = g.bad_handle();
            let ok = (3..=24).contains(&h.chars().count())
                && h.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_'))
                && !h.starts_with('_')
                && !h.ends_with('_');
            assert!(!ok, "bad_handle produced a valid handle: {h:?}");
        }
    }
}
