//! Morphology options: an experimental proposal and one dialect switch.
//!
//! Three presets:
//! - [`MorphOptions::default()`]: CLL Chapters 3–4 (strict CLL dialect) plus
//!   the §4.16 rafsi fu'ivla proposal.
//! - [`MorphOptions::settled()`]: strict, settled CLL — every option off.
//! - [`MorphOptions::camxes()`]: the default plus the camxes dialect
//!   ([`MorphOptions::camxes_dialect`]).
//!
//! The pipeline reads the options of the current thread
//! ([`current_options`]); [`with_options`] and
//! [`super::morph_parse_with`] set them for a computation, so stream
//! segmentation and the Chapter 21 lexer see the same options as the word
//! analysis.

use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MorphOptions {
    /// CLL §4.16 *rafsi fu'ivla* proposal (enabled by default).
    ///
    /// When set, Type-4 fu'ivla of form CCVVCV may contribute a stem rafsi
    /// (final vowel dropped) usable in lujvo, as in `tci'ile` → `tci'il` + `y` + …
    pub rafsi_fuhivla: bool,
    /// The camxes (modern-usage) dialect: one switch for the places where
    /// current usage, as implemented by camxes, goes beyond the CLL text. Off by default (strict CLL), on in [`MorphOptions::camxes`].
    ///
    /// * **Rafsi outside the CLL-era list.** A lujvo may be built from any
    ///   piece of a valid §4.5 rafsi *shape* — CVC, CVV/CV'V (without
    ///   apostrophe only `ai`/`ei`/`oi`/`au`), CCV with a permissible
    ///   initial pair, a 4-letter CVCC/CCVC followed by `y`, and a final
    ///   5-letter CVCCV/CCVCV — still subject to the CLL hyphen (§4.6),
    ///   fall-apart, tosmabru (§4.11.5) and slinku'i (§4.7) rules. CLL fixes
    ///   the 1994 rafsi list; rafsi assigned later have the same forms. When
    ///   off, a lujvo must segment into rafsi of the embedded CLL-era list.
    /// * **Brivla/fu'ivla-stem rafsi** ([`super::camxes_lujvo`]): a
    ///   consonant-final stem + `y`, a vowel-final stem + `'y`, `y'` before a
    ///   vowel-initial piece and a brivla-shaped last piece, as in camxes
    ///   (`afxanyru'u`, `baxmasi'ygu'e`, `pedy'utka`, `bemymantodi`).
    /// * **`la`, `lai`, `doi` inside names.** CLL §4.8 bans them (C19) so
    ///   that `la` + name cannot be misheard; camxes and later usage allow
    ///   them (`volapuk`, `tigbalan`). When off, C19 applies.
    pub camxes_dialect: bool,
}

impl Default for MorphOptions {
    fn default() -> Self {
        Self {
            rafsi_fuhivla: true,
            camxes_dialect: false,
        }
    }
}

impl MorphOptions {
    /// Options restricted to settled CLL morphology: no §4.16 proposal and
    /// the strict CLL dialect (CLL-era rafsi list, §4.8 name ban).
    pub fn settled() -> Self {
        Self {
            rafsi_fuhivla: false,
            camxes_dialect: false,
        }
    }

    /// The camxes dialect: [`MorphOptions::default`] with
    /// [`MorphOptions::camxes_dialect`] on (rafsi by shape, brivla-stem
    /// rafsi, `la`/`lai`/`doi` inside names).
    pub fn camxes() -> Self {
        Self {
            camxes_dialect: true,
            ..Self::default()
        }
    }

    pub fn with_rafsi_fuhivla(mut self, on: bool) -> Self {
        self.rafsi_fuhivla = on;
        self
    }

    pub fn with_camxes_dialect(mut self, on: bool) -> Self {
        self.camxes_dialect = on;
        self
    }
}

thread_local! {
    static CURRENT: Cell<MorphOptions> = Cell::new(MorphOptions::default());
}

/// Options in effect on this thread ([`MorphOptions::default`] unless set by
/// [`with_options`]).
pub fn current_options() -> MorphOptions {
    CURRENT.with(Cell::get)
}

/// Run `f` with `opts` as this thread's options (restored afterwards, also
/// on panic). Everything inside — `morph_parse`, stream segmentation, the
/// Chapter 21 lexer and parser — uses `opts`.
pub fn with_options<R>(opts: MorphOptions, f: impl FnOnce() -> R) -> R {
    struct Restore(MorphOptions);
    impl Drop for Restore {
        fn drop(&mut self) {
            CURRENT.with(|c| c.set(self.0));
        }
    }
    let _restore = Restore(CURRENT.with(|c| c.replace(opts)));
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_options_scopes_and_restores() {
        assert_eq!(current_options(), MorphOptions::default());
        let inner = with_options(MorphOptions::camxes(), current_options);
        assert_eq!(inner, MorphOptions::camxes());
        assert_eq!(current_options(), MorphOptions::default());
    }
}
