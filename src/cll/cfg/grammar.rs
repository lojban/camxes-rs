//! Morphology CFG productions — CLL Ch. 3–4 shapes only.
//!
//! Pure CFG: consuming symbols, no lookaround. Ambiguity expected.
//! Pair/triple tables are inlined as finite literal unions (§3.6–3.7).

use super::earley::{Grammar, NtId, Production, Symbol};
use crate::cll::phonology::{
    all_initial_triples, all_medial_pairs, CONSONANTS, DIPHTHONG_NAME_ONLY, DIPHTHONG_OFF,
    DIPHTHONG_ON, INITIAL_PAIRS, VOWELS,
};

/// Named nonterminals used as recognition starts / classification keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShapeNt {
    /// §4.2 simple (+ experimental vowel-heavy) cmavo form
    CmavoSimple,
    /// §4.2 compound cmavo (no CC by construction)
    CmavoCompound,
    /// §4.4 five-letter gismu shape
    Gismu,
    /// §4.5–4.6 rafsi-chain lujvo shape (hyphens overgenerated)
    Lujvo,
    /// §4.7 positive fu'ivla shape (exclusions → constraints)
    Fuhivla,
    /// §4.8 consonant-final name body
    Cmevla,
    /// Any isolated word shape (union)
    Word,
    /// Phonology fragments for fixtures
    MedialPair,
    InitialPair,
    ConsonantTriple,
    DiphthongOff,
    DiphthongOn,
    DiphthongName,
}

impl ShapeNt {
    pub fn as_str(self) -> &'static str {
        match self {
            ShapeNt::CmavoSimple => "CmavoSimple",
            ShapeNt::CmavoCompound => "CmavoCompound",
            ShapeNt::Gismu => "Gismu",
            ShapeNt::Lujvo => "Lujvo",
            ShapeNt::Fuhivla => "Fuhivla",
            ShapeNt::Cmevla => "Cmevla",
            ShapeNt::Word => "Word",
            ShapeNt::MedialPair => "MedialPair",
            ShapeNt::InitialPair => "InitialPair",
            ShapeNt::ConsonantTriple => "ConsonantTriple",
            ShapeNt::DiphthongOff => "DiphthongOff",
            ShapeNt::DiphthongOn => "DiphthongOn",
            ShapeNt::DiphthongName => "DiphthongName",
        }
    }
}

struct Builder {
    names: Vec<&'static str>,
    prods: Vec<Production>,
}

impl Builder {
    fn new() -> Self {
        Self {
            names: Vec::new(),
            prods: Vec::new(),
        }
    }

    fn nt(&mut self, name: &'static str) -> NtId {
        if let Some(i) = self.names.iter().position(|n| *n == name) {
            return i as NtId;
        }
        let id = self.names.len() as NtId;
        self.names.push(name);
        id
    }

    fn prod(&mut self, lhs: NtId, rhs: Vec<Symbol>) {
        self.prods.push(Production { lhs, rhs });
    }

    fn lit(&mut self, lhs: NtId, bytes: &[u8]) {
        self.prod(lhs, bytes.iter().copied().map(Symbol::T).collect());
    }

    fn finish(self) -> Grammar {
        Grammar::new(self.names, self.prods)
    }
}

