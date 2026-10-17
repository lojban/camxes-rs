//! Parse options — CLL features and morphology proposals.

/// Morphology parse options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MorphOptions {
    /// CLL §4.16 *rafsi fu'ivla* proposal (enabled by default).
    ///
    /// When set, Type-4 fu'ivla of form CCVVCV may contribute a stem rafsi
    /// (final vowel dropped) usable in lujvo, as in `tci'ile` → `tci'il` + `y` + …
    pub rafsi_fuhivla: bool,
}

impl Default for MorphOptions {
    fn default() -> Self {
        Self {
            rafsi_fuhivla: true,
        }
    }
}

impl MorphOptions {
    /// Options restricted to settled CLL morphology, without §4.16 proposal.
    pub fn settled() -> Self {
        Self {
            rafsi_fuhivla: false,
        }
    }

    pub fn with_rafsi_fuhivla(mut self, on: bool) -> Self {
        self.rafsi_fuhivla = on;
        self
    }
}