/// Build the morphology shape grammar (cached via [`morph_grammar`]).
pub fn build_morph_grammar() -> Grammar {
    let mut grammar_builder = Builder::new();

    // --- alphabet ---
    let consonant_nt = grammar_builder.nt("C");
    let vowel_nt = grammar_builder.nt("V");
    let y_letter_nt = grammar_builder.nt("Y");
    let apostrophe_nt = grammar_builder.nt("Apostrophe");
    let pause = grammar_builder.nt("Pause");

    for &ch in CONSONANTS {
        grammar_builder.lit(consonant_nt, &[ch]);
    }
    for &ch in VOWELS {
        grammar_builder.lit(vowel_nt, &[ch]);
    }
    grammar_builder.lit(y_letter_nt, b"y");
    grammar_builder.lit(apostrophe_nt, b"'");
    grammar_builder.lit(pause, b".");

    let comma = grammar_builder.nt("Comma"); // §3.3 syllable break without pause
    grammar_builder.lit(comma, b",");

    // --- diphthongs ---
    let off_glide_diphthong_nt = grammar_builder.nt("DiphthongOff");
    let on_glide_diphthong_nt = grammar_builder.nt("DiphthongOn");
    let name_only_diphthong_nt = grammar_builder.nt("DiphthongName");
    for diphthong in DIPHTHONG_OFF {
        grammar_builder.lit(off_glide_diphthong_nt, diphthong.as_bytes());
    }
    for diphthong in DIPHTHONG_ON {
        grammar_builder.lit(on_glide_diphthong_nt, diphthong.as_bytes());
    }
    for diphthong in DIPHTHONG_NAME_ONLY {
        grammar_builder.lit(name_only_diphthong_nt, diphthong.as_bytes());
    }

    let vowel_pair_nt = grammar_builder.nt("VowelPair"); // V ' V  §3.5
    grammar_builder.prod(
        vowel_pair_nt,
        vec![
            Symbol::Nt(vowel_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(vowel_nt),
        ],
    );

    // --- clusters ---
    let initial_consonant_pair_nt = grammar_builder.nt("InitialCc");
    for diphthong in INITIAL_PAIRS {
        grammar_builder.lit(initial_consonant_pair_nt, diphthong.as_bytes());
    }

    let medial_consonant_pair_nt = grammar_builder.nt("MedialCc");
    for (first_consonant, third_consonant) in all_medial_pairs() {
        grammar_builder.lit(
            medial_consonant_pair_nt,
            &[first_consonant, third_consonant],
        );
    }

    let medial_pair_nt = grammar_builder.nt("MedialPair");
    grammar_builder.prod(medial_pair_nt, vec![Symbol::Nt(medial_consonant_pair_nt)]);

    let initial_pair_nt = grammar_builder.nt("InitialPair");
    grammar_builder.prod(initial_pair_nt, vec![Symbol::Nt(initial_consonant_pair_nt)]);

    let consonant_triple_nt = grammar_builder.nt("ConsonantTriple");
    // Encode via MedialCc + following C that makes initial pair — too loose.
    // Instead: enumerate permissible triples only.
    for first_consonant in CONSONANTS {
        for second_consonant in CONSONANTS {
            for third_consonant in CONSONANTS {
                if crate::cll::phonology::is_permissible_triple(
                    *first_consonant,
                    *second_consonant,
                    *third_consonant,
                ) {
                    grammar_builder.lit(
                        consonant_triple_nt,
                        &[*first_consonant, *second_consonant, *third_consonant],
                    );
                }
            }
        }
    }

    let initial_triple_nt = grammar_builder.nt("InitialTriple"); // §4.7.1 adjacent-initial
    for (first_consonant, second_consonant, third_consonant) in all_initial_triples() {
        grammar_builder.lit(
            initial_triple_nt,
            &[first_consonant, second_consonant, third_consonant],
        );
    }

    // Longer initial clusters (4+): InitialCc chained — e.g. optional for overgen
    let initial_cluster_nt = grammar_builder.nt("InitialCluster");
    grammar_builder.prod(
        initial_cluster_nt,
        vec![Symbol::Nt(initial_consonant_pair_nt)],
    );
    grammar_builder.prod(initial_cluster_nt, vec![Symbol::Nt(initial_triple_nt)]);
    // 4-consonant: InitialTriple + C where last pair initial
    // Keep to 2–3 for milestone; longer rare in CLL fixtures.

    // --- nuclei ---
    let nucleus_cmavo = grammar_builder.nt("NucleusCmavo"); // V | off | on | V'V
    grammar_builder.prod(nucleus_cmavo, vec![Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(nucleus_cmavo, vec![Symbol::Nt(off_glide_diphthong_nt)]);
    grammar_builder.prod(nucleus_cmavo, vec![Symbol::Nt(on_glide_diphthong_nt)]);
    grammar_builder.prod(nucleus_cmavo, vec![Symbol::Nt(vowel_pair_nt)]);

    let nucleus_brivla = grammar_builder.nt("NucleusBrivla"); // V | off | on | V'V  (borrowings may use on-glides)
    grammar_builder.prod(nucleus_brivla, vec![Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(nucleus_brivla, vec![Symbol::Nt(off_glide_diphthong_nt)]);
    grammar_builder.prod(nucleus_brivla, vec![Symbol::Nt(on_glide_diphthong_nt)]);
    grammar_builder.prod(nucleus_brivla, vec![Symbol::Nt(vowel_pair_nt)]);

    // §3.5: "When more than two vowels occur together in Lojban, the normal
    // pronunciation pairs vowels from the left into syllables" (example: a
    // diphthong followed by a vowel). In borrowings this admits runs such as
    // `ie|i` or `ue|ia`: a diphthong followed by a vowel or another diphthong.
    let diphthong_nt = grammar_builder.nt("Diphthong");
    grammar_builder.prod(diphthong_nt, vec![Symbol::Nt(off_glide_diphthong_nt)]);
    grammar_builder.prod(diphthong_nt, vec![Symbol::Nt(on_glide_diphthong_nt)]);
    let vowel_run_nt = grammar_builder.nt("VowelRun");
    grammar_builder.prod(
        vowel_run_nt,
        vec![Symbol::Nt(diphthong_nt), Symbol::Nt(vowel_nt)],
    );
    grammar_builder.prod(
        vowel_run_nt,
        vec![Symbol::Nt(diphthong_nt), Symbol::Nt(diphthong_nt)],
    );
    grammar_builder.prod(
        vowel_run_nt,
        vec![Symbol::Nt(diphthong_nt), Symbol::Nt(vowel_run_nt)],
    );
    // The apostrophe separates two syllables (§3.5); since a diphthong counts
    // as a vowel for syllables (§3.9), `ii'i` in a borrowing is `ii` + `'i`
    // and `u'ui` is `u` + `'ui`.
    grammar_builder.prod(
        vowel_run_nt,
        vec![
            Symbol::Nt(diphthong_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(vowel_nt),
        ],
    );
    grammar_builder.prod(
        vowel_run_nt,
        vec![
            Symbol::Nt(vowel_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(diphthong_nt),
        ],
    );
    grammar_builder.prod(nucleus_brivla, vec![Symbol::Nt(vowel_run_nt)]);
    // General left-grouped vowel runs (§3.5): within an apostrophe-free
    // stretch the vowels pair from the left, so a stretch is a sequence of
    // diphthongs optionally ending in one vowel (`ai|a`, `uu|e`, `iu|au|a`).
    // A non-diphthong adjacent pair (`eu`, `ea`) has no reading here: it needs
    // a comma or an apostrophe ("In order to indicate a different grouping,
    // the comma must always be used"). Which diphthongs a word class may use
    // (§3.4) is unchanged: this nucleus is only used by fu'ivla, where
    // on-glides are allowed ("borrowings").
    let vowel_stretch_nt = grammar_builder.nt("VowelStretch");
    grammar_builder.prod(vowel_stretch_nt, vec![Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(vowel_stretch_nt, vec![Symbol::Nt(diphthong_nt)]);
    grammar_builder.prod(
        vowel_stretch_nt,
        vec![Symbol::Nt(diphthong_nt), Symbol::Nt(vowel_stretch_nt)],
    );
    let vowel_group_run_nt = grammar_builder.nt("VowelGroupRun");
    grammar_builder.prod(vowel_group_run_nt, vec![Symbol::Nt(vowel_stretch_nt)]);
    grammar_builder.prod(
        vowel_group_run_nt,
        vec![
            Symbol::Nt(vowel_stretch_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(vowel_group_run_nt),
        ],
    );
    grammar_builder.prod(nucleus_brivla, vec![Symbol::Nt(vowel_group_run_nt)]);
    // A run of at least two syllables, so that it can be a fu'ivla's only
    // vowel group (`blaia` = `blai|a`).
    let vowel_run_multi_nt = grammar_builder.nt("VowelRunMulti");
    grammar_builder.prod(
        vowel_run_multi_nt,
        vec![Symbol::Nt(diphthong_nt), Symbol::Nt(vowel_stretch_nt)],
    );
    grammar_builder.prod(
        vowel_run_multi_nt,
        vec![
            Symbol::Nt(vowel_stretch_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(vowel_group_run_nt),
        ],
    );
    // Apostrophe chains `N'N'N…` of vowels and off-glide diphthongs inside
    // brivla (`banbu'a'a`, `krai'a`). CLL §3.4: "all vowel pairs are allowed
    // with an apostrophe between them"; nothing in §4 limits the length.
    let brivla_chain_nucleus = grammar_builder.nt("BrivlaChainNucleus");
    grammar_builder.prod(brivla_chain_nucleus, vec![Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(
        brivla_chain_nucleus,
        vec![Symbol::Nt(off_glide_diphthong_nt)],
    );
    let brivla_chain = grammar_builder.nt("BrivlaChain");
    grammar_builder.prod(
        brivla_chain,
        vec![
            Symbol::Nt(brivla_chain_nucleus),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(brivla_chain_nucleus),
        ],
    );
    grammar_builder.prod(
        brivla_chain,
        vec![
            Symbol::Nt(brivla_chain_nucleus),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(brivla_chain),
        ],
    );
    grammar_builder.prod(nucleus_brivla, vec![Symbol::Nt(brivla_chain)]);

    let nucleus_cmevla = grammar_builder.nt("NucleusCmevla");
    grammar_builder.prod(nucleus_cmevla, vec![Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(nucleus_cmevla, vec![Symbol::Nt(off_glide_diphthong_nt)]);
    grammar_builder.prod(nucleus_cmevla, vec![Symbol::Nt(on_glide_diphthong_nt)]);
    grammar_builder.prod(nucleus_cmevla, vec![Symbol::Nt(name_only_diphthong_nt)]);
    grammar_builder.prod(nucleus_cmevla, vec![Symbol::Nt(vowel_pair_nt)]);
    grammar_builder.prod(nucleus_cmevla, vec![Symbol::Nt(y_letter_nt)]);

    // --- cmavo §4.2 ---
    let cmavo_simple = grammar_builder.nt("CmavoSimple");
    // Word-boundary pauses are handled outside the shape CFG.
    grammar_builder.prod(cmavo_simple, vec![Symbol::Nt(vowel_nt)]);
    // CV
    grammar_builder.prod(
        cmavo_simple,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(vowel_nt)],
    );
    // VV-form: off-glide / on-glide / V'V
    grammar_builder.prod(cmavo_simple, vec![Symbol::Nt(off_glide_diphthong_nt)]);
    grammar_builder.prod(cmavo_simple, vec![Symbol::Nt(on_glide_diphthong_nt)]);
    grammar_builder.prod(cmavo_simple, vec![Symbol::Nt(vowel_pair_nt)]);
    // CVV
    grammar_builder.prod(
        cmavo_simple,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(off_glide_diphthong_nt)],
    );
    grammar_builder.prod(
        cmavo_simple,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(on_glide_diphthong_nt)],
    );
    grammar_builder.prod(
        cmavo_simple,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(vowel_pair_nt)],
    );
    // Cy
    grammar_builder.prod(
        cmavo_simple,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(y_letter_nt)],
    );
    // The isolated hesitation cmavo has body y; C06 checks its two pauses.
    grammar_builder.prod(cmavo_simple, vec![Symbol::Nt(y_letter_nt)]);
    // CLL §3.5 / §17.2 apostrophe letteral `.y'y` (BY): Y ' Y nucleus.
    // Boundary pauses around this letteral are checked by the constraint layer.
    let yhy = grammar_builder.nt("Yhy"); // Y ' Y
    grammar_builder.prod(
        yhy,
        vec![
            Symbol::Nt(y_letter_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(y_letter_nt),
        ],
    );
    grammar_builder.prod(cmavo_simple, vec![Symbol::Nt(yhy)]);

    // Experimental cmavo: ≥3 vowel-slots, no CC — §4.2
    // VPart = V | DiphOff | DiphOn | V'V
    let vpart = grammar_builder.nt("VPart");
    grammar_builder.prod(vpart, vec![Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(vpart, vec![Symbol::Nt(off_glide_diphthong_nt)]);
    grammar_builder.prod(vpart, vec![Symbol::Nt(on_glide_diphthong_nt)]);
    grammar_builder.prod(vpart, vec![Symbol::Nt(vowel_pair_nt)]);
    // Also allow 'V as extension after a nucleus (ku'a'e = C V ' V ' V)
    let vext = grammar_builder.nt("VExt");
    grammar_builder.prod(vext, vec![Symbol::Nt(apostrophe_nt), Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(
        vext,
        vec![
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(off_glide_diphthong_nt),
        ],
    );
    grammar_builder.prod(
        vext,
        vec![Symbol::Nt(apostrophe_nt), Symbol::Nt(on_glide_diphthong_nt)],
    );

    let exp_cmavo = grammar_builder.nt("ExpCmavo");
    let exp_cmavo_c = grammar_builder.nt("ExpCmavoC");
    // C VPart VExt VExt+   (ku'a'e, ki'e'u'e)
    let vexts = grammar_builder.nt("VExts");
    grammar_builder.prod(vexts, vec![Symbol::Nt(vext)]);
    grammar_builder.prod(vexts, vec![Symbol::Nt(vext), Symbol::Nt(vexts)]);
    grammar_builder.prod(
        exp_cmavo_c,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(vpart),
            Symbol::Nt(vext),
            Symbol::Nt(vexts),
        ],
    );
    // C VPart VPart with enough vowels: bai'ai = C DiphOff ' DiphOff
    grammar_builder.prod(
        exp_cmavo_c,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(off_glide_diphthong_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(off_glide_diphthong_nt),
        ],
    );
    grammar_builder.prod(
        exp_cmavo_c,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(on_glide_diphthong_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(off_glide_diphthong_nt),
        ],
    );
    grammar_builder.prod(
        exp_cmavo_c,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(off_glide_diphthong_nt),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(on_glide_diphthong_nt),
        ],
    );
    // sau'e = C DiphOff? actually s-au-'e = C diph_off + ' + V — two vowel groups + ext
    // Treat as C VPart VExt (only 2 vowel nuclei) — CLL lists sau'e as experimental
    // with "additional vowels"; sau'e has au + e = enough. Allow C VPart VExt as experimental too
    // when VPart is s or V'V (already ≥2 vowels) + another V.
    grammar_builder.prod(
        exp_cmavo_c,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(off_glide_diphthong_nt),
            Symbol::Nt(vext),
        ],
    );
    grammar_builder.prod(
        exp_cmavo_c,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(on_glide_diphthong_nt),
            Symbol::Nt(vext),
        ],
    );
    grammar_builder.prod(
        exp_cmavo_c,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(vowel_pair_nt),
            Symbol::Nt(vext),
        ],
    );
    // V-initial experimental body
    grammar_builder.prod(
        exp_cmavo,
        vec![Symbol::Nt(vpart), Symbol::Nt(vext), Symbol::Nt(vexts)],
    );
    grammar_builder.prod(exp_cmavo, vec![Symbol::Nt(exp_cmavo_c)]);

    grammar_builder.prod(cmavo_simple, vec![Symbol::Nt(exp_cmavo)]);

    // Experimental cmavo `onset? N ('N)*` where a nucleus N is a vowel, an
    // off-glide diphthong or `y`, and the onset is one consonant or an
    // on-glide `i`/`u` (e.g. `ba'au`, `iau`, `ty'i`, `a'y`). CLL §4.2: forms
    // with "three or more vowels in a row … are also of cmavo form, but are
    // reserved for experimental use"; §3.4: on-glides occur in stand-alone
    // words; `y` with an apostrophe forms lerfu words (Ch. 17, `y'y`).
    let chain_nucleus = grammar_builder.nt("ChainNucleus");
    grammar_builder.prod(chain_nucleus, vec![Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(chain_nucleus, vec![Symbol::Nt(off_glide_diphthong_nt)]);
    grammar_builder.prod(chain_nucleus, vec![Symbol::Nt(y_letter_nt)]);
    let chain_nuclei = grammar_builder.nt("ChainNuclei");
    grammar_builder.prod(chain_nuclei, vec![Symbol::Nt(chain_nucleus)]);
    grammar_builder.prod(
        chain_nuclei,
        vec![
            Symbol::Nt(chain_nucleus),
            Symbol::Nt(apostrophe_nt),
            Symbol::Nt(chain_nuclei),
        ],
    );
    let chain_glide = grammar_builder.nt("ChainGlideOnset");
    grammar_builder.lit(chain_glide, b"i");
    grammar_builder.lit(chain_glide, b"u");
    let chain_cmavo_c = grammar_builder.nt("ChainCmavoC");
    grammar_builder.prod(
        chain_cmavo_c,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(chain_nuclei)],
    );
    let chain_cmavo = grammar_builder.nt("ChainCmavo");
    grammar_builder.prod(chain_cmavo, vec![Symbol::Nt(chain_nuclei)]);
    grammar_builder.prod(
        chain_cmavo,
        vec![Symbol::Nt(chain_glide), Symbol::Nt(chain_nuclei)],
    );
    grammar_builder.prod(chain_cmavo, vec![Symbol::Nt(chain_cmavo_c)]);
    grammar_builder.prod(cmavo_simple, vec![Symbol::Nt(chain_cmavo)]);

    // Compound cmavo: sequence of simple pieces; pauses between vowel-initial pieces
    let cmavo_core_c = grammar_builder.nt("CmavoCoreC");
    let cmavo_core = grammar_builder.nt("CmavoCore");
    grammar_builder.prod(cmavo_core, vec![Symbol::Nt(vowel_nt)]);
    grammar_builder.prod(
        cmavo_core_c,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(vowel_nt)],
    );
    grammar_builder.prod(cmavo_core, vec![Symbol::Nt(off_glide_diphthong_nt)]);
    grammar_builder.prod(cmavo_core, vec![Symbol::Nt(on_glide_diphthong_nt)]);
    grammar_builder.prod(cmavo_core, vec![Symbol::Nt(vowel_pair_nt)]);
    grammar_builder.prod(
        cmavo_core_c,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(off_glide_diphthong_nt)],
    );
    grammar_builder.prod(
        cmavo_core_c,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(on_glide_diphthong_nt)],
    );
    grammar_builder.prod(
        cmavo_core_c,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(vowel_pair_nt)],
    );
    grammar_builder.prod(
        cmavo_core_c,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(y_letter_nt)],
    );
    // Apostrophe letteral nucleus for compounds e.g. `.y'y.bu` (§17.2)
    grammar_builder.prod(cmavo_core, vec![Symbol::Nt(yhy)]);
    grammar_builder.prod(cmavo_core, vec![Symbol::Nt(cmavo_core_c)]);
    grammar_builder.prod(cmavo_core, vec![Symbol::Nt(exp_cmavo)]);
    grammar_builder.prod(cmavo_core_c, vec![Symbol::Nt(exp_cmavo_c)]);
    grammar_builder.prod(cmavo_core, vec![Symbol::Nt(chain_cmavo)]);
    grammar_builder.prod(cmavo_core_c, vec![Symbol::Nt(chain_cmavo_c)]);

    // A following V-initial cmavo needs an explicit pause (§4.2). The first
    // piece can be V-initial without a written dot; its boundary is external.
    let cmavo_next = grammar_builder.nt("CmavoNext");
    grammar_builder.prod(cmavo_next, vec![Symbol::Nt(cmavo_core_c)]);
    grammar_builder.prod(cmavo_next, vec![Symbol::Nt(pause), Symbol::Nt(cmavo_core)]);

    let cmavo_compound = grammar_builder.nt("CmavoCompound");
    // ≥2 pieces
    let cmavo_tail = grammar_builder.nt("CmavoTail");
    grammar_builder.prod(cmavo_tail, vec![Symbol::Nt(cmavo_next)]);
    grammar_builder.prod(
        cmavo_tail,
        vec![Symbol::Nt(cmavo_next), Symbol::Nt(cmavo_tail)],
    );
    grammar_builder.prod(
        cmavo_compound,
        vec![Symbol::Nt(cmavo_core), Symbol::Nt(cmavo_tail)],
    );
    // Pause in a compound is an internal boundary between its cmavo pieces.

    // --- gismu §4.4 ---
    let gismu = grammar_builder.nt("Gismu");
    // CVC/CV
    grammar_builder.prod(
        gismu,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(vowel_nt),
            Symbol::Nt(medial_consonant_pair_nt),
            Symbol::Nt(vowel_nt),
        ],
    );
    // CCVCV
    grammar_builder.prod(
        gismu,
        vec![
            Symbol::Nt(initial_consonant_pair_nt),
            Symbol::Nt(vowel_nt),
            Symbol::Nt(consonant_nt),
            Symbol::Nt(vowel_nt),
        ],
    );

    // --- rafsi / lujvo §4.5–4.6 ---
    let rafsi_cvc = grammar_builder.nt("RafsiCvc");
    grammar_builder.prod(
        rafsi_cvc,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(vowel_nt),
            Symbol::Nt(consonant_nt),
        ],
    );

    let rafsi_ccv = grammar_builder.nt("RafsiCcv");
    grammar_builder.prod(
        rafsi_ccv,
        vec![Symbol::Nt(initial_consonant_pair_nt), Symbol::Nt(vowel_nt)],
    );

    let rafsi_cvv = grammar_builder.nt("RafsiCvv");
    grammar_builder.prod(
        rafsi_cvv,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(off_glide_diphthong_nt)],
    );
    grammar_builder.prod(
        rafsi_cvv,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(vowel_pair_nt)],
    );

    let rafsi4 = grammar_builder.nt("Rafsi4");
    // CVC/C
    grammar_builder.prod(
        rafsi4,
        vec![
            Symbol::Nt(consonant_nt),
            Symbol::Nt(vowel_nt),
            Symbol::Nt(medial_consonant_pair_nt),
        ],
    );
    // CCVC
    grammar_builder.prod(
        rafsi4,
        vec![
            Symbol::Nt(initial_consonant_pair_nt),
            Symbol::Nt(vowel_nt),
            Symbol::Nt(consonant_nt),
        ],
    );

    let hyphen = grammar_builder.nt("Hyphen"); // y | r | n  (l not in lujvo)
    grammar_builder.lit(hyphen, b"y");
    grammar_builder.lit(hyphen, b"r");
    grammar_builder.lit(hyphen, b"n");

    let rafsi_nf = grammar_builder.nt("RafsiNonFinal");
    grammar_builder.prod(rafsi_nf, vec![Symbol::Nt(rafsi_cvc)]);
    grammar_builder.prod(rafsi_nf, vec![Symbol::Nt(rafsi_cvc), Symbol::Nt(hyphen)]);
    grammar_builder.prod(rafsi_nf, vec![Symbol::Nt(rafsi_ccv)]);
    grammar_builder.prod(rafsi_nf, vec![Symbol::Nt(rafsi_ccv), Symbol::Nt(hyphen)]);
    grammar_builder.prod(rafsi_nf, vec![Symbol::Nt(rafsi_cvv)]);
    grammar_builder.prod(rafsi_nf, vec![Symbol::Nt(rafsi_cvv), Symbol::Nt(hyphen)]);
    grammar_builder.prod(rafsi_nf, vec![Symbol::Nt(rafsi4)]);
    grammar_builder.prod(rafsi_nf, vec![Symbol::Nt(rafsi4), Symbol::Nt(hyphen)]);

    let rafsi_final = grammar_builder.nt("RafsiFinal");
    grammar_builder.prod(rafsi_final, vec![Symbol::Nt(rafsi_ccv)]);
    grammar_builder.prod(rafsi_final, vec![Symbol::Nt(rafsi_cvv)]);
    grammar_builder.prod(rafsi_final, vec![Symbol::Nt(gismu)]);

    let rafsi_nfs = grammar_builder.nt("RafsiNFs");
    grammar_builder.prod(rafsi_nfs, vec![Symbol::Nt(rafsi_nf)]);
    grammar_builder.prod(rafsi_nfs, vec![Symbol::Nt(rafsi_nf), Symbol::Nt(rafsi_nfs)]);

    let lujvo = grammar_builder.nt("Lujvo");
    grammar_builder.prod(lujvo, vec![Symbol::Nt(rafsi_nfs), Symbol::Nt(rafsi_final)]);

    // --- fu'ivla coarse §4.7 ---
    // Onset for non-initial syllables: ε | C | MedialCc-as-coda+C handled via cluster coda model.
    // Simpler syllable model:
    //   Body = (CodaCluster)? NucleusBrivla  but better:
    // Stream of: Nucleus and C/CC/CCC with constraints.

    // Approach: Fuhivla = Pause? Onset Nucleus (MedialLink Nucleus)+
    // where MedialLink is C | MedialCc | permissible triple letters
    // and initial Onset is ε | C | InitialCluster
    // No Y anywhere.
    // Must end in vowel (nucleus does).
    // ≥2 nuclei.

    let f_onset = grammar_builder.nt("FOnset");
    grammar_builder.prod(f_onset, vec![]); // ε (vowel-initial, usually after pause)
    grammar_builder.prod(f_onset, vec![Symbol::Nt(consonant_nt)]);
    grammar_builder.prod(f_onset, vec![Symbol::Nt(initial_cluster_nt)]);

    let f_link = grammar_builder.nt("FLink"); // between nuclei: one or more C with medial/triple OK
    grammar_builder.prod(f_link, vec![Symbol::Nt(consonant_nt)]);
    grammar_builder.prod(f_link, vec![Symbol::Nt(medial_consonant_pair_nt)]);
    grammar_builder.prod(f_link, vec![Symbol::Nt(consonant_triple_nt)]);
    grammar_builder.prod(f_link, vec![Symbol::Nt(comma)]);
    let f_cplus = grammar_builder.nt("FCPlus");
    grammar_builder.prod(f_cplus, vec![Symbol::Nt(consonant_nt)]);
    grammar_builder.prod(f_cplus, vec![Symbol::Nt(consonant_nt), Symbol::Nt(f_cplus)]);
    grammar_builder.prod(f_link, vec![Symbol::Nt(f_cplus)]);

    let f_rest = grammar_builder.nt("FRest");
    grammar_builder.prod(f_rest, vec![Symbol::Nt(f_link), Symbol::Nt(nucleus_brivla)]);
    grammar_builder.prod(
        f_rest,
        vec![
            Symbol::Nt(f_link),
            Symbol::Nt(nucleus_brivla),
            Symbol::Nt(f_rest),
        ],
    );

    let fuhivla = grammar_builder.nt("Fuhivla");
    grammar_builder.prod(
        fuhivla,
        vec![
            Symbol::Nt(f_onset),
            Symbol::Nt(nucleus_brivla),
            Symbol::Nt(f_rest),
        ],
    );

    // Stage-3 shape: 4-letter rafsi + l/r/n hyphen + fu'ivla-like body (§4.7)
    let stage3_hyphen_letter = grammar_builder.nt("Stage3HyphenLetter");
    grammar_builder.lit(stage3_hyphen_letter, b"r");
    grammar_builder.lit(stage3_hyphen_letter, b"n");
    grammar_builder.lit(stage3_hyphen_letter, b"l");
    // Orthographic commas around hyphen (§3.3 / §4.7 examples: tric,r,xaceru)
    let stage3_hyphen = grammar_builder.nt("Stage3Hyphen");
    grammar_builder.prod(stage3_hyphen, vec![Symbol::Nt(stage3_hyphen_letter)]);
    grammar_builder.prod(
        stage3_hyphen,
        vec![
            Symbol::Nt(comma),
            Symbol::Nt(stage3_hyphen_letter),
            Symbol::Nt(comma),
        ],
    );
    grammar_builder.prod(
        stage3_hyphen,
        vec![Symbol::Nt(comma), Symbol::Nt(stage3_hyphen_letter)],
    );
    grammar_builder.prod(
        stage3_hyphen,
        vec![Symbol::Nt(stage3_hyphen_letter), Symbol::Nt(comma)],
    );

    let fuhivla_s3 = grammar_builder.nt("FuhivlaS3");
    // Classifier: 4-letter OR short CCV/CVC rafsi (§4.7 examples: dja,r,spageti / ric,r,xaceru)
    let s3_rafsi = grammar_builder.nt("S3Rafsi");
    grammar_builder.prod(s3_rafsi, vec![Symbol::Nt(rafsi4)]);
    grammar_builder.prod(s3_rafsi, vec![Symbol::Nt(rafsi_ccv)]);
    grammar_builder.prod(s3_rafsi, vec![Symbol::Nt(rafsi_cvc)]);

    // Body after hyphen: freer onset (not word-initial — pairwise clusters OK)
    let s3_body = grammar_builder.nt("S3Body");
    grammar_builder.prod(
        s3_body,
        vec![
            Symbol::Nt(f_onset),
            Symbol::Nt(nucleus_brivla),
            Symbol::Nt(f_rest),
        ],
    );
    grammar_builder.prod(
        s3_body,
        vec![Symbol::Nt(f_onset), Symbol::Nt(nucleus_brivla)],
    );
    grammar_builder.prod(
        s3_body,
        vec![
            Symbol::Nt(f_cplus),
            Symbol::Nt(nucleus_brivla),
            Symbol::Nt(f_rest),
        ],
    );
    grammar_builder.prod(
        s3_body,
        vec![Symbol::Nt(f_cplus), Symbol::Nt(nucleus_brivla)],
    );
    grammar_builder.prod(
        s3_body,
        vec![
            Symbol::Nt(initial_cluster_nt),
            Symbol::Nt(nucleus_brivla),
            Symbol::Nt(f_rest),
        ],
    );
    grammar_builder.prod(
        s3_body,
        vec![Symbol::Nt(initial_cluster_nt), Symbol::Nt(nucleus_brivla)],
    );
    // vowel-initial body after hyphen/comma
    grammar_builder.prod(
        s3_body,
        vec![Symbol::Nt(nucleus_brivla), Symbol::Nt(f_rest)],
    );
    grammar_builder.prod(s3_body, vec![Symbol::Nt(nucleus_brivla)]);

    grammar_builder.prod(
        fuhivla_s3,
        vec![
            Symbol::Nt(s3_rafsi),
            Symbol::Nt(stage3_hyphen),
            Symbol::Nt(s3_body),
        ],
    );

    grammar_builder.prod(fuhivla, vec![Symbol::Nt(fuhivla_s3)]);
    // A fu'ivla whose only vowels form one apostrophe chain (`krai'a`,
    // `ska'ai`): §4.7 asks only for a brivla shape with a cluster in the
    // first five letters; §3.4 allows any vowel pair across an apostrophe.
    grammar_builder.prod(fuhivla, vec![Symbol::Nt(f_onset), Symbol::Nt(brivla_chain)]);
    // A fu'ivla whose only vowels are one left-grouped run of two or more
    // syllables (`blaia` = `blai|a`, `gruue` = `gruu|e`, `tsauai` =
    // `tsau|ai`); §3.5 vowel grouping, §4.7 rules 1–5 otherwise unchanged.
    grammar_builder.prod(
        fuhivla,
        vec![Symbol::Nt(f_onset), Symbol::Nt(vowel_run_multi_nt)],
    );

    // --- cmevla §4.8 ---
    // Consonant-final; pairs medial-OK (enforced by phonotactics filter);
    // may include y / name diphthongs / syllabic consonants and commas.
    let syllabic_c = grammar_builder.nt("SyllabicC"); // l m n r as syllabic nuclei in names §3.4
    for ch in b"lmnr" {
        grammar_builder.lit(syllabic_c, &[*ch]);
    }

    let c_nucleus = grammar_builder.nt("CNucleus");
    grammar_builder.prod(c_nucleus, vec![Symbol::Nt(nucleus_cmevla)]);
    grammar_builder.prod(c_nucleus, vec![Symbol::Nt(syllabic_c)]);
    // Extended V'V'V… chains in names (.i'ai'i'ai'on.)
    grammar_builder.prod(
        c_nucleus,
        vec![Symbol::Nt(vpart), Symbol::Nt(vext), Symbol::Nt(vexts)],
    );
    grammar_builder.prod(c_nucleus, vec![Symbol::Nt(vpart), Symbol::Nt(vext)]);
    // Apostrophe chains with `y` in names (`sa'ydar`, `cy'i'am`). CLL §4.8:
    // names "may have almost any form"; they may contain `y` as a vowel.
    grammar_builder.prod(c_nucleus, vec![Symbol::Nt(chain_nuclei)]);
    // comma-separated nucleus (disambiguation) §3.3
    grammar_builder.prod(
        c_nucleus,
        vec![Symbol::Nt(nucleus_cmevla), Symbol::Nt(comma)],
    );
    grammar_builder.prod(
        c_nucleus,
        vec![Symbol::Nt(comma), Symbol::Nt(nucleus_cmevla)],
    );

    let c_onset = grammar_builder.nt("COnset");
    grammar_builder.prod(c_onset, vec![]); // ε
    grammar_builder.prod(c_onset, vec![Symbol::Nt(consonant_nt)]);
    grammar_builder.prod(c_onset, vec![Symbol::Nt(medial_consonant_pair_nt)]);
    grammar_builder.prod(c_onset, vec![Symbol::Nt(consonant_triple_nt)]);
    grammar_builder.prod(c_onset, vec![Symbol::Nt(initial_consonant_pair_nt)]);
    // longer name-initial clusters: MedialCc / Triple already cover 2–3;
    // allow C+ greedily via CCluster
    let c_cluster = grammar_builder.nt("CCluster");
    grammar_builder.prod(c_cluster, vec![Symbol::Nt(consonant_nt)]);
    grammar_builder.prod(
        c_cluster,
        vec![Symbol::Nt(consonant_nt), Symbol::Nt(c_cluster)],
    );
    grammar_builder.prod(c_onset, vec![Symbol::Nt(c_cluster)]);

    let c_link = grammar_builder.nt("CLink");
    grammar_builder.prod(c_link, vec![]); // ε — vowel hiatus / s sequencing in names
    grammar_builder.prod(c_link, vec![Symbol::Nt(consonant_nt)]);
    grammar_builder.prod(c_link, vec![Symbol::Nt(medial_consonant_pair_nt)]);
    grammar_builder.prod(c_link, vec![Symbol::Nt(consonant_triple_nt)]);
    grammar_builder.prod(c_link, vec![Symbol::Nt(c_cluster)]);
    grammar_builder.prod(c_link, vec![Symbol::Nt(comma)]); // comma break

    let c_coda = grammar_builder.nt("CCoda");
    grammar_builder.prod(c_coda, vec![Symbol::Nt(consonant_nt)]);
    grammar_builder.prod(c_coda, vec![Symbol::Nt(medial_consonant_pair_nt)]);
    grammar_builder.prod(c_coda, vec![Symbol::Nt(consonant_triple_nt)]);
    grammar_builder.prod(c_coda, vec![Symbol::Nt(c_cluster)]);

    let cmevla_inner = grammar_builder.nt("CmevlaInner");
    // onset nucleus coda
    grammar_builder.prod(
        cmevla_inner,
        vec![
            Symbol::Nt(c_onset),
            Symbol::Nt(c_nucleus),
            Symbol::Nt(c_coda),
        ],
    );
    // multiple nuclei
    let c_mid = grammar_builder.nt("CMid");
    grammar_builder.prod(c_mid, vec![Symbol::Nt(c_link), Symbol::Nt(c_nucleus)]);
    grammar_builder.prod(
        c_mid,
        vec![Symbol::Nt(c_link), Symbol::Nt(c_nucleus), Symbol::Nt(c_mid)],
    );
    grammar_builder.prod(
        cmevla_inner,
        vec![
            Symbol::Nt(c_onset),
            Symbol::Nt(c_nucleus),
            Symbol::Nt(c_mid),
            Symbol::Nt(c_coda),
        ],
    );
    // pure syllabic-consonant name: .rl. / brlgan. style — cluster as both body and coda
    grammar_builder.prod(cmevla_inner, vec![Symbol::Nt(c_cluster)]);
    grammar_builder.prod(
        cmevla_inner,
        vec![
            Symbol::Nt(c_cluster),
            Symbol::Nt(c_nucleus),
            Symbol::Nt(c_coda),
        ],
    );
    grammar_builder.prod(
        cmevla_inner,
        vec![
            Symbol::Nt(c_cluster),
            Symbol::Nt(c_nucleus),
            Symbol::Nt(c_mid),
            Symbol::Nt(c_coda),
        ],
    );

    // Comma-joined name parts: .arm,strong. / kat,r,in.
    let cmevla_parts = grammar_builder.nt("CmevlaParts");
    grammar_builder.prod(cmevla_parts, vec![Symbol::Nt(cmevla_inner)]);
    grammar_builder.prod(
        cmevla_parts,
        vec![
            Symbol::Nt(cmevla_inner),
            Symbol::Nt(comma),
            Symbol::Nt(cmevla_parts),
        ],
    );
    // also allow bare cluster pieces between commas (kat,r,in)
    grammar_builder.prod(
        cmevla_parts,
        vec![
            Symbol::Nt(c_cluster),
            Symbol::Nt(comma),
            Symbol::Nt(cmevla_parts),
        ],
    );
    grammar_builder.prod(
        cmevla_parts,
        vec![
            Symbol::Nt(cmevla_inner),
            Symbol::Nt(comma),
            Symbol::Nt(c_cluster),
        ],
    );

    let cmevla = grammar_builder.nt("Cmevla");
    grammar_builder.prod(cmevla, vec![Symbol::Nt(cmevla_inner)]);
    grammar_builder.prod(cmevla, vec![Symbol::Nt(cmevla_parts)]);

    // --- Word union ---
    let word = grammar_builder.nt("Word");
    grammar_builder.prod(word, vec![Symbol::Nt(cmavo_simple)]);
    grammar_builder.prod(word, vec![Symbol::Nt(cmavo_compound)]);
    grammar_builder.prod(word, vec![Symbol::Nt(gismu)]);
    grammar_builder.prod(word, vec![Symbol::Nt(lujvo)]);
    grammar_builder.prod(word, vec![Symbol::Nt(fuhivla)]);
    grammar_builder.prod(word, vec![Symbol::Nt(cmevla)]);

    grammar_builder.finish()
}

use std::sync::OnceLock;

static MORPH_GRAMMAR: OnceLock<Grammar> = OnceLock::new();

pub fn morph_grammar() -> &'static Grammar {
    MORPH_GRAMMAR.get_or_init(build_morph_grammar)
}

pub fn shape_nt_id(grammar: &Grammar, nonterminal: ShapeNt) -> NtId {
    grammar
        .nt(nonterminal.as_str())
        .unwrap_or_else(|| panic!("missing NT {}", nonterminal.as_str()))
}

/// Documentary production outline (kept from sketch).
pub fn production_outline(nonterminal_name: &str) -> &'static str {
    match nonterminal_name {
        "Pause" => "PAUSE ::= '.'  /* CLL §3.3, §4.9 */",
        "CmavoSimple" => {
            "CmavoSimple ::= V | C V | VV | C VV | C Y | Y | Y'Y | Exp…  /* §4.2; boundary pauses checked separately */"
        }
        "Gismu" => "Gismu ::= C V MedialCc V | InitialCc V C V  /* §4.4 */",
        "Lujvo" => "Lujvo ::= RafsiNonFinal+ RafsiFinal  /* §4.5–4.6; hyphen legality → constraints */",
        "Fuhivla" => "Fuhivla ::= coarse brivla-like, no Y; initial cluster §4.7.1; exclusions → C16–C18",
        "Cmevla" => "Cmevla ::= … C+  /* §4.8; boundary pause and la/lai/doi ban → constraints */",
        _ => "see README §4 CFG nonterminals outline",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cll::cfg::earley::recognizes;

    #[test]
    fn grammar_builds_and_has_starts() {
        let grammar = morph_grammar();
        assert!(grammar.nt("Gismu").is_some());
        assert!(grammar.nt("Word").is_some());
        assert!(grammar.productions.len() > 100);
    }

    #[test]
    fn gismu_shapes() {
        let grammar = morph_grammar();
        let start_nonterminal = shape_nt_id(grammar, ShapeNt::Gismu);
        assert!(recognizes(grammar, start_nonterminal, b"creka"));
        assert!(recognizes(grammar, start_nonterminal, b"blanu"));
        assert!(recognizes(grammar, start_nonterminal, b"patfu"));
        assert!(!recognizes(grammar, start_nonterminal, b"ba"));
    }

    #[test]
    fn cmavo_simple_shapes() {
        let grammar = morph_grammar();
        let start_nonterminal = shape_nt_id(grammar, ShapeNt::CmavoSimple);
        assert!(recognizes(grammar, start_nonterminal, b"a"));
        assert!(recognizes(grammar, start_nonterminal, b"ba"));
        assert!(recognizes(grammar, start_nonterminal, b"au"));
        assert!(recognizes(grammar, start_nonterminal, b"pei"));
        assert!(recognizes(grammar, start_nonterminal, b"ki'a"));
        assert!(recognizes(grammar, start_nonterminal, b"y"));
        assert!(recognizes(grammar, start_nonterminal, b"cy"));
        // CLL §17.2 apostrophe letteral
        assert!(recognizes(grammar, start_nonterminal, b"y'y"));
        assert!(!recognizes(grammar, start_nonterminal, b".a"));
    }

    #[test]
    fn yhy_letteral_compound() {
        let grammar = morph_grammar();
        let start_nonterminal = shape_nt_id(grammar, ShapeNt::CmavoCompound);
        assert!(recognizes(grammar, start_nonterminal, b"y'y.bu"));
        assert!(recognizes(grammar, start_nonterminal, b"abu"));
        assert!(recognizes(grammar, start_nonterminal, b"ki'e.u'e"));
        assert!(!recognizes(grammar, start_nonterminal, b"au"));
        assert!(!recognizes(grammar, start_nonterminal, b"paui"));
    }
}
