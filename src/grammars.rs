//! Publicly accessible grammar constants.
//!
//! For long-running apps, build the parser once and reuse: `let peg = Peg::new(LOGLAN_GRAMMAR.0, LOGLAN_GRAMMAR.1).unwrap();`

/// The Loglan grammar definition.
/// Format: (start_rule, grammar_string)
pub const LOGLAN_GRAMMAR: (&str, &str) = (
    "utterance",
    r#"sp <- ([ ]+/([~`] &Co1))

sp2 <- [ ]+

spnopause <- ([~`] &Co1)

Vo1 <- [aeiouyAEIOUY]

Vo2 <- [aeiouAEIOU]

Co1 <- [bcdfghjklmnprstvzBCDFGHJKLMNPRSTVZ]

Cvoiced <- [bdgjvzBDGJVZ]

Cunvoiced <- [ptkcfsPTKCFS]

Badvoice <- ((Cvoiced (Cunvoiced/[Hh]))/(Cunvoiced (Cvoiced/[Hh])))

Letter <- (![qwxQWX] [a-zA-Z])

Stress2 <- [\'*]

Juncture <- ((([-] &Letter)/Stress2) !Juncture)

Stress <- ([\'*] !Juncture)

Lowercase <- (![qwx] [a-z])

Uppercase <- (![QWX] [A-Z])

caprule <- ([\"(]? &(([z] Vo1 (!Uppercase/&TAI0))/(Lowercase TAI0 (!Uppercase/&TAI0))/(!(Lowercase Uppercase) .)) Letter (&(([z] Vo1 (!Uppercase/&TAI0))/(Lowercase TAI0 (!Uppercase/&TAI0))/(!(Lowercase Uppercase) .)) (Letter/Juncture))* !(Letter/Juncture))

Terminal <- [.:?!;#]

Character <- (Letter/Juncture)

Comma0 <- ([,]? sp2)

AlienText <- ((Comma0 [\"] (![\"] .)+ [\"])/(Comma0 (!Comma0 !Terminal .)+ (Comma0 [Yy] Comma0 (!Comma0 !Terminal .)+)*))

HOIalien <- ([Hh] [Oo] [Ii])

HUEalien <- ([Hh] [Uu] Juncture? [Ee])

LIEalien <- ([Ll] [Ii] Juncture? [Ee])

LAOalien <- ([Ll] [Aa] [Oo])

LIOalien <- ([Ll] [Ii] Juncture? [Oo])

SAOalien <- ([Ss] [Aa] [Oo])

SUEalien <- ([Ss] [Uu] Juncture? [Ee])

AlienWord <- (&caprule ((HOIalien Juncture? &(Comma0 [\"]))/(HUEalien Juncture? &(Comma0 [\"]))/(LIEalien Juncture?)/(LAOalien Juncture?)/(LIOalien Juncture?)/(SAOalien Juncture?)/(SUEalien Juncture?)) AlienText)

Alienmarker <- ((([Hh] [Oo] [Ii] Juncture? &[\"])/([Hh] [Uu] Juncture? [Ee] Juncture? &(Comma0 [\"]))/([Ll] [Ii] Juncture? [Ee] Juncture?)/([Ll] [Aa] [Oo] Juncture?)/([Ll] [Ii] Juncture? [Oo] Juncture?)/([Ss] [Aa] [Oo] Juncture?)/([Ss] [Uu] Juncture? [Ee] Juncture?)) !Vo1)

Continuant <- [mnlrMNLR]

Syllabic <- (([mM] [mM] !(Juncture? [mM]))/([nN] [nN] !(Juncture? [nN]))/([rR] [rR] !(Juncture? [rR]))/([lL] [lL] !(Juncture? [lL])))

MustMono <- (([aeoAEO] [iI] ![iI])/([aA] [oO]))

BrokenMono <- (([aeoAEO] Juncture [iI] ![iI])/([aA] Juncture [oO]))

Mono <- (MustMono/([iI] !([uU] [uU]) Vo2)/([uU] !([iI] [iI]) Vo2))

Vv <- (!(!MustMono Vo2 Juncture? Vo2 Juncture? [Rr] [Rr]) (!BrokenMono Vo2 Juncture? Vo2))

NextVowels <- (MustMono/(Vo2 &MustMono)/Mono/(!([Ii] Juncture [Ii] Vo1) !([Uu] Juncture [Uu] Vo1) Vo2))

DoubleVowel <- (([aA] Juncture? [aA])/([eE] Juncture? [eE])/([oO] Juncture? [oO])/([iI] Juncture [iI])/([uU] Juncture [uU])/([iI] [Ii] &[iI])/([Uu] [uU] &[uU]))

Vocalic <- (NextVowels/Syllabic/[Yy])

Initial <- (([Bb] [Ll])/([Bb] [Rr])/([Cc] [Kk])/([Cc] [Ll])/([Cc] [Mm])/([Cc] [Nn])/([Cc] [Pp])/([Cc] [Rr])/([Cc] [Tt])/([Dd] [Jj])/([Dd] [Rr])/([Dd] [Zz])/([Ff] [Ll])/([Ff] [Rr])/([Gg] [Ll])/([Gg] [Rr])/([Jj] [Mm])/([Kk] [Ll])/([Kk] [Rr])/([Mm] [Rr])/([Pp] [Ll])/([Pp] [Rr])/([Ss] [Kk])/([Ss] [Ll])/([Ss] [Mm])/([Ss] [Nn])/([Ss] [Pp])/([Ss] [Rr])/([Ss] [Tt])/([Ss] [Vv])/([Tt] [Cc])/([Tt] [Rr])/([Tt] [Ss])/([Vv] [Ll])/([Vv] [Rr])/([Zz] [Bb])/([Zz] [Ll])/([Zz] [Vv]))

MaybeInitial <- (([Bb] Juncture? [Ll])/([Bb] Juncture? [Rr])/([Cc] Juncture? [Kk])/([Cc] Juncture? [Ll])/([Cc] Juncture? [Mm])/([Cc] Juncture? [Nn])/([Cc] Juncture? [Pp])/([Cc] Juncture? [Rr])/([Cc] Juncture? [Tt])/([Dd] Juncture? [Jj])/([Dd] Juncture? [Rr])/([Dd] Juncture? [Zz])/([Ff] Juncture? [Ll])/([Ff] Juncture? [Rr])/([Gg] Juncture? [Ll])/([Gg] Juncture? [Rr])/([Jj] Juncture? [Mm])/([Kk] Juncture? [Ll])/([Kk] Juncture? [Rr])/([Mm] Juncture? [Rr])/([Pp] Juncture? [Ll])/([Pp] Juncture? [Rr])/([Ss] Juncture? [Kk])/([Ss] Juncture? [Ll])/([Ss] Juncture? [Mm])/([Ss] Juncture? [Nn])/([Ss] Juncture? [Pp])/([Ss] Juncture? [Rr])/([Ss] Juncture? [Tt])/([Ss] Juncture? [Vv])/([Tt] Juncture? [Cc])/([Tt] Juncture? [Rr])/([Tt] Juncture? [Ss])/([Vv] Juncture? [Ll])/([Vv] Juncture? [Rr])/([Zz] Juncture? [Bb])/([Zz] Juncture? [Ll])/([Zz] Juncture? [Vv]))

InitialConsonants <- ((!Syllabic Co1 &Vocalic)/(!(Co1 Syllabic) Initial &Vocalic)/(&Initial Co1 !(Co1 Syllabic) Initial !Syllabic &Vocalic))

NoMedial2 <- (([Bb] Juncture? [Bb])/([Cc] Juncture? [Cc])/([Dd] Juncture? [Dd])/([Ff] Juncture? [Ff])/([Gg] Juncture? [Gg])/([Hh] Juncture? Co1)/([Jj] Juncture? [Jj])/([Kk] Juncture? [Kk])/([Ll] Juncture? [Ll])/([Mm] Juncture? [Mm])/([Nn] Juncture? [Nn])/([Pp] Juncture? [Pp])/([Rr] Juncture? [Rr])/([Ss] Juncture? [Ss])/([Tt] Juncture? [Tt])/([Vv] Juncture? [Vv])/([Zz] Juncture? [Zz])/([CJSZcjsz] Juncture? [CJSZcjsz])/([Ff] Juncture? [Vv])/([Kk] Juncture? [Gg])/([Pp] Juncture? [Bb])/([Tt] Juncture? [Dd])/([FKPTfkpt] Juncture? [JZjz])/([Bb] Juncture? [Jj])/([Ss] Juncture? [Bb]))

NoMedial3 <- (([Cc] Juncture? [Dd] Juncture? [Zz])/([Cc] Juncture? [Vv] Juncture? [Ll])/([Nn] Juncture? [Dd] Juncture? [Jj])/([Nn] Juncture? [Dd] Juncture? [Zz])/([Dd] Juncture? [Cc] Juncture? [Mm])/([Dd] Juncture? [Cc] Juncture? [Tt])/([Dd] Juncture? [Tt] Juncture? [Ss])/([Pp] Juncture? [Dd] Juncture? [Zz])/([Gg] Juncture? [Tt] Juncture? [Ss])/([Gg] Juncture? [Zz] Juncture? [Bb])/([Ss] Juncture? [Vv] Juncture? [Ll])/([Jj] Juncture? [Dd] Juncture? [Jj])/([Jj] Juncture? [Tt] Juncture? [Cc])/([Jj] Juncture? [Tt] Juncture? [Ss])/([Jj] Juncture? [Vv] Juncture? [Rr])/([Tt] Juncture? [Vv] Juncture? [Ll])/([Kk] Juncture? [Dd] Juncture? [Zz])/([Vv] Juncture? [Tt] Juncture? [Ss])/([Mm] Juncture? [Zz] Juncture? [Bb]))

SyllableA <- (Co1 Vo2 FinalConsonant (!Syllable FinalConsonant)?)

SyllableB <- (InitialConsonants? Vocalic (!Syllable FinalConsonant)? (!Syllable FinalConsonant)?)

Syllable <- ((SyllableA/SyllableB) Juncture?)

FinalConsonant <- (!Syllabic !(&Badvoice Co1 !Syllable) (!(!Continuant Co1 !Syllable Continuant) !NoMedial2 !NoMedial3 Co1 !(Juncture? (Vo2/Syllabic))))

SyllableD <- (&(InitialConsonants? ([Yy]/DoubleVowel/BrokenMono/(&Mono Vo2 DoubleVowel)/(!MustMono &Mono Vo2 BrokenMono))) Syllable)

BorrowingSyllable <- (!Syllabic !SyllableD Syllable)

VowelFinal <- (InitialConsonants? Vocalic Juncture? !Vo2)

SyllableC <- (&(InitialConsonants? Syllabic) Syllable)

SyllableY <- (&(InitialConsonants? [Yy]) Syllable)

StressedSyllable <- ((SyllableA/SyllableB) Stress2)

NameEndSyllable <- (InitialConsonants? (Syllabic/(Vocalic &FinalConsonant)) FinalConsonant? FinalConsonant? Stress? !spnopause !Letter)

Explicitpause <- ((Co1 Stress2? sp2 &Letter)/(Letter Stress2? sp2 &Vo1)/(Letter Stress2? [,] sp2 &Letter))

MaybePauseSyllable <- (InitialConsonants? Vocalic Stress2? &(sp2 &Co1))

PRENAME <- ((Syllable &Syllable)* NameEndSyllable)

LAname <- ([Ll] [Aa])

HOIname <- ([Hh] [Oo] [Ii])

CIname <- ([Cc] [Ii])

LIUname <- ([Ll] [Ii] Juncture? [Uu])

MUEname <- ([Mm] [Uu] Juncture? [Ee])

GAOname <- ([Gg] [Aa] [Oo])

HUEname <- ([Hh] [Uu] Juncture? [Ee])

LAname2 <- ([Ll] !Explicitpause [Aa])

HOIname2 <- ([Hh] [Oo] !Explicitpause [Ii])

LIUname2 <- ([Ll] [Ii] Juncture? !Explicitpause [Uu])

MUEname2 <- ([Mm] [Uu] Juncture? !Explicitpause [Ee])

GAOname2 <- ([Gg] [Aa] !Explicitpause [Oo])

HUEname2 <- ([Hh] [Uu] Juncture? !Explicitpause [Ee])

MarkedName <- (&caprule ((LAname2 Juncture?)/(HOIname2 Juncture?)/(HUEname2 Juncture?)/(LIUname2 Juncture?)/(GAOname2 Juncture?)/(MUEname2 Juncture?)) sp? &Co1 &caprule PRENAME)

FalseMarked <- (&PRENAME (!MarkedName Character)* MarkedName)

Comma1 <- ([,] sp2)

NAMEWORD <- (((&caprule MarkedName)/(Comma1 !FalseMarked &caprule PRENAME)/(&Vo1 !FalseMarked &caprule PRENAME)/(&caprule (((LAname Juncture?)/(HOIname Juncture?)/(HUEname Juncture?)/(CIname Juncture? &Comma0)/(LIUname Juncture?)/(MUEname Juncture?)/(GAOname Juncture?)) !Vo1 Comma0? &caprule PRENAME))) ((Comma0 !FalseMarked &caprule PRENAME)/(Comma0 &([Cc] [Ii]) NAMEWORD))* &((sp2? [Cc] [Ii] predunit)/(&(Comma1/Terminal/[\")]/!.) .)/!.))

Namemarker <- ((([Ll] [Aa] Juncture?)/([Hh] [Oo] [Ii] Juncture?)/([Hh] [Uu] Juncture? [Ee] Juncture?)/([Cc] &(Explicitpause/([Ii] Juncture? sp2 PRENAME)) [Ii] Juncture?)/([Ll] [Ii] Juncture? [Uu] Juncture?)/([Gg] [Aa] [Oo] Juncture?)/([Mm] [Uu] Juncture? [Ee] Juncture?)) !Vo1)

Badnamemarker <- (Namemarker !Vo1 Comma0? (Syllable &Syllable)* MaybePauseSyllable)

Vthree <- ((Vo2 Juncture?) (Vo2 Juncture?) (Vo2 Juncture?))

Vfour <- ((Vo2 Juncture?) (Vo2 Juncture?) (Vo2 Juncture?) (Vo2 Juncture?))

Predstart1 <- (((&MaybeInitial Co1 Juncture? MaybeInitial)/MaybeInitial) &Vo2 !(Vo2 Stress !Mono Vo2) !(Vo2 Juncture? Vo2 !Character) !(Vo2 Juncture? !Character))

Predstart2 <- (Co1 Vo2 Juncture? (Vo2 Juncture?)? !Predstart1 Co1 Juncture? Co1)

Predstart3 <- (Co1 !Vthree (!StressedSyllable Vo2 Juncture?)? &StressedSyllable Vo2 Vo2? Juncture? Co1 Juncture? Co1)

Predstart4 <- (Co1? Vo2 Juncture? (Vo2 Juncture?)? (Vo2 Juncture?)? !Predstart1 !(MaybeInitial Vo2) Co1 Juncture? Co1)

Predstart5 <- (Co1? !Vfour (!StressedSyllable Vo2 Juncture?)? (!StressedSyllable Vo2 Juncture?)? &StressedSyllable Vo2 Vo2? Juncture? !(MaybeInitial Vo2) Co1 Juncture? Co1)

Predstart6 <- (Co1 (Vo2 Juncture?) ((Vo2 Juncture? [Hh]?)/(Co1 Juncture? (Co1 Juncture?)?)) [Yy])

Predstart <- (Predstart1/Predstart2/Predstart3/Predstart4/Predstart5/Predstart6)

a <- ([Aa] !Badstress Juncture? !Vo1)

e <- ([Ee] !Badstress Juncture? !Vo1)

i <- ([Ii] !Badstress Juncture? !Vo1)

o <- ([Oo] !Badstress Juncture? !Vo1)

u <- ([Uu] !Badstress Juncture? !Vo1)

Hearly <- (!Predstart [Hh])

Nearly <- (!Predstart [Nn])

Connective <- (sp? !Predstart ([Nn] [Oo] Juncture? !i)? (a/e/i/o/u/(Hearly a)/(Nearly uu)) Juncture? !Vo2 !(!Predstart [Ff] [Ii]) !(!Predstart [Mm] [Aa]) !(!Predstart [Zz] [Ii]))

CmapuaUnit <- ((Co1 Mono Juncture? Vo2 !(Stress2 sp? &Co1 Predstart) Juncture? !Vo1)/(Co1 (Vv/([Ii] [Yy])/([Uu] [Yy])) !(Stress2 sp? &Co1 Predstart) Juncture? !Vo1)/(Co1 Vo2 !(Stress2 sp? &Co1 Predstart) Juncture? !Vo1))

Likie <- (([Ll] [Ii] Juncture? !Vo1)/([Ki] [Ii] Juncture? [Ee] Juncture? !Vo1))

Cmapua <- (&caprule !Badnamemarker ((!Predstart (Vv/([Ii] [Yy])/([Uu] [Yy])) !(Stress2 sp? &Co1 Predstart) Juncture? NOI0)/(!Predstart [Nn] [Oo] Juncture? !Predstart (Vv/([Ii] [Yy])/([Uu] [Yy])) !(Stress2 sp? &Co1 Predstart) Juncture?)/((!Predstart (Vv/([Ii] [Yy])/([Uu] [Yy])) !(Stress2 sp? &Co1 Predstart) Juncture?)+/(((!Predstart Vo1 !(Stress2 sp? &Co1 Predstart) Juncture?)/(!Predstart CmapuaUnit)) (!Namemarker !Alienmarker !Likie !Predstart CmapuaUnit)*))/(!Predstart Vo2 !(Stress2 sp? &Co1 Predstart) Juncture?)) !Vo1 !(Co1+ Juncture) !(sp? Connective))

wy <- [Yy]

ar <- [Rr]

en <- [Nn]

hh <- [Hh]

Dash <- [-]

Cvv <- (Co1 Vv ((Juncture? hh? wy Dash? &Complex)/(Juncture? ar ar? Juncture? &Co1)/(en Juncture? &ar)/(Juncture? !Vo2)))

CvvNoHyphen <- (Co1 Vv Juncture? !Vo2)

CvvHiddenStress <- (Co1 &DoubleVowel Vo1 Dash? Vo1 ((Dash? hh? wy Dash? &Complex)/(ar Dash? &Co1)/(en Dash? &ar)/(Dash? !Vo2)))

CvvFinalStress <- (Co1 Vv ((Stress2 hh? wy Dash? &Complex)/(ar Stress2 &Co1)/(Stress2 ar ar Juncture? &Co1)/(en Stress2 &ar)/(Stress2 !Vo2)))

CvvNoY <- (Co1 Vv ((Juncture? ar ar? Juncture? &Co1)/(en Juncture? &ar)/(Juncture? !Vo2)))

CvvNoYFinalStress <- (Co1 Vv ((ar Stress2 &Co1)/(Stress2 ar ar Juncture? &Co1)/(en Stress2 &ar)/(Stress2 !Vo2)))

CvvNoYMedialStress <- (Co1 !BrokenMono Vo2 Stress2 Vo2 Dash? !Vo2)

Ccv <- (Initial Vo2 ((Juncture? wy Dash? &Letter)/(Juncture? !Vo2)))

CcvStressed <- (Initial Vo2 ((Stress2 wy Dash? &Letter)/(Stress2 !Vo2)))

CcvNoY <- (Initial Vo2 Juncture? !Vo2)

CcvBad <- (MaybeInitial Vo2 Juncture? !Vo2)

CCVBadStressed <- (MaybeInitial Vo2 Stress2 !Vo2)

Cvc <- ((Co1 Vo2 !NoMedial2 !NoMedial3 Co1 ((Juncture? wy Dash? &Letter)/(Juncture? &Co1)))/(Co1 Vo2 Juncture Co1 wy Dash? &Letter))

CvcStressed <- ((Co1 Vo2 !NoMedial2 !NoMedial3 Co1 ((Stress2 wy Dash? &Letter)/(Stress2 &Letter)))/(Co1 Vo2 Stress2 Co1 wy Dash? &Letter))

CvcNoY <- (Co1 Vo2 !NoMedial2 !NoMedial3 Co1 Juncture? &Co1)

CvcBad <- (Co1 Vo2 !NoMedial2 !NoMedial3 Juncture? Co1 &Co1)

CvcNoYStressed <- (Co1 Vo2 !NoMedial2 !NoMedial3 Co1 Stress2 &Co1)

CvcBadStressed <- (Co1 Vo2 !NoMedial2 !NoMedial3 Stress2 Co1 &Co1)

CcvCv <- (Initial Vo2 Juncture? Co1 Vo2 Dash? !Vo2)

CcvCvStressed <- (Initial Vo2 Stress2 Co1 Vo2 Dash? !Vo2)

CcvCvBad <- (MaybeInitial Vo2 Juncture? Co1 Vo2 Dash? !Vo2)

CcvCvBadStressed <- (MaybeInitial Vo2 Stress2 Co1 Vo2 Dash? !Vo2)

CvcCv <- ((Co1 Vo2 Juncture? Initial Vo2 Dash? !Vo2)/(Co1 Vo2 !NoMedial2 Co1 Juncture? Co1 Vo2 Dash? !Vo2))

CvcCvStressed <- ((Co1 Vo2 Stress2 Initial Vo2 Dash? !Vo2)/(Co1 Vo2 !NoMedial2 Co1 Stress2 Co1 Vo2 Dash? !Vo2))

CcvCy <- (Initial Vo2 Juncture? Co1 wy Dash?)

CvcCy <- ((Co1 Vo2 Juncture? Initial wy Dash?)/(Co1 Vo2 !NoMedial2 Co1 Juncture? Co1 wy Dash?))

CcvCyStressed <- (Initial Vo2 Stress2 Co1 wy Dash?)

CvcCyStressed <- ((Co1 Vo2 Stress2 Initial wy Dash?)/(Co1 Vo2 !NoMedial2 Co1 Stress2 Co1 wy Dash?))

BorrowingTail1 <- (!SyllableC &StressedSyllable BorrowingSyllable (!StressedSyllable &SyllableC BorrowingSyllable)? !StressedSyllable &BorrowingSyllable VowelFinal)

BorrowingTail2 <- (!SyllableC BorrowingSyllable (!StressedSyllable &SyllableC BorrowingSyllable)? !StressedSyllable &BorrowingSyllable VowelFinal (&wy/!Character))

BorrowingTail3 <- (!SyllableC !StressedSyllable BorrowingSyllable (!StressedSyllable &SyllableC BorrowingSyllable)? &BorrowingSyllable InitialConsonants? Vocalic Stress2 &wy)

BorrowingTail <- (BorrowingTail1/BorrowingTail2)

Ccvv <- ((InitialConsonants Vo2 Juncture? Vo2 Juncture? !Character)/(InitialConsonants Vo2 Stress2 !Mono Vo2 Juncture?))

PreBorrowing <- (&Predstart !Ccvv !Cmapua !SyllableC (!BorrowingTail !StressedSyllable !(SyllableC SyllableC) BorrowingSyllable)* BorrowingTail)

StressedPreBorrowing <- (&Predstart !Ccvv !Cmapua !SyllableC (!BorrowingTail !StressedSyllable !(SyllableC SyllableC) BorrowingSyllable)* BorrowingTail1)

PreBorrowing2 <- (&Predstart !Ccvv !Cmapua !SyllableC (!BorrowingTail !StressedSyllable !(SyllableC SyllableC) BorrowingSyllable)* BorrowingTail2)

PreBorrowing3 <- (&Predstart !Ccvv !Cmapua !SyllableC (!BorrowingTail3 !StressedSyllable !(SyllableC SyllableC) BorrowingSyllable)* BorrowingTail3)

RfinalDjifoa <- ((CcvCvBad/CvcCv/CvvNoHyphen/CcvBad) (&wy/!Character))

RmediallyStressed <- (CcvCvBadStressed/CvcCvStressed/CvvNoYMedialStress)

RfinallyStressed <- (CvvNoYFinalStress/CCVBadStressed/CvcBadStressed/CvcNoYStressed)

BorrowingComplexTail <- (RmediallyStressed/(RfinallyStressed ((&(Co1 Mono) CvvNoHyphen)/CcvBad))/RfinalDjifoa)

ResolvedBorrowing <- ((!BorrowingComplexTail (CvvNoY/CcvBad/CvcBad))* BorrowingComplexTail)

Borrowing <- (!ResolvedBorrowing &caprule PreBorrowing !(sp? Connective))

StressedBorrowing <- (!ResolvedBorrowing &caprule StressedPreBorrowing !(sp? &Vo1 Cmapua))

BorrowingDjifoa <- (!ResolvedBorrowing &caprule PreBorrowing2 ((Stress2 wy Comma1)/(Juncture? wy Dash?)))

StressedBorrowingDjifoa <- (!ResolvedBorrowing &caprule PreBorrowing3 wy Dash? Comma1?)

DefaultStressedSyllable <- Syllable

PhoneticComplexTail1 <- (!SyllableC !SyllableY &StressedSyllable DefaultStressedSyllable (!StressedSyllable &(SyllableC/SyllableY) Syllable)? !StressedSyllable !SyllableY VowelFinal !Vo1)

PhoneticComplexTail2 <- (!SyllableC !SyllableY DefaultStressedSyllable (!StressedSyllable &(SyllableC/SyllableY) Syllable)? !StressedSyllable !SyllableY VowelFinal !Character)

PhoneticComplexTail <- (PhoneticComplexTail1/PhoneticComplexTail2)

PhoneticComplex <- (&Predstart !Ccvv !Cmapua !SyllableC ((StressedBorrowingDjifoa &PhoneticComplex)/(!PhoneticComplexTail !StressedSyllable !(SyllableC SyllableC) Syllable))* PhoneticComplexTail)

FinalDjifoa <- ((Borrowing/CcvCv/CvcCv/CvvNoHyphen/CcvNoY) !Character)

MediallyStressed <- (StressedBorrowing/CcvCvStressed/CvcCvStressed/CvvNoYMedialStress)

FinallyStressed <- (StressedBorrowingDjifoa/CcvCyStressed/CvcCyStressed/CvvFinalStress/CcvStressed/CvcStressed)

ComplexTail <- ((CvvHiddenStress ((&(Co1 Mono) CvvNoHyphen)/CcvNoY) !Character)/(FinallyStressed ((&(Co1 Mono) CvvNoHyphen)/CcvNoY))/MediallyStressed/FinalDjifoa)

PreComplex <- ((!CvvHiddenStress !ComplexTail ((StressedBorrowingDjifoa &PhoneticComplex)/BorrowingDjifoa/CvcCy/CcvCy/Cvv/Ccv/Cvc))* ComplexTail)

Complex <- (&caprule &PreComplex PhoneticComplex !(sp? Connective))

LiQuote <- ((&caprule [Ll] [Ii] Juncture? Comma2? [\"] PhoneticUtterance [\"] Comma2? &caprule [Ll] [Uu] Juncture? !(sp? Connective))/(&caprule [Kk] [Ii] Juncture? [Ee] Juncture? Comma2? [(] PhoneticUtterance [)] Comma2? &caprule [Kk] [Ii] Juncture? [Uu] Juncture? !(sp? Connective)))

Word <- (NAMEWORD/Cmapua/Complex/CcvNoY)

SingleWord <- (((Borrowing !.)/(Complex !.)/(Word !.)/(PRENAME !.)/CcvNoY) !.)

PhoneticUtterance1 <- (NAMEWORD/(sp? LiQuote)/(sp? NAMEWORD)/(sp? AlienWord)/(sp? Cmapua)/(sp? '--')/(sp? '...')/(sp? Borrowing !wy)/(sp? Complex)/(sp? CcvNoY))+

PhoneticUtterance <- (PhoneticUtterance1/Comma1/Terminal)+

Badstress <- (Stress2 sp? &Co1 Predstart)

b <- (!Predstart [Bb])

c <- (!Predstart [Cc])

d <- (!Predstart [Dd])

f <- (!Predstart [Ff])

g <- (!Predstart [Gg])

h <- (!Predstart [Hh])

j <- (!Predstart [Jj])

k <- (!Predstart [Kk])

l <- (!Predstart [Ll])

m <- (!Predstart [Mm])

n <- (!Predstart [Nn])

p <- (!Predstart [Pp])

r <- (!Predstart [Rr])

s <- (!Predstart [Ss])

t <- (!Predstart [Tt])

v <- (!Predstart [Vv])

z <- (!Predstart [Zz])

Vo3 <- (Juncture? Vo2 !Badstress)

aa <- ([Aa] Juncture? [Aa] !Badstress Juncture? !Vo1)

ae <- ([Aa] Juncture? [Ee] !Badstress Juncture? !Vo1)

ai <- ([Aa] [Ii] !Badstress Juncture? !Vo1)

ao <- ([Aa] [Oo] !Badstress Juncture? !Vo1)

ai2 <- ([Aa] [Ii] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

ao2 <- ([Aa] [Oo] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

au <- ([Aa] Juncture? [Uu] !Badstress Juncture? !Vo1)

ea <- ([Ee] Juncture? [Aa] !Badstress Juncture? !Vo1)

ee <- ([Ee] Juncture? [Ee] !Badstress Juncture? !Vo1)

ei <- ([Ee] [Ii] !Badstress Juncture? !Vo1)

ei2 <- ([Ee] [Ii] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

eo <- ([Ee] Juncture? [Oo] !Badstress Juncture? !Vo1)

eu <- ([Ee] Juncture? [Uu] !Badstress Juncture? !Vo1)

ia <- ([Ii] Juncture? [Aa] !Badstress Juncture? !Vo1)

ie <- ([Ii] Juncture? [Ee] !Badstress Juncture? !Vo1)

ii <- ([Ii] Juncture? [Ii] !Badstress Juncture? !Vo1)

io <- ([Ii] Juncture? [Oo] !Badstress Juncture? !Vo1)

iu <- ([Ii] Juncture? [Uu] !Badstress Juncture? !Vo1)

ia2 <- ([Ii] Juncture? [Aa] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

ie2 <- ([Ii] Juncture? [Ee] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

ii2 <- ([Ii] Juncture? [Ii] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

io2 <- ([Ii] Juncture? [Oo] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

iu2 <- ([Ii] Juncture? [Uu] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

oa <- ([Oo] Juncture? [Aa] !Badstress Juncture? !Vo1)

oe <- ([Oo] Juncture? [Ee] !Badstress Juncture? !Vo1)

oi <- ([Oo] [Ii] !Badstress Juncture? !Vo1)

oi2 <- ([Oo] [Ii] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

oo <- ([Oo] Juncture? [Oo] !Badstress Juncture? !Vo1)

ou <- ([Oo] Juncture? [Uu] !Badstress Juncture? !Vo1)

ua <- ([Uu] Juncture? [Aa] !Badstress Juncture? !Vo1)

ue <- ([Uu] Juncture? [Ee] !Badstress Juncture? !Vo1)

ui <- ([Uu] Juncture? [Ii] !Badstress Juncture? !Vo1)

uo <- ([Uu] Juncture? [Oo] !Badstress Juncture? !Vo1)

uu <- ([Uu] Juncture? [Uu] !Badstress Juncture? !Vo1)

ua2 <- ([Uu] Juncture? [Aa] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

ue2 <- ([Uu] Juncture? [Ee] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

ui2 <- ([Uu] Juncture? [Ii] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

uo2 <- ([Uu] Juncture? [Oo] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

uu2 <- ([Uu] Juncture? [Uu] !Badstress Juncture? &(Vo2 Juncture? !Vo1))

iy <- ([Ii] [Yy] !Badstress Juncture? !Vo1)

uy <- ([Uu] [Yy] !Badstress Juncture? !Vo1)

OptPause <- (Comma1 !(Vo1/Connective) &caprule)

Comma <- ([,] sp2 &caprule)

Comma2 <- ([,]? sp2 &caprule)

End <- ((sp2? '#' sp2 utterance)/(sp2 !.)/!.)

Period <- (([!.:;?] (&End/(sp2 &caprule))) (invvoc Period?)?)

TAI0 <- ((Vo1 Juncture? m a)/(Vo1 Juncture? f i)/(Vo1 Juncture? z i)/(!Predstart Co1 ai)/(!Predstart Co1 ei)/(!Predstart Co1 ai2 u)/(!Predstart Co1 ei2 u)/(!Predstart Co1 eo)/(z [Ii] Vo1 !Badstress Juncture? !Vo1 (m a)?))

NOI0 <- (n oi)

A0 <- (&Cmapua (a/e/o/u/(h a)/(n uu)))

A <- (sp? !Predstart !TAI0 (n [o])? A0 NOI0? !(sp PAWORD0 OptPause) !(PAWORD0 !OptPause [ ,]) (PAWORD0 ((f i)/&OptPause))?)

ANOFI <- (sp? (!Predstart !TAI0 ((n [o])? A0 NOI0? PAWORD0?)))

AONE <- A

ACI <- (ANOFI c i)

AGE <- (ANOFI g e)

CA0 <- (((n o)? ((c a)/(c e)/(c o)/(c u)/(z e)/(c i h a)/(n u c u))) NOI0?)

CA1 <- (CA0 !(sp PAWORD0 OptPause) !(PAWORD0 !OptPause [ ,]) (PAWORD0 ((f i)/&OptPause))?)

CANOFI1 <- (CA0 PAWORD0?)

CA <- (sp? CA1)

ZE <- (sp? (z e))

I <- (sp? !Predstart !TAI0 i !(sp PAWORD0 OptPause) !(PAWORD0 !OptPause [ ,]) (PAWORD0 ((f i)/&OptPause))?)

ICA <- (sp? i ((h a)/CA1))

ICI <- (sp? i CANOFI1? c i)

IGE <- (sp? i CANOFI1? g e)

KA0 <- ((k a)/(k e)/(k o)/(k u)/(k i h a)/(n u k u))

KOU0 <- ((k ou)/(m oi)/(r au)/(s oa)/(m ou)/(c iu))

KOU1 <- (((n u n o)/(n u)/(n o)) KOU0)

KA <- (sp? (KA0/((KOU1/KOU0) k i)) NOI0?)

KI <- (sp? (k i) NOI0?)

KOU2 <- (KOU1 !KI)

BadNIStress <- ((Co1 Vo2 Vo2? Stress (m a)? (m oa)? NI RA0)/(Co1 Vo2 Stress Vo2 (m a)? (m oa)? NI RA0))

NI0 <- (!BadNIStress ((k ua)/(g ie)/(g iu)/(h ie)/(h iu)/(n ea)/(n io)/(p ea)/(p io)/(s uu)/(s ua)/(t ia)/(z oa)/(z oo)/(h o)/(n i)/(n e)/(t o)/(t e)/(f o)/(f e)/(v o)/(v e)/(p i)/(r e)/(r u)/(s e)/(s o)/(h i)))

SA0 <- (!BadNIStress ((s a)/(s i)/(s u)/(ie (Comma2? !ie SA0)?)) NOI0?)

RA0 <- (!BadNIStress ((r a)/(r i)/(r e)/(r u)/(r o)/(b ao)))

NI1 <- ((NI0 (!BadNIStress m a)? (!BadNIStress m oa NI0*)?) (Comma2 !(NI RA0) &NI)?)

RA1 <- ((RA0 (!BadNIStress m a)? (!BadNIStress m oa NI0*)?) (Comma2 !(NI RA0) &NI)?)

NI2 <- (((SA0? (NI1+/RA1))/SA0) NOI0? (CA0 ((SA0? (NI1+/RA1))/SA0) NOI0?)*)

NI <- (sp? (p i)? NI2 ((&(m ue) ACRONYM (Comma/&End/&Period) !(c u))/(Comma2? m ue Comma2? PRENAME !(c u)))? (c u)?)

mex <- (sp? NI)

CI <- (sp? (c i))

ACRONYM <- (sp? &caprule ((m ue)/TAI0/(z Vo2 !Vo2)) ((Comma &ACRONYM m ue)/NI1/TAI0/(z Vo2 (!Vo2/(z &Vo2))))+)

TAI <- (sp? (TAI0/((g ao) !Vo2 sp? (PRENAME/Predicate/CmapuaUnit))))

DA0 <- ((t ao)/(t io)/(t ua)/(m io)/(m iu)/(m uo)/(m uu)/(t oa)/(t oi)/(t oo)/(t ou)/(t uo)/(t uu)/(s uo)/(h u)/(b a)/(b e)/(b o)/(b u)/(d a)/(d e)/(d i)/(d o)/(d u)/(m i)/(t u)/(m u)/(t i)/(t a)/(m o)/(k oo)/(d ao))

DA1 <- ((TAI0/DA0) (c i ![ ] NI0)?)

DA <- (sp? DA1)

PA00 <- ((g ia)/(g ua)/(p au)/(v au)/(f au)/(p ia)/(p ua)/(n ia)/(n ua)/(b iu)/(f ea)/(f ia)/(f ua)/(v ia)/(v ii)/(v iu)/(c oi)/(d au)/(d ii)/(d uo)/(f oi)/(f ui)/(g au)/(h ea)/(k au)/(k ii)/(k ui)/(l ia)/(l ui)/(m ia)/(n ui)/(p eu)/(r oi)/(r ui)/(s ea)/(s io)/(t ie)/(v ie)/(v a)/(v i)/(v u)/(p a)/(n a)/(f a)/(v a)/(KOU0 !(n oi) !KI))

PA0 <- (NI2? (n u !KOU0)? PA00 (n oi)? ZI?)

PAWORD0 <- ((KOU2/PA0)+ ((Comma2? CA0 Comma2?) (KOU2/PA0)+)*)

PAWORD <- (sp? PAWORD0)

PAPHRASE0 <- (((KOU2/PA0)+ (((Comma2? CA0 Comma2?)/(Comma2 !mod1a)) (KOU2/PA0)+)*) !modifier)

PAPHRASE <- (sp? PAPHRASE0)

GA <- (sp? (g a))

TENSE <- (PAPHRASE/GA)

ZI <- ((z i)/(z a)/(z u))

LE <- (sp? ((l ea)/(l eu)/(l oe)/(l ee)/(l aa)/(l e)/(l o)/(l a)))

LEFORPO <- (sp? ((l e)/(l o)/NI2))

LIO <- (sp? (l io))

LAU <- (sp? (l au))

LOU <- (sp? (l ou))

LUA <- (sp? (l ua))

LUO <- (sp? (l uo))

ZEIA <- (sp? z ei2 a)

ZEIO <- (sp? z ei2 o)

LIWORD <- (l i)

LUWORD <- (l u)

liquote <- ((sp? LIWORD Comma2? utterance0 Comma2? LUWORD)/(sp? LIWORD Comma2? [\"] utterance0 [\"] Comma2? LUWORD))

LAO <- (sp? &(LAOalien Juncture?) AlienWord)

LIE <- (sp? &(LIEalien Juncture?) AlienWord)

LIOALIEN <- (sp? &(LIOalien Juncture?) AlienWord)

Lw <- Cmapua

LIU0 <- ((l iu)/(n iu))

LNIU <- (([Ll]/[Nn]) [iI] Juncture? [Uu])

LIU <- ((sp? LNIU Juncture? !Vo1 Comma2? (PRENAME/Word) &(Comma/Terminal/End))/(sp? (l ii TAI)))

SUE <- (sp? &(([Ss] [Uu] Juncture? [Ee] Juncture?)/([Ss] [Aa] [Oo] Juncture?)) AlienWord)

CUI <- (sp? (c ui))

GATWO <- (sp? (g a))

GE <- (sp? (g e))

GEU <- (sp? ((c ue)/(g eu)))

GI <- (sp? ((g i)/(g oi)))

GO <- (sp? (g o))

GIO <- (sp? (g io))

GU <- (sp? (g u))

GUIZA <- (sp? (g ui) (z a))

GUIZI <- (sp? (g ui) (z i))

GUIZU <- (sp? (g ui) (z u))

GUI <- (!GUIZA !GUIZI !GUIZU (sp? (g ui)))

GUO <- (sp? (g uo))

GUOA <- (sp? ((g uo2 a)/(g uo z a)))

GUOE <- (sp? (g uo2 e))

GUOI <- (sp? ((g uo2 i)/(g uo z i)))

GUOO <- (sp? (g uo2 o))

GUOU <- (sp? ((g uo2 u)/(g uo z u)))

GUU <- (sp? (g uu))

GUUA <- (sp? (g uu2 a))

GIUO <- (sp? (g iu2 o))

GUE <- (sp? (g ue))

GUEA <- (sp? (g ue2 a))

JE <- (sp? (j e))

JUE <- (sp? (j ue))

JIZA <- (sp? ((j ie)/(j ae)/(p e)/(j i)/(j a)) (z a))

JIOZA <- (sp? ((j io)/(j ao)) (z a))

JIZI <- (sp? ((j ie)/(j ae)/(p e)/(j i)/(j a)) (z i))

JIOZI <- (sp? ((j io)/(j ao)) (z i))

JIZU <- (sp? ((j ie)/(j ae)/(p e)/(j i)/(j a)) (z u))

JIOZU <- (sp? ((j io)/(j ao)) (z u))

JI <- (!JIZA !JIZI !JIZU (sp? ((j ie)/(j ae)/(p e)/(j i)/(j a))))

NUJI <- (sp? n u !sp JI)

JIO <- (!JIOZA !JIOZI !JIOZU (sp? ((j io)/(j ao))))

DIO <- ((sp? ((b eu)/(c au)/(d io)/(f oa)/(k ao)/(j ui)/(n eu)/(p ou)/(g oa)/(s au)/(v eu)/(z ua)/(z ue)/(z ui)/(z uo)/(z uu))) ((c i ![ ] NI0)/ZI)?)

LAE <- (sp? ((l ae)/(l ue)))

ME <- (sp? ((m ea)/(m e)))

MEU <- (sp? m eu)

NU0 <- ((n uo)/(f uo)/(j uo)/(n u)/(f u)/(j u)/(k ue))

NU <- (sp? ((((n u)/(n uo)/(k ue)) !(sp (NI0/RA0)) (NI0/RA0)?)/NU0)+ freemod?)

PO1 <- (sp? ((p o)/(p u)/(z o)))

PO1A <- (sp? ((p oi2 a)/(p ui2 a)/(z oi2 a)/(p o z a)/(p u z a)/(z o z a)))

PO1E <- (sp? ((p oi2 e)/(p ui2 e)/(z oi2 e)))

PO1I <- (sp? ((p oi2 i)/(p ui2 i)/(z oi2 i)/(p o z i)/(p u z i)/(z o z i)))

PO1O <- (sp? ((p oi2 o)/(p ui2 o)/(z oi2 o)))

PO1U <- (sp? ((p oi2 u)/(p ui2 u)/(z oi2 u)/(p o z u)/(p u z u)/(z o z u)))

POSHORT1 <- (sp? ((p oi)/(p ui)/(z oi)))

PO <- (sp? PO1)

POA <- (sp? PO1A)

POE <- (sp? PO1E)

POI <- (sp? PO1E)

POO <- (sp? PO1O)

POU <- (sp? PO1U)

POSHORT <- (sp? POSHORT1)

DIE <- (sp? ((d ie)/(f ie)/(k ae)/(n ue)/(r ie)))

HOI <- (sp? h oi)

JO <- (sp? (NI0/RA0/SA0)? (j o))

KIE <- (sp? (k ie))

KIU <- (sp? (k iu))

KIE2 <- (sp? k ie Comma2? [(])

KIU2 <- (sp? [)] Comma2? k iu)

SOI <- (sp? (s oi))

UI0 <- (!Predstart ((!([Ii] Juncture? [Ee]) Vv Juncture?)/(b ea)/(b uo)/(c ea)/(c ia)/(c oa)/(d ou)/(f ae)/(f ao)/(f eu)/(g ea)/(k uo)/(k uu)/(r ea)/(n ao)/(n ie)/(p ae)/(p iu)/(s aa)/(s ui)/(t aa)/(t oe)/(v oi)/(z ou)/(l oi)/(l oa)/(s ia)/(s ii)/(t oe)/(s iu)/(c ao)/(c eu)/(s ie)/(s eu)/(s ie2 i)))

NOUI <- ((sp? UI0 NOI0)/(sp? n [o] Juncture? Comma? sp? UI0))

UI <- (sp? (UI0+/(NI f i)))

HUE <- (sp? (h ue))

NOWORD <- (sp? !KOU1 !NOUI (n o) !(Comma2? z ao Comma2? Predicate) !(sp? KOU0) !(sp? (JIO/JI/JIZA/JIOZA/JIZI/JIOZI/JIZU/JIOZU)))

ACRONYMICNAME <- (ACRONYM &(Comma/Period/End))

DJAN <- (PRENAME/ACRONYMICNAME)

BI <- (sp? (n u)? ((b ia)/(b ie)/(c ie)/(c io)/(b ia)/(b i)/(b ii)))

LWPREDA0 <- ((h e)/(d ua)/(d ui)/(b ua)/(b ui))

Predicate <- ((CmapuaUnit Comma2? z ao Comma2?)* Complex (Comma2? z ao Comma2? Predicate)?)

PREDA <- (sp? &caprule (Predicate/LWPREDA0/(![ ] NI RA0)))

guoa <- (OptPause? (GUOA/GU) freemod?)

guoe <- (OptPause? (GUOE/GU) freemod?)

guoi <- (OptPause? (GUOI/GU) freemod?)

guoo <- (OptPause? (GUOO/GU) freemod?)

guou <- (OptPause? (GUOU/GU) freemod?)

guo <- (!guoa !guoe !guoi !guoo !guou (OptPause? (GUO/GU) freemod?))

guiza <- (OptPause? (GUIZA/GU) freemod?)

guizi <- (OptPause? (GUIZI/GU) freemod?)

guizu <- (OptPause? (GUIZU/GU) freemod?)

gui <- (OptPause? (GUI/GU) freemod?)

gue <- (OptPause? (GUE/GU) freemod?)

guea <- (OptPause? (GUEA/GU) freemod?)

guu <- (OptPause? (GUU/GU) freemod?)

guua <- (OptPause? (GUUA/GU) freemod?)

giuo <- (OptPause? (GIUO/GU) freemod?)

meu <- (OptPause? (MEU/GU) freemod?)

geu <- GEU

gap <- (OptPause? GU freemod?)

HOI0 <- ((sp? (([Hh] oi)/([Ll] oi)/([Ll] oa)/([Ss] ia)/([Ss] ie)/([Ss] iu))) Juncture? !Vo1)

voc <- (((HOI/(HOI0 !(sp? FalseMarked))) Comma2? name)/(HOI Comma2? descpred guea? namesuffix?)/(HOI Comma2? argument1 guua?)/(sp? &([Hh] [Oo] [Ii] Juncture?) AlienWord))

HUE0 <- (sp? &caprule [Hh] [Uu] Juncture? [Ee] Juncture? !Vo1)

invvoc <- ((HUE0 Comma2? name)/(HUE freemod? descpred guea? namesuffix?)/(HUE freemod? argument1 &((freemod? gasent1a)/gasent2/(freemod? &subject statement)))/(HUE freemod? statement giuo?)/(HUE freemod? argument1 guu?)/(sp? &([Hh] [Uu] Juncture? [Ee] Juncture?) AlienWord))

kiamod <- (Comma2? !(!PRENAME !Predstart k ia) ((PRENAME/LIU/AlienWord/(Cmapua (sp? !(k ia) !PRENAME !Predstart Cmapua)*)/Word) kiamod* Comma2? !PRENAME !Predstart k ia) Comma2?)

freemod <- ((kiamod/NOUI/(SOI freemod? descpred guea?)/DIE/(NOWORD DIE)/(KIE Comma? utterance0 Comma? KIU)/(KIE2 Comma? utterance0 Comma? KIU2)/invvoc/voc/(Comma !(!FalseMarked PRENAME))/JO/UI/(sp? '...' (sp? &Letter)?)/(sp? '--' (sp? &Letter)?)) freemod?)

juelink <- (JUE freemod? (term/(PAPHRASE freemod? gap?)))

links1 <- (juelink (freemod? juelink)* gue?)

links <- ((links1/(KA freemod? links freemod? KI freemod? links1)) (freemod? AONE freemod? links1)*)

jelink <- (JE freemod? (term/(PAPHRASE freemod? gap?)))

linkargs1 <- (jelink freemod? (links/gue)?)

linkargs <- ((linkargs1/(KA freemod? linkargs freemod? KI freemod? linkargs1)) (freemod? AONE freemod? linkargs1)*)

abstractpred <- ((POA freemod? uttAxclone guoa?)/(POA freemod? sentenceclone guoa?)/(POE freemod? uttAxclone guoe?)/(POE freemod? sentenceclone guoe?)/(POI freemod? uttAxclone guoi?)/(POI freemod? sentenceclone guoi?)/(POO freemod? uttAxclone guoo?)/(POO freemod? sentenceclone guoo?)/(POU freemod? uttAxclone guou?)/(POU freemod? sentenceclone guou?)/(PO freemod? uttAxclone guo?)/(PO freemod? sentenceclone guo?))

predunit1 <- ((SUE/(NU freemod? GE freemod? despredE (freemod? geu Comma?)?)/(NU freemod? PREDA)/(Comma? GE freemod? descpred (freemod? geu Comma?)?)/abstractpred/(ME freemod? argument1 meu?)/PREDA) freemod?)

predunit2 <- ((NOWORD freemod?)* predunit1)

neg2 <- (!predunit2 NOWORD)

predunit3 <- ((predunit2 freemod? linkargs)/predunit2)

predunit <- ((POSHORT freemod?)? predunit3)

kekpredunit <- ((NOWORD freemod?)* KA freemod? predicate freemod? KI freemod? predicate guu?)

despredA <- ((predunit/kekpredunit) (freemod? CI freemod? (predunit/kekpredunit))*)

despredB <- ((!PREDA CUI freemod? despredC freemod? CA freemod? despredB)/despredA)

despredC <- (despredB (freemod? despredB)*)

despredD <- (despredB (freemod? CA freemod? despredB)*)

despredE <- (despredD (freemod? despredD)*)

descpred <- ((despredE freemod? GO freemod? descpred)/despredE)

sentpred <- ((despredE freemod? GO freemod? barepred)/despredE)

mod1a <- (PAWORD freemod? argument1 guua?)

mod1 <- ((PAWORD freemod? argument1 guua?)/(PAPHRASE freemod? !barepred gap?))

kekmod <- ((NOWORD freemod?)* (KA freemod? modifier freemod? KI freemod? mod))

mod <- (mod1/((NOWORD freemod?)* mod1)/kekmod)

modifier <- (mod (AONE freemod? mod)*)

name <- ((PRENAME/ACRONYMICNAME) ((Comma2? !FalseMarked PRENAME)/(Comma2? &([Cc] [Ii]) NAMEWORD)/(Comma2? CI predunit !(Comma2? (!FalseMarked PRENAME)))/(Comma2? CI ACRONYMICNAME))* freemod?)

LAWORD <- (sp? [Ll] [Aa] Juncture?)

LANAME <- (LAWORD Comma2? name)

descriptn <- (!LANAME ((LAU wordset1)/(LOU wordset2)/(LE freemod? ((!mex arg1a freemod?)? (PAPHRASE freemod?)?)? ((mex freemod? arg1a)/(mex freemod? descpred)/descpred))/(GE freemod? mex freemod? descpred)))

abstractn <- ((LEFORPO freemod? POA freemod? uttAxclone guoa?)/(LEFORPO freemod? POA freemod? sentenceclone guoa?)/(LEFORPO freemod? POE freemod? uttAxclone guoe?)/(LEFORPO freemod? POE freemod? sentenceclone guoe?)/(LEFORPO freemod? POI freemod? uttAxclone guoi?)/(LEFORPO freemod? POI freemod? sentenceclone guoi?)/(LEFORPO freemod? POO freemod? uttAxclone guoo?)/(LEFORPO freemod? POO freemod? sentenceclone guoo?)/(LEFORPO freemod? POU freemod? uttAxclone guou?)/(LEFORPO freemod? POU freemod? sentenceclone guou?)/(LEFORPO freemod? PO freemod? uttAxclone guo?)/(LEFORPO freemod? PO freemod? sentenceclone guo?))

Ciforsuffix <- ([Cc] [Ii])

namesuffix <- (&((Comma2 !FalseMarked PRENAME)/(sp? Ciforsuffix Juncture? Comma2? (PRENAME/ACRONYMICNAME))) ((sp? Ciforsuffix Juncture? Comma2?)/Comma2)? name)

arg1 <- (abstractn/(LIO freemod? descpred guea?)/(LIO freemod? argument1 guua?)/(LIO freemod? mex gap?)/LIOALIEN/LAO/LANAME/(descriptn guua? namesuffix?)/LIU/LIE/liquote)

arg1a <- ((DA/TAI/arg1/(GE freemod? arg1a)) freemod?)

argmod1 <- (((sp? (n o) sp?)? ((JI freemod? predicate)/(JIO freemod? sentence)/(JIO freemod? uttAx)/(JI freemod? modifier)/((JI/NUJI) freemod? argument1)))/((sp? (n o) sp?)? (((JIZA freemod? predicate) guiza?)/((JIOZA freemod? sentence) guiza?)/((JIOZA freemod? uttAx) guiza?)/((JIZA freemod? modifier) guiza?)/(JIZA freemod? argument1 guiza?)))/((sp? (n o) sp?)? ((JIZI freemod? predicate guizi?)/(JIOZI freemod? sentence guizi?)/(JIOZI freemod? uttAx guizi?)/(JIZI freemod? modifier guizi?)/(JIZI freemod? argument1 guizi?)))/((sp? (n o) sp?)? ((JIZU freemod? predicate guizu?)/(JIOZU freemod? sentence guizu?)/(JIOZU freemod? uttAx guizu?)/(JIZU freemod? modifier guizu?)/(JIZU freemod? argument1 guizu?))))

argmod <- (argmod1 (AONE freemod? argmod1)* gui?)

arg2 <- (arg1a freemod? argmod*)

arg3 <- (arg2/(mex freemod? arg2))

indef1 <- (mex freemod? descpred)

indef2 <- (indef1 guua? argmod*)

indefinite <- indef2

arg4 <- ((arg3/indefinite) (ZE freemod? (arg3/indefinite))*)

arg5 <- (arg4/(KA freemod? argument1 freemod? KI freemod? argx))

argx <- ((NOWORD freemod?)* (LAE freemod?)* arg5)

arg7 <- (argx freemod? (ACI freemod? argx)?)

arg8 <- (!GE (arg7 freemod? (AONE freemod? arg7)*))

argument1 <- (((arg8 freemod? AGE freemod? argument1)/arg8) (GUU freemod? argmod)*)

argument <- ((NOWORD freemod?)* (DIO freemod?)* argument1)

argxx <- (&((NOWORD freemod?)* DIO) argument)

term <- (argument/modifier)

modifiers <- (modifier (freemod? modifier)*)

modifiersx <- ((modifier/argxx) (freemod? (modifier/argxx))*)

subject <- ((modifiers freemod?)? ((argxx subject)/(argument (modifiersx freemod?)?)))

argumentA <- argument

argumentB <- argument

argumentC <- argument

argumentD <- argument

argumentA1 <- argument

argumentB1 <- argument

argumentC1 <- argument

argumentD1 <- argument

terms <- ((modifiersx? argumentA (freemod? modifiersx)? argumentB? (freemod? modifiersx)? argumentC? (freemod? modifiersx)? argumentD?)/modifiersx)

terms1 <- ((modifiersx? argumentA1 (freemod? modifiersx)? argumentB1? (freemod? modifiersx)? argumentC1? (freemod? modifiersx)? argumentD1?)/modifiersx)

word <- (arg1a/indef2)

words1 <- (word (ZEIA? word)*)

words2 <- (word (ZEIO? word)*)

wordset1 <- (words1? LUA)

wordset2 <- (words2? LUO)

termset1 <- (terms/(KA freemod? termset2 freemod? guu? KI freemod? termset1))

termset2 <- (termset1 (guu &AONE)? (AONE freemod? termset1 (guu &AONE)?)*)

termset <- ((terms freemod? GO freemod? barepred)/termset2)

barepred <- (sentpred freemod? ((termset guu?)/(guu &termset))?)

markpred <- (TENSE freemod? barepred)

backpred1 <- ((neg2 freemod?)* (barepred/markpred))

backpred <- (((backpred1 (ACI freemod? backpred1)+ freemod? ((termset guu?)/(guu &termset))?) ((ACI freemod? backpred)+ freemod? ((termset guu?)/(guu &termset))?)?)/backpred1)

predicate2 <- (!GE (((backpred (AONE !GE freemod? backpred)+ freemod? ((termset guu?)/(guu &termset))?) ((AONE freemod? predicate2)+ freemod? ((termset guu?)/(guu &termset))?)?)/backpred))

predicate1 <- ((predicate2 AGE freemod? predicate1)/predicate2)

identpred <- ((NOWORD freemod?)* (BI freemod? argument1 guu?))

predicate <- (predicate1/identpred)

gasent1 <- ((NOWORD freemod?)* (freemod? &markpred predicate (GATWO freemod? subject)?))

gasent1a <- ((NOWORD freemod?)* (freemod? &markpred predicate (GATWO freemod? subject)))

gasent2 <- ((NOWORD freemod?)* (TENSE freemod? sentpred modifiers? (GATWO freemod? subject freemod? GIO? freemod? terms?)))

gasent <- (gasent2/gasent1)

statement <- (gasent/(modifiers freemod? gasent)/(subject freemod? freemod? (GIO? freemod? terms1)? predicate))

keksent <- (terms? freemod? (NOWORD freemod?)* (KA freemod? headterms? freemod? sentence freemod? KI freemod? sen1))

keksentclone <- (terms? freemod? (NOWORD freemod?)* (KA freemod? headterms? freemod? sentenceclone freemod? KI freemod? sen1clone))

neghead <- ((NOWORD freemod? gap)/(neg2 OptPause))

imperative <- ((modifiers freemod?)? !gasent predicate)

nosubject <- ((modifiers freemod?)? !gasent predicate)

headterms <- (terms GI freemod?)+

uttAx <- (headterms freemod? sentence giuo?)

uttAxclone <- (headterms freemod? sentenceclone giuo?)

sen1 <- ((neghead freemod?)* (imperative/statement/keksent/uttAx))

sen1clone <- ((neghead freemod?)* (nosubject/statement/keksentclone/uttAxclone))

sentence1 <- (sen1 (ICI freemod? sen1)*)

sentence1clone <- (sen1clone (ICI freemod? sen1clone)*)

sentence <- (sentence1 ([!.:;?]? ICA freemod? sentence1)*)

sentenceclone <- (sentence1clone ([!.:;?]? ICA freemod? sentence1clone)*)

uttA <- ((AONE/mex) freemod?)

uttA1 <- ((links/linkargs/argmod/terms/uttA/NOWORD) freemod? Period?)

uttC <- ((sentence Period?)/(neghead freemod? uttC)/uttA1)

ICAUTT <- ICA

uttF <- (uttC (I freemod? uttC)*)

utterance0 <- (!GE ((ICAUTT freemod? uttF)/(!OptPause freemod Period? utterance0)/(!OptPause freemod Period?)/(uttF IGE utterance0)/uttF/(I freemod? uttF?)/(I freemod? Period?)) (&I utterance0)?)

utterance <- (&(PhoneticUtterance End) (!GE ((ICAUTT freemod? uttF (&I utterance)? End)/(!OptPause freemod Period? utterance)/(!OptPause freemod Period? (&I utterance)? End)/(uttF IGE utterance)/(I freemod? Period? (&I utterance)? End)/(uttF (&I utterance)? End)/(I freemod? uttF (&I utterance)? End))))
"#,
);

/// camxes-style Lojban **morphology** grammar (word level).
///
/// Format: `(start_rule, grammar_string)`. The start rule `text` matches a
/// sequence of `any_word`s; each `jbovla` node's first child names the word
/// class (`cmevla`, `cmavo`, `gismu`, `lujvo_core`, `fuhivla`) or a pause
/// (`pause_0`). Input is expected in lowercase with a trailing pause (space)
/// so that penultimate-stress lookahead can see the word end.
///
/// See [`crate::morphology`] for a normalized word-segmentation API on top of
/// this grammar.
pub const LOJBAN_MORPHOLOGY_GRAMMAR: (&str, &str) = (
    "text",
    r#"text <- any_word+

any_word <- jbovla

jbovla <- pause_0 / cmevla / (cmavo) / (gismu) / (!gismu !fuhivla !cmavo !(ccv h y onset) lujvo_core) / (fuhivla)

cmevla <- ((&zifcme any_syllable+ &pause) / zifcme)

gismu <- long_rafsi &stress &final_syllable pa_zei_karsna &post_word

fuhivla_head <- !rafsi_string !cmavo !(!rafsi_string zunsna rafsi_string) !h &onset unstressed_syllable*

fuhivla_trim <- fuhivla_head slaka &stress consonantal_syllable*

generic_fuhivla <- fuhivla_trim final_syllable

fuhivla <- &generic_fuhivla (ccv / cvv / cvc) (r / n / l) (unstressed_syllable* slaka &stress consonantal_syllable* final_syllable) / &generic_fuhivla (long_rafsi) (r / n / l) (unstressed_syllable* slaka &stress consonantal_syllable* final_syllable / final_syllable) / generic_fuhivla

cmavo <- !cmevla !(cvc !stress y h? lujvo_core / cvc &stress y short_final_rafsi) (!h !(zunsna zunsna+) onset (nucleus h)* nucleus / y+) &post_word

lujvo_core <- ((hy_rafsi / fuhivla_rafsi / y_rafsi / !any_fuhivla_rafsi y_less_rafsi !any_fuhivla_rafsi)*) ((fuhivla / gismu_cvv_final_rafsi) / (((stressed_hy_rafsi / stressed_fuhivla_rafsi / stressed_y_rafsi / cvc_ccv_cvv &stress)) (short_final_rafsi)))

any_fuhivla_rafsi <- fuhivla / fuhivla_rafsi / stressed_fuhivla_rafsi

rafsi_string <- y_less_rafsi* (gismu_cvv_final_rafsi / cvc_ccv_cvv &stress !y short_final_rafsi / y_rafsi / stressed_y_rafsi / (cvc_ccv_cvv &stress !y)? initial_pair y / hy_rafsi / stressed_hy_rafsi)

zifcme <- !h (nucleus / glaide / h / zunsna !pause)* zunsna &pause

stressed_fuhivla_rafsi <- (fuhivla_trim) (h y) / (fuhivla_trim onset) (y)

fuhivla_rafsi <- &unstressed_syllable (fuhivla_head) (h y) / (fuhivla_head onset) (y h?)

stressed_y_rafsi <- (long_rafsi / cvc) &stress (y)

y_rafsi <- ((long_rafsi / cvc) !stress) (y h?)

y_less_rafsi <- !y_rafsi !stressed_y_rafsi !hy_rafsi !stressed_hy_rafsi cvc_ccv_cvv !stress !y !h

stressed_hy_rafsi <- ((long_rafsi pa_zei_karsna / cvc_ccv_cvv)) &stress (h y)

hy_rafsi <- ((long_rafsi pa_zei_karsna / cvc_ccv_cvv)) (!stress h y h?)

cvc <- cv zunsna

cvc_ccv <- cvc / ccv

ccv <- initial_pair pa_zei_karsna

cvv <- zunsna re_zei_karsna

cvc_ccv_cvv <- cvc_ccv / cvvr

cvvr <- (zunsna pa_zei_karsna !stress h pa_zei_karsna / cvv) ((r &zunsna / n &r)?)

gismu_cvv_final_rafsi <- gismu / cv &stress h &final_syllable pa_zei_karsna &post_word

short_final_rafsi <- &final_syllable (zunsna re_zei_karsna / ccv) &post_word

unstressed_syllable <- slaka !stress / consonantal_syllable

long_rafsi <- cvc_ccv zunsna

cv <- zunsna pa_zei_karsna

final_syllable <- onset !y nucleus !cmevla &post_word

stress <- (zunsna / glaide)* h? y? slaka pause

any_syllable <- onset nucleus coda? / consonantal_syllable

slaka <- onset !y nucleus coda?

consonantal_syllable <- zunsna &syllabic coda

coda <- !any_syllable zunsna &any_syllable / syllabic? zunsna? &pause

onset <- (h / glaide / affricate / (cs !x / jz !(n / l / r))? (pfbgvkx / (t / d / n !r) !l / m)? (l / r)?) !zunsna !glaide

nucleus <- pa_zei_karsna / re_zei_karsna / y !nucleus

glaide <- (ii / w) &nucleus

re_zei_karsna <- ([a] w !u / [aeo] ii !i) !nucleus

pa_zei_karsna <- [aeiou] !nucleus

i <- [i]

u <- [u]

y <- [y] !(!y nucleus)

ii <- [i]

w <- [uw]

initial_pair <- &onset zunsna zunsna !zunsna

affricate <- t cs / d jz

zunsna <- pfbgvkx / d / jz / cs / t / syllabic

syllabic <- l / m / n / r

l <- [l]

m <- [m]

n <- [n] !affricate

r <- [r]

pfbgvkx <- [pfbgvkx]

d <- [d]

jz <- [jz]

cs <- [cs]

x <- [x]

t <- [t]

h <- [,'] &nucleus

post_word <- pause / !nucleus jbovla

pause <- pause_0 / !.

pause_0 <- [ ,.]+
"#,
);

/// Full Lojban **syntax** grammar (camxes), start rule `text`.
///
/// From ilmentufa's `camxes.peg` (<https://github.com/lojban/ilmentufa>),
/// based on camxes.js by Masato Hagiwara (MIT license), comments stripped.
/// The ilmentufa `#: PUSH` / `#: POP-EQ` / `#: PEEK-DIFF` annotations that
/// make `zoi`/`la'o` check matching delimiters are not supported by this
/// crate's PEG engine; see [`crate::syntax`] for the consequences.
pub const LOJBAN_GRAMMAR: (&str, &str) = (
    "text",
    r##"text <- intro_null NAI_clause* text_part_2 (!gek joik_jek)? text_1? faho_clause EOF?
intro_null <- initial_spaces? su_clause* intro_si_clause
text_part_2 <- (CMEVLA_clause+ / indicators?) free*
intro_si_clause <- si_clause? SI_clause*
faho_clause <- (FAhO_clause dot_star)?
text_1 <- I_clause (jek / joik)? (stag? BO_clause)? free* text_1? / NIhO_clause+ free* su_clause* paragraphs? / paragraphs
paragraphs <- paragraph? (NIhO_clause+ free* su_clause* paragraphs)?
paragraph <- (statement / fragment) (I_clause !jek !joik !joik_jek free* (statement / fragment)?)*
statement <- statement_1 / prenex statement
statement_1 <- statement_2 (I_clause joik_jek statement_2?)*
statement_2 <- statement_3 (I_clause (jek / joik)? stag? BO_clause free* statement_2?)?
statement_3 <- sentence / tag? TUhE_clause free* text_1 TUhU_elidible free*
fragment <- prenex / terms VAU_elidible free* / ek free* / gihek free* / quantifier / NA_clause !JA_clause free* / relative_clauses / links / linkargs
prenex <- terms ZOhU_clause free*
sentence <- (terms bridi_tail_sa* CU_elidible free*)? bridi_tail_sa* bridi_tail
sentence_sa <- sentence_start (!sentence_start (sa_word / SA_clause !sentence_start ) )* SA_clause &text_1
sentence_start <- I_pre / NIhO_pre
subsentence <- sentence / prenex subsentence
bridi_tail <- bridi_tail_1 (gihek stag? KE_clause free* bridi_tail KEhE_elidible free* tail_terms)?
bridi_tail_sa <- bridi_tail_start (term / !bridi_tail_start (sa_word / SA_clause !bridi_tail_start ) )* SA_clause &bridi_tail
bridi_tail_start <- ME_clause / NUhA_clause / NU_clause / NA_clause !KU_clause / NAhE_clause !BO_clause / selbri / tag bridi_tail_start / KE_clause bridi_tail_start / bridi_tail
bridi_tail_1 <- bridi_tail_2 (gihek !(stag? BO_clause) !(stag? KE_clause) free* bridi_tail_2 tail_terms)*
bridi_tail_2 <- bridi_tail_3 (gihek stag? BO_clause free* bridi_tail_2 tail_terms)?
bridi_tail_3 <- selbri tail_terms / gek_sentence
gek_sentence <- gek subsentence gik subsentence tail_terms / tag* KE_clause free* gek_sentence KEhE_elidible free* / NA_clause free* gek_sentence
tail_terms <- terms? VAU_elidible free*
terms <- terms_1+
terms_1 <- terms_2 (pehe_sa* PEhE_clause free* joik_jek terms_2)*
terms_2 <- term (cehe_sa* CEhE_clause free* nonabs_term)*
pehe_sa <- PEhE_clause (!PEhE_clause (sa_word / SA_clause !PEhE_clause))* SA_clause
cehe_sa <- CEhE_clause (!CEhE_clause (sa_word / SA_clause !CEhE_clause))* SA_clause
term <- term_sa* term_1
term_1 <- sumti / ( !gek (tag !(!tag selbri) / FA_clause free*) (sumti / KU_elidible free*) ) / termset / NA_clause KU_clause free*
nonabs_term <- term_sa* (sumti / ( !gek (tag / FA_clause free*) (sumti / KU_elidible free*) ) / termset / NA_clause KU_clause free*)
term_sa <- term_start (!term_start (sa_word / SA_clause !term_start ) )* SA_clause &term_1
term_start <- term_1 / LA_clause / LE_clause / LI_clause / LU_clause / LAhE_clause / quantifier term_start / gek sumti gik / FA_clause / tag term_start
termset <- gek_termset / NUhI_clause free* gek terms NUhU_elidible free* gik terms NUhU_elidible free* / NUhI_clause free* terms NUhU_elidible free*
gek_termset <- gek terms_gik_terms
terms_gik_terms <- nonabs_term (gik / terms_gik_terms) nonabs_term
sumti <- sumti_1 (VUhO_clause free* relative_clauses)?
sumti_1 <- sumti_2 (joik_ek stag? KE_clause free* sumti KEhE_elidible free*)?
sumti_2 <- sumti_3 (joik_ek sumti_3)*
sumti_3 <- sumti_4 (joik_ek stag? BO_clause free* sumti_3)?
sumti_4 <- sumti_5 / gek sumti gik sumti_4
sumti_5 <- quantifier? sumti_6 relative_clauses? / quantifier selbri KU_elidible free* relative_clauses?
sumti_6 <- ZO_clause free* / ZOI_clause free* / LOhU_clause free* / lerfu_string !MOI_clause BOI_elidible free* / LU_clause text LIhU_elidible free* / (LAhE_clause free* / NAhE_clause BO_clause free*) relative_clauses? sumti LUhU_elidible free* / KOhA_clause free* / LA_clause free* relative_clauses? CMEVLA_clause+ free* / (LA_clause / LE_clause) free* sumti_tail KU_elidible free* / li_clause
li_clause <- LI_clause free* mex LOhO_elidible free*
sumti_tail <- (sumti_6 relative_clauses?)? sumti_tail_1 / relative_clauses sumti_tail_1
sumti_tail_1 <- selbri relative_clauses? / quantifier selbri relative_clauses? / quantifier sumti
relative_clauses <- relative_clause (ZIhE_clause free* relative_clause)*
relative_clause <- relative_clause_sa* relative_clause_1
relative_clause_sa <- relative_clause_start (!relative_clause_start (sa_word / SA_clause !relative_clause_start ) )* SA_clause &relative_clause_1
relative_clause_1 <- GOI_clause free* nonabs_term GEhU_elidible free* / NOI_clause free* subsentence KUhO_elidible free*
relative_clause_start <- GOI_clause / NOI_clause
selbri <- tag? selbri_1
selbri_1 <- selbri_2 / NA_clause free* selbri
selbri_2 <- selbri_3 (CO_clause free* selbri_2)?
selbri_3 <- selbri_4+
selbri_4 <- selbri_5 (joik_jek selbri_5 / joik stag? KE_clause free* selbri_3 KEhE_elidible free*)*
selbri_5 <- selbri_6 ((jek / joik) stag? BO_clause free* selbri_5)?
selbri_6 <- tanru_unit (BO_clause free* selbri_6)? / NAhE_clause? free* guhek selbri gik selbri_6
tanru_unit <- tanru_unit_1 (CEI_clause free* tanru_unit_1)*
tanru_unit_1 <- tanru_unit_2 linkargs?
tanru_unit_2 <- BRIVLA_clause free* / GOhA_clause RAhO_clause? free* / KE_clause free* selbri_3 KEhE_elidible free* / ME_clause free* (sumti / lerfu_string) MEhU_elidible free* MOI_clause? free* / (number / lerfu_string) MOI_clause free* / NUhA_clause free* mex_operator / SE_clause free* tanru_unit_2 / JAI_clause free* tag? tanru_unit_2 / NAhE_clause free* tanru_unit_2 / NU_clause NAI_clause? free* (joik_jek NU_clause NAI_clause? free*)* subsentence KEI_elidible free*
linkargs <- linkargs_sa* linkargs_1
linkargs_1 <- BE_clause free* nonabs_term links? BEhO_elidible free*
linkargs_sa <- linkargs_start (!linkargs_start (sa_word / SA_clause !linkargs_start ) )* SA_clause &linkargs_1
linkargs_start <- BE_clause
links <- links_sa* links_1
links_1 <- BEI_clause free* nonabs_term links?
links_sa <- links_start (!links_start (sa_word / SA_clause !links_start ) )* SA_clause &links_1
links_start <- BEI_clause
quantifier <- number !MOI_clause BOI_elidible free* / VEI_clause free* mex VEhO_elidible free*
mex <- mex_sa* mex_0
mex_0 <- mex_1 (operator mex_1)* / rp_clause
mex_sa <- mex_start (!mex_start (sa_word / SA_clause !mex_start) )* SA_clause &mex_0
mex_start <- FUhA_clause / PEhO_clause / operand_start
rp_clause <- FUhA_clause free* rp_expression
mex_1 <- mex_2 (BIhE_clause free* operator mex_1)?
mex_2 <- operand / mex_forethought
mex_forethought <- PEhO_clause? free* operator fore_operands KUhE_elidible free*
fore_operands <- mex_2+
rp_expression <- operand rp_expression_tail
rp_expression_tail <- rp_expression operator rp_expression_tail / ""
operator <- operator_sa* operator_0
operator_0 <- operator_1 (joik_jek operator_1 / joik stag? KE_clause free* operator KEhE_elidible free*)*
operator_sa <- operator_start (!operator_start (sa_word / SA_clause !operator_start) )* SA_clause &operator_0
operator_start <- guhek / KE_clause / SE_clause? NAhE_clause / SE_clause? MAhO_clause / SE_clause? VUhU_clause
operator_1 <- operator_2 / guhek operator_1 gik operator_2 / operator_2 (jek / joik) stag? BO_clause free* operator_1
operator_2 <- mex_operator / KE_clause free* operator KEhE_elidible free*
mex_operator <- SE_clause free* mex_operator / NAhE_clause free* mex_operator / MAhO_clause free* mex TEhU_elidible free* / NAhU_clause free* selbri TEhU_elidible free* / VUhU_clause free*
operand <- operand_sa* operand_0
operand_0 <- operand_1 (joik_ek stag? KE_clause free* operand KEhE_elidible free*)?
operand_sa <- operand_start (!operand_start (sa_word / SA_clause !operand_start) )* SA_clause &operand_0
operand_start <- quantifier / lerfu_word / NIhE_clause / MOhE_clause / JOhI_clause / gek / LAhE_clause / NAhE_clause
operand_1 <- operand_2 (joik_ek operand_2)*
operand_2 <- operand_3 (joik_ek stag? BO_clause free* operand_2)?
operand_3 <- quantifier / lerfu_string !MOI_clause BOI_elidible free* / NIhE_clause free* selbri TEhU_elidible free* / MOhE_clause free* sumti TEhU_elidible free* / JOhI_clause free* mex_2+ TEhU_elidible free* / gek operand gik operand_3 / (LAhE_clause free* / NAhE_clause BO_clause free*) operand LUhU_elidible free*
number <- PA_clause (PA_clause / lerfu_word)*
lerfu_string <- lerfu_word (PA_clause / lerfu_word)*
lerfu_word <- BY_clause / LAU_clause lerfu_word / TEI_clause lerfu_string FOI_clause
ek <- NA_clause? SE_clause? A_clause NAI_clause?
gihek <- gihek_sa* gihek_1
gihek_1 <- NA_clause? SE_clause? GIhA_clause NAI_clause?
gihek_sa <- gihek_1 (!gihek_1 (sa_word / SA_clause !gihek_1 ) )* SA_clause &gihek
jek <- NA_clause? SE_clause? JA_clause NAI_clause?
joik <- SE_clause? JOI_clause NAI_clause? / interval / GAhO_clause interval GAhO_clause
interval <- SE_clause? BIhI_clause NAI_clause?
joik_ek <- joik_ek_sa* joik_ek_1
joik_ek_1 <- joik free* / ek free*
joik_ek_sa <- joik_ek_1 (!joik_ek_1 (sa_word / SA_clause !joik_ek_1 ) )* SA_clause &joik_ek
joik_jek <- joik free* / jek free*
gek <- SE_clause? GA_clause NAI_clause? free* / joik GI_clause free* / stag gik
guhek <- SE_clause? GUhA_clause NAI_clause? free*
gik <- GI_clause NAI_clause? free*
tag <- tense_modal (joik_jek tense_modal)*
stag <- simple_tense_modal ((jek / joik) simple_tense_modal)* / tense_modal (joik_jek tense_modal)*
tense_modal <- simple_tense_modal free* / FIhO_clause free* selbri FEhU_elidible free*
simple_tense_modal <- NAhE_clause? SE_clause? BAI_clause NAI_clause? KI_clause? / NAhE_clause? ( ((time space? / space time?) CAhA_clause) / (time space? / space time?) / CAhA_clause ) KI_clause? / KI_clause / CUhE_clause
time <- ZI_clause time_offset* (ZEhA_clause (PU_clause NAI_clause?)?)? interval_property* / ZI_clause? time_offset+ (ZEhA_clause (PU_clause NAI_clause?)?)? interval_property* / ZI_clause? time_offset* ZEhA_clause (PU_clause NAI_clause?)? interval_property* / ZI_clause? time_offset* (ZEhA_clause (PU_clause NAI_clause?)?)? interval_property+
time_offset <- PU_clause NAI_clause? ZI_clause?
space <- VA_clause space_offset* space_interval? (MOhI_clause space_offset)? / VA_clause? space_offset+ space_interval? (MOhI_clause space_offset)? / VA_clause? space_offset* space_interval (MOhI_clause space_offset)? / VA_clause? space_offset* space_interval? MOhI_clause space_offset
space_offset <- FAhA_clause NAI_clause? VA_clause?
space_interval <- (VEhA_clause VIhA_clause? / VIhA_clause) (FAhA_clause NAI_clause?)? space_int_props? / space_int_props
space_int_props <- (FEhE_clause interval_property)+
interval_property <- number ROI_clause NAI_clause? / TAhE_clause NAI_clause? / ZAhO_clause NAI_clause?
free <- SEI_clause free* (terms CU_elidible free*)? selbri SEhU_elidible / SOI_clause free* sumti sumti? SEhU_elidible / vocative relative_clauses? selbri relative_clauses? DOhU_elidible / vocative relative_clauses? CMEVLA_clause+ free* relative_clauses? DOhU_elidible / vocative sumti? DOhU_elidible / (number / lerfu_string) MAI_clause / TO_clause text TOI_elidible / xi_clause
xi_clause <- XI_clause free* (number / lerfu_string) BOI_elidible / XI_clause free* VEI_clause free* mex VEhO_elidible
vocative <- (COI_clause NAI_clause?)+ DOI_clause / (COI_clause NAI_clause?) (COI_clause NAI_clause?)* / DOI_clause
indicators <- FUhE_clause? indicator+
indicator <- ((UI_clause / CAI_clause) NAI_clause? / DAhO_clause / FUhO_clause) !BU_clause
zei_clause <- pre_clause zei_clause_no_pre
zei_clause_no_pre <- pre_zei_bu (zei_tail? BU_clause+)* zei_tail post_clause
bu_clause <- pre_clause bu_clause_no_pre
bu_clause_no_pre <- pre_zei_bu (BU_clause* zei_tail)* BU_clause+ post_clause
zei_tail <- (ZEI_clause any_word)+
bu_tail <- BU_clause+
pre_zei_bu <- !ZOI_start !BU_clause !ZEI_clause !SI_clause !SA_clause !SU_clause !FAhO_clause any_word_SA_handling si_clause?
dot_star <- .*
post_clause <- spaces? si_clause? !ZEI_clause !BU_clause indicators*
pre_clause <- BAhE_clause*
any_word_SA_handling <- BRIVLA_pre / known_cmavo_SA / CMAVO_pre / CMEVLA_pre
known_cmavo_SA <- A_pre / BAI_pre / BAhE_pre / BE_pre / BEI_pre / BEhO_pre / BIhE_pre / BIhI_pre / BO_pre / BOI_pre / BU_pre / BY_pre / CAI_pre / CAhA_pre / CEI_pre / CEhE_pre / CO_pre / COI_pre / CU_pre / CUhE_pre / DAhO_pre / DOI_pre / DOhU_pre / FA_pre / FAhA_pre / FEhE_pre / FEhU_pre / FIhO_pre / FOI_pre / FUhA_pre / FUhE_pre / FUhO_pre / GA_pre / GAhO_pre / GEhU_pre / GI_pre / GIhA_pre / GOI_pre / GOhA_pre / GUhA_pre / I_pre / JA_pre / JAI_pre / JOI_pre / JOhI_pre / KE_pre / KEI_pre / KEhE_pre / KI_pre / KOhA_pre / KU_pre / KUhE_pre / KUhO_pre / LA_pre / LAU_pre / LAhE_pre / LE_pre / LEhU_pre / LI_pre / LIhU_pre / LOhO_pre / LOhU_pre / LU_pre / LUhU_pre / MAI_pre / MAhO_pre / ME_pre / MEhU_pre / MOI_pre / MOhE_pre / MOhI_pre / NA_pre / NAI_pre / NAhE_pre / NAhU_pre / NIhE_pre / NIhO_pre / NOI_pre / NU_pre / NUhA_pre / NUhI_pre / NUhU_pre / PA_pre / PEhE_pre / PEhO_pre / PU_pre / RAhO_pre / ROI_pre / SA_pre / SE_pre / SEI_pre / SEhU_pre / SI_clause / SOI_pre / SU_pre / TAhE_pre / TEI_pre / TEhU_pre / TO_pre / TOI_pre / TUhE_pre / TUhU_pre / UI_pre / VA_pre / VAU_pre / VEI_pre / VEhA_pre / VEhO_pre / VIhA_pre / VUhO_pre / VUhU_pre / XI_pre / ZAhO_pre / ZEI_pre / ZEhA_pre / ZI_pre / ZIhE_pre / ZO_pre / ZOI_pre / ZOhU_pre
su_clause <- (erasable_clause / su_word)* SU_clause
si_clause <- ((erasable_clause / si_word / SA_clause) si_clause? SI_clause)+
erasable_clause <- bu_clause_no_pre !ZEI_clause !BU_clause / zei_clause_no_pre !ZEI_clause !BU_clause
sa_word <- pre_zei_bu
si_word <- pre_zei_bu
su_word <- !ZOI_start !NIhO_clause !LU_clause !TUhE_clause !TO_clause !SU_clause !FAhO_clause any_word_SA_handling
BEhO_elidible <- BEhO_clause?
BOI_elidible <- BOI_clause?
CU_elidible <- CU_clause?
DOhU_elidible <- DOhU_clause?
FEhU_elidible <- FEhU_clause?
GEhU_elidible <- GEhU_clause?
KEI_elidible <- KEI_clause?
KEhE_elidible <- KEhE_clause?
KU_elidible <- KU_clause?
KUhE_elidible <- KUhE_clause?
KUhO_elidible <- KUhO_clause?
LIhU_elidible <- LIhU_clause?
LOhO_elidible <- LOhO_clause?
LUhU_elidible <- LUhU_clause?
MEhU_elidible <- MEhU_clause?
NUhU_elidible <- NUhU_clause?
SEhU_elidible <- SEhU_clause?
TEhU_elidible <- TEhU_clause?
TOI_elidible <- TOI_clause?
TUhU_elidible <- TUhU_clause?
VAU_elidible <- VAU_clause?
VEhO_elidible <- VEhO_clause?
BRIVLA_clause <- BRIVLA_pre BRIVLA_post / zei_clause
BRIVLA_pre <- pre_clause BRIVLA spaces?
BRIVLA_post <- post_clause
CMEVLA_clause <- CMEVLA_pre CMEVLA_post
CMEVLA_pre <- pre_clause CMEVLA spaces?
CMEVLA_post <- post_clause
CMAVO_clause <- CMAVO_pre CMAVO_post
CMAVO_pre <- pre_clause CMAVO spaces?
CMAVO_post <- post_clause
A_clause <- A_pre A_post
A_pre <- pre_clause A spaces?
A_post <- post_clause
BAI_clause <- BAI_pre BAI_post
BAI_pre <- pre_clause BAI spaces?
BAI_post <- post_clause
BAhE_clause <- BAhE_pre BAhE_post
BAhE_pre <- BAhE spaces?
BAhE_post <- si_clause? !ZEI_clause !BU_clause
BE_clause <- BE_pre BE_post
BE_pre <- pre_clause BE spaces?
BE_post <- post_clause
BEI_clause <- BEI_pre BEI_post
BEI_pre <- pre_clause BEI spaces?
BEI_post <- post_clause
BEhO_clause <- BEhO_pre BEhO_post
BEhO_pre <- pre_clause BEhO spaces?
BEhO_post <- post_clause
BIhE_clause <- BIhE_pre BIhE_post
BIhE_pre <- pre_clause BIhE spaces?
BIhE_post <- post_clause
BIhI_clause <- BIhI_pre BIhI_post
BIhI_pre <- pre_clause BIhI spaces?
BIhI_post <- post_clause
BO_clause <- BO_pre BO_post
BO_pre <- pre_clause BO spaces?
BO_post <- post_clause
BOI_clause <- BOI_pre BOI_post
BOI_pre <- pre_clause BOI spaces?
BOI_post <- post_clause
BU_clause <- BU_pre BU_post
BU_pre <- pre_clause BU spaces?
BU_post <- spaces?
BY_clause <- BY_pre BY_post / bu_clause
BY_pre <- pre_clause BY spaces?
BY_post <- post_clause
CAhA_clause <- CAhA_pre CAhA_post
CAhA_pre <- pre_clause CAhA spaces?
CAhA_post <- post_clause
CAI_clause <- CAI_pre CAI_post
CAI_pre <- pre_clause CAI spaces?
CAI_post <- post_clause
CEI_clause <- CEI_pre CEI_post
CEI_pre <- pre_clause CEI spaces?
CEI_post <- post_clause
CEhE_clause <- CEhE_pre CEhE_post
CEhE_pre <- pre_clause CEhE spaces?
CEhE_post <- post_clause
CO_clause <- CO_pre CO_post
CO_pre <- pre_clause CO spaces?
CO_post <- post_clause
COI_clause <- COI_pre COI_post
COI_pre <- pre_clause COI spaces?
COI_post <- post_clause
CU_clause <- CU_pre CU_post
CU_pre <- pre_clause CU spaces?
CU_post <- post_clause
CUhE_clause <- CUhE_pre CUhE_post
CUhE_pre <- pre_clause CUhE spaces?
CUhE_post <- post_clause
DAhO_clause <- DAhO_pre DAhO_post
DAhO_pre <- pre_clause DAhO spaces?
DAhO_post <- post_clause
DOI_clause <- DOI_pre DOI_post
DOI_pre <- pre_clause DOI spaces?
DOI_post <- post_clause
DOhU_clause <- DOhU_pre DOhU_post
DOhU_pre <- pre_clause DOhU spaces?
DOhU_post <- post_clause
FA_clause <- FA_pre FA_post
FA_pre <- pre_clause FA spaces?
FA_post <- post_clause
FAhA_clause <- FAhA_pre FAhA_post
FAhA_pre <- pre_clause FAhA spaces?
FAhA_post <- post_clause
FAhO_clause <- pre_clause FAhO spaces?
FEhE_clause <- FEhE_pre FEhE_post
FEhE_pre <- pre_clause FEhE spaces?
FEhE_post <- post_clause
FEhU_clause <- FEhU_pre FEhU_post
FEhU_pre <- pre_clause FEhU spaces?
FEhU_post <- post_clause
FIhO_clause <- FIhO_pre FIhO_post
FIhO_pre <- pre_clause FIhO spaces?
FIhO_post <- post_clause
FOI_clause <- FOI_pre FOI_post
FOI_pre <- pre_clause FOI spaces?
FOI_post <- post_clause
FUhA_clause <- FUhA_pre FUhA_post
FUhA_pre <- pre_clause FUhA spaces?
FUhA_post <- post_clause
FUhE_clause <- FUhE_pre FUhE_post
FUhE_pre <- pre_clause FUhE spaces?
FUhE_post <- !BU_clause spaces? !ZEI_clause !BU_clause
FUhO_clause <- FUhO_pre FUhO_post
FUhO_pre <- pre_clause FUhO spaces?
FUhO_post <- post_clause
GA_clause <- GA_pre GA_post
GA_pre <- pre_clause GA spaces?
GA_post <- post_clause
GAhO_clause <- GAhO_pre GAhO_post
GAhO_pre <- pre_clause GAhO spaces?
GAhO_post <- post_clause
GEhU_clause <- GEhU_pre GEhU_post
GEhU_pre <- pre_clause GEhU spaces?
GEhU_post <- post_clause
GI_clause <- GI_pre GI_post
GI_pre <- pre_clause GI spaces?
GI_post <- post_clause
GIhA_clause <- GIhA_pre GIhA_post
GIhA_pre <- pre_clause GIhA spaces?
GIhA_post <- post_clause
GOI_clause <- GOI_pre GOI_post
GOI_pre <- pre_clause GOI spaces?
GOI_post <- post_clause
GOhA_clause <- GOhA_pre GOhA_post
GOhA_pre <- pre_clause GOhA spaces?
GOhA_post <- post_clause
GUhA_clause <- GUhA_pre GUhA_post
GUhA_pre <- pre_clause GUhA spaces?
GUhA_post <- post_clause
I_clause <- sentence_sa* I_pre I_post
I_pre <- pre_clause I spaces?
I_post <- post_clause
JA_clause <- JA_pre JA_post
JA_pre <- pre_clause JA spaces?
JA_post <- post_clause
JAI_clause <- JAI_pre JAI_post
JAI_pre <- pre_clause JAI spaces?
JAI_post <- post_clause
JOhI_clause <- JOhI_pre JOhI_post
JOhI_pre <- pre_clause JOhI spaces?
JOhI_post <- post_clause
JOI_clause <- JOI_pre JOI_post
JOI_pre <- pre_clause JOI spaces?
JOI_post <- post_clause
KE_clause <- KE_pre KE_post
KE_pre <- pre_clause KE spaces?
KE_post <- post_clause
KEhE_clause <- KEhE_pre KEhE_post
KEhE_pre <- pre_clause KEhE spaces?
KEhE_post <- post_clause
KEI_clause <- KEI_pre KEI_post
KEI_pre <- pre_clause KEI spaces?
KEI_post <- post_clause
KEI_no_SA_handling <- pre_clause KEI post_clause
KI_clause <- KI_pre KI_post
KI_pre <- pre_clause KI spaces?
KI_post <- post_clause
KOhA_clause <- KOhA_pre KOhA_post
KOhA_pre <- pre_clause KOhA spaces?
KOhA_post <- post_clause
KU_clause <- KU_pre KU_post
KU_pre <- pre_clause KU spaces?
KU_post <- post_clause
KUhE_clause <- KUhE_pre KUhE_post
KUhE_pre <- pre_clause KUhE spaces?
KUhE_post <- post_clause
KUhO_clause <- KUhO_pre KUhO_post
KUhO_pre <- pre_clause KUhO spaces?
KUhO_post <- post_clause
LA_clause <- LA_pre LA_post
LA_pre <- pre_clause LA spaces?
LA_post <- post_clause
LAU_clause <- LAU_pre LAU_post
LAU_pre <- pre_clause LAU spaces?
LAU_post <- post_clause
LAhE_clause <- LAhE_pre LAhE_post
LAhE_pre <- pre_clause LAhE spaces?
LAhE_post <- post_clause
LE_clause <- LE_pre LE_post
LE_pre <- pre_clause LE spaces?
LE_post <- post_clause
LEhU_clause <- LEhU_pre LEhU_post
LEhU_pre <- pre_clause LEhU spaces?
LEhU_post <- spaces?
LI_clause <- LI_pre LI_post
LI_pre <- pre_clause LI spaces?
LI_post <- post_clause
LIhU_clause <- LIhU_pre LIhU_post
LIhU_pre <- pre_clause LIhU spaces?
LIhU_post <- post_clause
LOhO_clause <- LOhO_pre LOhO_post
LOhO_pre <- pre_clause LOhO spaces?
LOhO_post <- post_clause
LOhU_clause <- LOhU_pre LOhU_post
LOhU_pre <- pre_clause LOhU spaces? (!LEhU any_word)* LEhU_clause spaces?
LOhU_post <- post_clause
LU_clause <- LU_pre LU_post
LU_pre <- pre_clause LU spaces?
LU_post <- spaces? si_clause? !ZEI_clause !BU_clause
LUhU_clause <- LUhU_pre LUhU_post
LUhU_pre <- pre_clause LUhU spaces?
LUhU_post <- post_clause
MAhO_clause <- MAhO_pre MAhO_post
MAhO_pre <- pre_clause MAhO spaces?
MAhO_post <- post_clause
MAI_clause <- MAI_pre MAI_post
MAI_pre <- pre_clause MAI spaces?
MAI_post <- post_clause
ME_clause <- ME_pre ME_post
ME_pre <- pre_clause ME spaces?
ME_post <- post_clause
MEhU_clause <- MEhU_pre MEhU_post
MEhU_pre <- pre_clause MEhU spaces?
MEhU_post <- post_clause
MOhE_clause <- MOhE_pre MOhE_post
MOhE_pre <- pre_clause MOhE spaces?
MOhE_post <- post_clause
MOhI_clause <- MOhI_pre MOhI_post
MOhI_pre <- pre_clause MOhI spaces?
MOhI_post <- post_clause
MOI_clause <- MOI_pre MOI_post
MOI_pre <- pre_clause MOI spaces?
MOI_post <- post_clause
NA_clause <- NA_pre NA_post
NA_pre <- pre_clause NA spaces?
NA_post <- post_clause
NAI_clause <- NAI_pre NAI_post
NAI_pre <- pre_clause NAI spaces?
NAI_post <- post_clause
NAhE_clause <- NAhE_pre NAhE_post
NAhE_pre <- pre_clause NAhE spaces?
NAhE_post <- post_clause
NAhU_clause <- NAhU_pre NAhU_post
NAhU_pre <- pre_clause NAhU spaces?
NAhU_post <- post_clause
NIhE_clause <- NIhE_pre NIhE_post
NIhE_pre <- pre_clause NIhE spaces?
NIhE_post <- post_clause
NIhO_clause <- sentence_sa* NIhO_pre NIhO_post
NIhO_pre <- pre_clause NIhO spaces?
NIhO_post <- su_clause* post_clause
NOI_clause <- NOI_pre NOI_post
NOI_pre <- pre_clause NOI spaces?
NOI_post <- post_clause
NU_clause <- NU_pre NU_post
NU_pre <- pre_clause NU spaces?
NU_post <- post_clause
NUhA_clause <- NUhA_pre NUhA_post
NUhA_pre <- pre_clause NUhA spaces?
NUhA_post <- post_clause
NUhI_clause <- NUhI_pre NUhI_post
NUhI_pre <- pre_clause NUhI spaces?
NUhI_post <- post_clause
NUhU_clause <- NUhU_pre NUhU_post
NUhU_pre <- pre_clause NUhU spaces?
NUhU_post <- post_clause
PA_clause <- PA_pre PA_post
PA_pre <- pre_clause PA spaces?
PA_post <- post_clause
PEhE_clause <- PEhE_pre PEhE_post
PEhE_pre <- pre_clause PEhE spaces?
PEhE_post <- post_clause
PEhO_clause <- PEhO_pre PEhO_post
PEhO_pre <- pre_clause PEhO spaces?
PEhO_post <- post_clause
PU_clause <- PU_pre PU_post
PU_pre <- pre_clause PU spaces?
PU_post <- post_clause
RAhO_clause <- RAhO_pre RAhO_post
RAhO_pre <- pre_clause RAhO spaces?
RAhO_post <- post_clause
ROI_clause <- ROI_pre ROI_post
ROI_pre <- pre_clause ROI spaces?
ROI_post <- post_clause
SA_clause <- SA_pre SA_post
SA_pre <- pre_clause SA spaces?
SA_post <- spaces?
SE_clause <- SE_pre SE_post
SE_pre <- pre_clause SE spaces?
SE_post <- post_clause
SEI_clause <- SEI_pre SEI_post
SEI_pre <- pre_clause SEI spaces?
SEI_post <- post_clause
SEhU_clause <- SEhU_pre SEhU_post
SEhU_pre <- pre_clause SEhU spaces?
SEhU_post <- post_clause
SI_clause <- spaces? SI spaces?
SOI_clause <- SOI_pre SOI_post
SOI_pre <- pre_clause SOI spaces?
SOI_post <- post_clause
SU_clause <- SU_pre SU_post
SU_pre <- pre_clause SU spaces?
SU_post <- post_clause
TAhE_clause <- TAhE_pre TAhE_post
TAhE_pre <- pre_clause TAhE spaces?
TAhE_post <- post_clause
TEhU_clause <- TEhU_pre TEhU_post
TEhU_pre <- pre_clause TEhU spaces?
TEhU_post <- post_clause
TEI_clause <- TEI_pre TEI_post
TEI_pre <- pre_clause TEI spaces?
TEI_post <- post_clause
TO_clause <- TO_pre TO_post
TO_pre <- pre_clause TO spaces?
TO_post <- post_clause
TOI_clause <- TOI_pre TOI_post
TOI_pre <- pre_clause TOI spaces?
TOI_post <- post_clause
TUhE_clause <- TUhE_pre TUhE_post
TUhE_pre <- pre_clause TUhE spaces?
TUhE_post <- su_clause* post_clause
TUhU_clause <- TUhU_pre TUhU_post
TUhU_pre <- pre_clause TUhU spaces?
TUhU_post <- post_clause
UI_clause <- UI_pre UI_post
UI_pre <- pre_clause UI spaces?
UI_post <- post_clause
VA_clause <- VA_pre VA_post
VA_pre <- pre_clause VA spaces?
VA_post <- post_clause
VAU_clause <- VAU_pre VAU_post
VAU_pre <- pre_clause VAU spaces?
VAU_post <- post_clause
VEI_clause <- VEI_pre VEI_post
VEI_pre <- pre_clause VEI spaces?
VEI_post <- post_clause
VEhO_clause <- VEhO_pre VEhO_post
VEhO_pre <- pre_clause VEhO spaces?
VEhO_post <- post_clause
VUhU_clause <- VUhU_pre VUhU_post
VUhU_pre <- pre_clause VUhU spaces?
VUhU_post <- post_clause
VEhA_clause <- VEhA_pre VEhA_post
VEhA_pre <- pre_clause VEhA spaces?
VEhA_post <- post_clause
VIhA_clause <- VIhA_pre VIhA_post
VIhA_pre <- pre_clause VIhA spaces?
VIhA_post <- post_clause
VUhO_clause <- VUhO_pre VUhO_post
VUhO_pre <- pre_clause VUhO spaces?
VUhO_post <- post_clause
XI_clause <- XI_pre XI_post
XI_pre <- pre_clause XI spaces?
XI_post <- post_clause
ZAhO_clause <- ZAhO_pre ZAhO_post
ZAhO_pre <- pre_clause ZAhO spaces?
ZAhO_post <- post_clause
ZEhA_clause <- ZEhA_pre ZEhA_post
ZEhA_pre <- pre_clause ZEhA spaces?
ZEhA_post <- post_clause
ZEI_clause <- ZEI_pre ZEI_post
ZEI_pre <- pre_clause ZEI spaces?
ZEI_post <- spaces?
ZI_clause <- ZI_pre ZI_post
ZI_pre <- pre_clause ZI spaces?
ZI_post <- post_clause
ZIhE_clause <- ZIhE_pre ZIhE_post
ZIhE_pre <- pre_clause ZIhE spaces?
ZIhE_post <- post_clause
ZO_clause <- ZO_pre ZO_post
ZO_pre <- pre_clause ZO spaces? any_word spaces?
ZO_post <- post_clause
ZOI_clause <- ZOI_pre ZOI_post
ZOI_pre <- pre_clause ZOI spaces? zoi_open spaces? (zoi_word spaces)* zoi_close spaces?
ZOI_post <- post_clause
ZOI_start <- !ZOI_pre ZOI
ZOhU_clause <- ZOhU_pre ZOhU_post
ZOhU_pre <- pre_clause ZOhU spaces?
ZOhU_post <- post_clause
CMEVLA <- cmevla
BRIVLA <- gismu / lujvo / fuhivla
CMAVO <- A / BAI / BAhE / BE / BEI / BEhO / BIhE / BIhI / BO / BOI / BU / BY / CAhA / CAI / CEI / CEhE / CO / COI / CU / CUhE / DAhO / DOI / DOhU / FA / FAhA / FAhO / FEhE / FEhU / FIhO / FOI / FUhA / FUhE / FUhO / GA / GAhO / GEhU / GI / GIhA / GOI / GOhA / GUhA / I / JA / JAI / JOhI / JOI / KE / KEhE / KEI / KI / KOhA / KU / KUhE / KUhO / LA / LAU / LAhE / LE / LEhU / LI / LIhU / LOhO / LOhU / LU / LUhU / MAhO / MAI / ME / MEhU / MOhE / MOhI / MOI / NA / NAI / NAhE / NAhU / NIhE / NIhO / NOI / NU / NUhA / NUhI / NUhU / PA / PEhE / PEhO / PU / RAhO / ROI / SA / SE / SEI / SEhU / SI / SOI / SU / TAhE / TEhU / TEI / TO / TOI / TUhE / TUhU / UI / VA / VAU / VEI / VEhO / VUhU / VEhA / VIhA / VUhO / XI / ZAhO / ZEhA / ZEI / ZI / ZIhE / ZO / ZOI / ZOhU / cmavo
lojban_word <- CMEVLA / CMAVO / BRIVLA
any_word <- lojban_word spaces?
zoi_open <- lojban_word
zoi_word_2 <- non_space+
zoi_word <- zoi_word_2
zoi_close <- any_word
cmevla <- jbocme / zifcme
zifcme <- !h (nucleus / glide / h / consonant !pause / digit)* consonant &pause
jbocme <- &zifcme (any_syllable / digit)+ &pause
cmavo <- !cmevla !CVCy_lujvo cmavo_form &post_word
CVCy_lujvo <- CVC_rafsi y h? initial_rafsi* brivla_core / stressed_CVC_rafsi y short_final_rafsi
cmavo_form <- !h !cluster onset (nucleus h)* (!stressed nucleus / nucleus !cluster) / y+ / digit
brivla <- !cmavo initial_rafsi* brivla_core
brivla_core <- fuhivla / gismu / CVV_final_rafsi / stressed_initial_rafsi short_final_rafsi
stressed_initial_rafsi <- stressed_extended_rafsi / stressed_y_rafsi / stressed_y_less_rafsi
initial_rafsi <- extended_rafsi / y_rafsi / !any_extended_rafsi y_less_rafsi !any_extended_rafsi
any_extended_rafsi <- fuhivla / extended_rafsi / stressed_extended_rafsi
fuhivla <- fuhivla_head stressed_syllable consonantal_syllable* final_syllable
stressed_extended_rafsi <- stressed_brivla_rafsi / stressed_fuhivla_rafsi
extended_rafsi <- brivla_rafsi / fuhivla_rafsi
stressed_brivla_rafsi <- &unstressed_syllable brivla_head stressed_syllable h y
brivla_rafsi <- &(syllable consonantal_syllable* syllable) brivla_head h y h?
stressed_fuhivla_rafsi <- fuhivla_head stressed_syllable consonantal_syllable* !h onset y
fuhivla_rafsi <- &unstressed_syllable fuhivla_head !h onset y h?
fuhivla_head <- !rafsi_string brivla_head
brivla_head <- !cmavo !slinkuhi !h &onset unstressed_syllable*
slinkuhi <- !rafsi_string consonant rafsi_string
rafsi_string <- y_less_rafsi* (gismu / CVV_final_rafsi / stressed_y_less_rafsi short_final_rafsi / y_rafsi / stressed_y_rafsi / stressed_y_less_rafsi? initial_pair y / hy_rafsi / stressed_hy_rafsi)
gismu <- (initial_pair stressed_vowel / consonant stressed_vowel consonant) &final_syllable consonant vowel &post_word
CVV_final_rafsi <- consonant stressed_vowel h &final_syllable vowel &post_word
short_final_rafsi <- &final_syllable (consonant diphthong / initial_pair vowel) &post_word
stressed_y_rafsi <- (stressed_long_rafsi / stressed_CVC_rafsi) y
stressed_y_less_rafsi <- stressed_CVC_rafsi !y / stressed_CCV_rafsi / stressed_CVV_rafsi
stressed_long_rafsi <- initial_pair stressed_vowel consonant / consonant stressed_vowel consonant consonant
stressed_CVC_rafsi <- consonant stressed_vowel consonant
stressed_CCV_rafsi <- initial_pair stressed_vowel
stressed_CVV_rafsi <- consonant (unstressed_vowel h stressed_vowel / stressed_diphthong) r_hyphen?
y_rafsi <- (long_rafsi / CVC_rafsi) y h?
y_less_rafsi <- !y_rafsi !stressed_y_rafsi !hy_rafsi !stressed_hy_rafsi (CVC_rafsi / CCV_rafsi / CVV_rafsi) !h
hy_rafsi <- (long_rafsi vowel / CCV_rafsi / CVV_rafsi) h y h?
stressed_hy_rafsi <- (long_rafsi stressed_vowel / stressed_CCV_rafsi / stressed_CVV_rafsi) h y
long_rafsi <- initial_pair unstressed_vowel consonant / consonant unstressed_vowel consonant consonant
CVC_rafsi <- consonant unstressed_vowel consonant
CCV_rafsi <- initial_pair unstressed_vowel
CVV_rafsi <- consonant (unstressed_vowel h unstressed_vowel / unstressed_diphthong) r_hyphen?
r_hyphen <- r &consonant / n &r
final_syllable <- onset !y !stressed nucleus !cmevla &post_word
stressed_syllable <- &stressed syllable / syllable &stress
stressed_diphthong <- &stressed diphthong / diphthong &stress
stressed_vowel <- &stressed vowel / vowel &stress
unstressed_syllable <- !stressed syllable !stress / consonantal_syllable
unstressed_diphthong <- !stressed diphthong !stress
unstressed_vowel <- !stressed vowel !stress
stress <- (consonant / glide)* h? y? syllable pause
stressed <- onset comma* [AEIOU]
any_syllable <- onset nucleus coda? / consonantal_syllable
syllable <- onset !y nucleus coda?
consonantal_syllable <- consonant &syllabic coda
coda <- !any_syllable consonant &any_syllable / syllabic? consonant? &pause
onset <- h / glide / initial
nucleus <- vowel / diphthong / y !nucleus
glide <- (i / u) &nucleus
diphthong <- (a i !i / a u !u / e i !i / o i !i) !nucleus
vowel <- (a / e / i / o / u) !nucleus
a <- comma* [aA]
e <- comma* [eE]
i <- comma* [iI]
o <- comma* [oO]
u <- comma* [uU]
y <- comma* [yY] !(!y nucleus)
cluster <- consonant consonant+
initial_pair <- &initial consonant consonant !consonant
initial <- (affricate / sibilant? other? liquid?) !consonant !glide
affricate <- t c / t s / d j / d z
liquid <- l / r
other <- p / t !l / k / f / x / b / d !l / g / v / m / n !liquid
sibilant <- c / s !x / (j / z) !n !liquid
consonant <- voiced / unvoiced / syllabic
syllabic <- l / m / n / r
voiced <- b / d / g / j / v / z
unvoiced <- c / f / k / p / s / t / x
l <- comma* [lL] !h !glide !l
m <- comma* [mM] !h !glide !m !z
n <- comma* [nN] !h !glide !n !affricate
r <- comma* [rR] !h !glide !r
b <- comma* [bB] !h !glide !b !unvoiced
d <- comma* [dD] !h !glide !d !unvoiced
g <- comma* [gG] !h !glide !g !unvoiced
v <- comma* [vV] !h !glide !v !unvoiced
j <- comma* [jJ] !h !glide !j !z !unvoiced
z <- comma* [zZ] !h !glide !z !j !unvoiced
s <- comma* [sS] !h !glide !s !c !voiced
c <- comma* [cC] !h !glide !c !s !x !voiced
x <- comma* [xX] !h !glide !x !c !k !voiced
k <- comma* [kK] !h !glide !k !x !voiced
f <- comma* [fF] !h !glide !f !voiced
p <- comma* [pP] !h !glide !p !voiced
t <- comma* [tT] !h !glide !t !voiced
h <- comma* ['h] &nucleus
digit <- comma* [0123456789] !h !nucleus
post_word <- pause / !nucleus lojban_word
pause <- comma* space_char+ / EOF
EOF <- comma* !.
comma <- [,]
non_lojban_word <- !lojban_word non_space+
non_space <- !space_char .
space_char <- [.\t\n\r?! ]
spaces <- !Y initial_spaces
initial_spaces <- (comma* space_char / !ybu Y)+ EOF? / EOF
ybu <- Y space_char* BU
lujvo <- !gismu !fuhivla brivla
A <- &cmavo ( a / e / j i / o / u ) &post_word
BAI <- &cmavo ( d u h o / s i h u / z a u / k i h i / d u h i / c u h u / t u h i / t i h u / d i h o / j i h u / r i h a / n i h i / m u h i / k i h u / v a h u / k o i / c a h i / t a h i / p u h e / j a h i / k a i / b a i / f i h e / d e h i / c i h o / m a u / m u h u / r i h i / r a h i / k a h a / p a h u / p a h a / l e h a / k u h u / t a i / b a u / m a h i / c i h e / f a u / p o h i / c a u / m a h e / c i h u / r a h a / p u h a / l i h e / l a h u / b a h i / k a h i / s a u / f a h e / b e h i / t i h i / j a h e / g a h a / v a h o / j i h o / m e h a / d o h e / j i h e / p i h o / g a u / z u h e / m e h e / r a i ) &post_word
BAhE <- &cmavo ( b a h e / z a h e ) &post_word
BE <- &cmavo ( b e ) &post_word
BEI <- &cmavo ( b e i ) &post_word
BEhO <- &cmavo ( b e h o ) &post_word
BIhE <- &cmavo ( b i h e ) &post_word
BIhI <- &cmavo ( m i h i / b i h o / b i h i ) &post_word
BO <- &cmavo ( b o ) &post_word
BOI <- &cmavo ( b o i ) &post_word
BU <- &cmavo ( b u ) &post_word
BY <- &cmavo ( ybu / j o h o / r u h o / g e h o / j e h o / l o h a / n a h a / s e h e / t o h a / g a h e / y h y / b y / c y / d y / f y / g y / j y / k y / l y / m y / n y / p y / r y / s y / t y / v y / x y / z y ) &post_word
CAhA <- &cmavo ( c a h a / p u h i / n u h o / k a h e ) &post_word
CAI <- &cmavo ( p e i / c a i / c u h i / s a i / r u h e ) &post_word
CEI <- &cmavo ( c e i ) &post_word
CEhE <- &cmavo ( c e h e ) &post_word
CO <- &cmavo ( c o ) &post_word
COI <- &cmavo ( j u h i / c o i / f i h i / t a h a / m u h o / f e h o / c o h o / p e h u / k e h o / n u h e / r e h i / b e h e / j e h e / m i h e / k i h e / v i h o ) &post_word
CU <- &cmavo ( c u ) &post_word
CUhE <- &cmavo ( c u h e / n a u ) &post_word
DAhO <- &cmavo ( d a h o ) &post_word
DOI <- &cmavo ( d o i ) &post_word
DOhU <- &cmavo ( d o h u ) &post_word
FA <- &cmavo ( f a i / f a / f e / f o / f u / f i h a / f i ) &post_word
FAhA <- &cmavo ( d u h a / b e h a / n e h u / v u h a / g a h u / t i h a / n i h a / c a h u / z u h a / r i h u / r u h u / r e h o / t e h e / b u h u / n e h a / p a h o / n e h i / t o h o / z o h i / z e h o / z o h a / f a h a ) &post_word &post_word
FAhO <- &cmavo ( f a h o ) &post_word
FEhE <- &cmavo ( f e h e ) &post_word
FEhU <- &cmavo ( f e h u ) &post_word
FIhO <- &cmavo ( f i h o ) &post_word
FOI <- &cmavo ( f o i ) &post_word
FUhA <- &cmavo ( f u h a ) &post_word
FUhE <- &cmavo ( f u h e ) &post_word
FUhO <- &cmavo ( f u h o ) &post_word
GA <- &cmavo ( g e h i / g e / g o / g a / g u ) &post_word
GAhO <- &cmavo ( k e h i / g a h o ) &post_word
GEhU <- &cmavo ( g e h u ) &post_word
GI <- &cmavo ( g i ) &post_word
GIhA <- &cmavo ( g i h e / g i h i / g i h o / g i h a / g i h u ) &post_word
GOI <- &cmavo ( n o h u / n e / g o i / p o h u / p e / p o h e / p o ) &post_word
GOhA <- &cmavo ( m o / n e i / g o h u / g o h o / g o h i / n o h a / g o h e / g o h a / d u / b u h a / b u h e / b u h i / c o h e ) &post_word
GUhA <- &cmavo ( g u h e / g u h i / g u h o / g u h a / g u h u ) &post_word
I <- &cmavo ( i ) &post_word
JA <- &cmavo ( j e h i / j e / j o / j a / j u ) &post_word
JAI <- &cmavo ( j a i ) &post_word
JOhI <- &cmavo ( j o h i ) &post_word
JOI <- &cmavo ( f a h u / p i h u / j o i / c e h o / c e / j o h u / k u h a / j o h e / j u h e ) &post_word
KE <- &cmavo ( k e ) &post_word
KEhE <- &cmavo ( k e h e ) &post_word
KEI <- &cmavo ( k e i ) &post_word
KI <- &cmavo ( k i ) &post_word
KOhA <- &cmavo ( d a h u / d a h e / d i h u / d i h e / d e h u / d e h e / d e i / d o h i / m i h o / m a h a / m i h a / d o h o / k o h a / f o h u / k o h e / k o h i / k o h o / k o h u / f o h a / f o h e / f o h i / f o h o / v o h a / v o h e / v o h i / v o h o / v o h u / r u / r i / r a / t a / t u / t i / z i h o / k e h a / m a / z u h i / z o h e / c e h u / d a / d e / d i / k o / m i / d o ) &post_word
KU <- &cmavo ( k u ) &post_word
KUhE <- &cmavo ( k u h e ) &post_word
KUhO <- &cmavo ( k u h o ) &post_word
LA <- &cmavo ( l a i / l a h i / l a ) &post_word
LAU <- &cmavo ( c e h a / l a u / z a i / t a u ) &post_word
LAhE <- &cmavo ( t u h a / l u h a / l u h o / l a h e / v u h i / l u h i / l u h e ) &post_word
LE <- &cmavo ( l e i / l o i / l e h i / l o h i / l e h e / l o h e / l o / l e ) &post_word
LEhU <- &cmavo ( l e h u ) &post_word
LI <- &cmavo ( m e h o / l i ) &post_word
LIhU <- &cmavo ( l i h u ) &post_word
LOhO <- &cmavo ( l o h o ) &post_word
LOhU <- &cmavo ( l o h u ) &post_word
LU <- &cmavo ( l u ) &post_word
LUhU <- &cmavo ( l u h u ) &post_word
MAhO <- &cmavo ( m a h o ) &post_word
MAI <- &cmavo ( m o h o / m a i ) &post_word
ME <- &cmavo ( m e ) &post_word
MEhU <- &cmavo ( m e h u ) &post_word
MOhE <- &cmavo ( m o h e ) &post_word
MOhI <- &cmavo ( m o h i ) &post_word
MOI <- &cmavo ( m e i / m o i / s i h e / c u h o / v a h e ) &post_word
NA <- &cmavo ( j a h a / n a ) &post_word
NAI <- &cmavo ( n a i ) &post_word
NAhE <- &cmavo ( t o h e / j e h a / n a h e / n o h e ) &post_word
NAhU <- &cmavo ( n a h u ) &post_word
NIhE <- &cmavo ( n i h e ) &post_word
NIhO <- &cmavo ( n i h o / n o h i ) &post_word
NOI <- &cmavo ( v o i / n o i / p o i ) &post_word
NU <- &cmavo ( n i / d u h u / s i h o / n u / l i h i / k a / j e i / s u h u / z u h o / m u h e / p u h u / z a h i ) &post_word
NUhA <- &cmavo ( n u h a ) &post_word
NUhI <- &cmavo ( n u h i ) &post_word
NUhU <- &cmavo ( n u h u ) &post_word
PA <- &cmavo ( d a u / f e i / g a i / j a u / r e i / v a i / p i h e / p i / f i h u / z a h u / m e h i / n i h u / k i h o / c e h i / m a h u / r a h e / d a h a / s o h a / j i h i / s u h o / s u h e / r o / r a u / s o h u / s o h i / s o h e / s o h o / m o h a / d u h e / t e h o / k a h o / c i h i / t u h o / x o / p a i / n o h o / n o / p a / r e / c i / v o / m u / x a / z e / b i / s o / digit ) &post_word
PEhE <- &cmavo ( p e h e ) &post_word
PEhO <- &cmavo ( p e h o ) &post_word
PU <- &cmavo ( b a / p u / c a ) &post_word
RAhO <- &cmavo ( r a h o ) &post_word
ROI <- &cmavo ( r e h u / r o i ) &post_word
SA <- &cmavo ( s a ) &post_word
SE <- &cmavo ( s e / t e / v e / x e ) &post_word
SEI <- &cmavo ( s e i / t i h o ) &post_word
SEhU <- &cmavo ( s e h u ) &post_word
SI <- &cmavo ( s i ) &post_word
SOI <- &cmavo ( s o i ) &post_word
SU <- &cmavo ( s u ) &post_word
TAhE <- &cmavo ( r u h i / t a h e / d i h i / n a h o ) &post_word
TEhU <- &cmavo ( t e h u ) &post_word
TEI <- &cmavo ( t e i ) &post_word
TO <- &cmavo ( t o h i / t o ) &post_word
TOI <- &cmavo ( t o i ) &post_word
TUhE <- &cmavo ( t u h e ) &post_word
TUhU <- &cmavo ( t u h u ) &post_word
UI <- &cmavo ( i h a / i e / a h e / u h i / i h o / i h e / a h a / i a / o h i / o h e / e h e / o i / u o / e h i / u h o / a u / u a / a h i / i h u / i i / u h a / u i / a h o / a i / a h u / i u / e i / o h o / e h a / u u / o h a / o h u / u h u / e h o / i o / e h u / u e / i h i / u h e / b a h a / j a h o / c a h e / s u h a / t i h e / k a h u / s e h o / z a h a / p e h i / r u h a / j u h a / t a h o / r a h u / l i h a / b a h u / m u h a / d o h a / t o h u / v a h i / p a h e / z u h u / s a h e / l a h a / k e h u / s a h u / d a h i / j e h u / s a h a / k a u / t a h u / n a h i / j o h a / b i h u / l i h o / p a u / m i h u / k u h i / j i h a / s i h a / p o h o / p e h a / r o h i / r o h e / r o h o / r o h u / r o h a / r e h e / l e h o / j u h o / f u h i / d a i / g a h i / z o h o / b e h u / r i h e / s e h i / s e h a / v u h e / k i h a / x u / g e h e / b u h o ) &post_word
VA <- &cmavo ( v i / v a / v u ) &post_word
VAU <- &cmavo ( v a u ) &post_word
VEI <- &cmavo ( v e i ) &post_word
VEhO <- &cmavo ( v e h o ) &post_word
VUhU <- &cmavo ( g e h a / f u h u / p i h i / f e h i / v u h u / s u h i / j u h u / g e i / p a h i / f a h i / t e h a / c u h a / v a h a / n e h o / d e h o / f e h a / s a h o / r e h a / r i h o / s a h i / p i h a / s i h i ) &post_word
VEhA <- &cmavo ( v e h u / v e h a / v e h i / v e h e ) &post_word
VIhA <- &cmavo ( v i h i / v i h a / v i h u / v i h e ) &post_word
VUhO <- &cmavo ( v u h o ) &post_word
XI <- &cmavo ( x i ) &post_word
Y <- &cmavo ( y+ ) &post_word
ZAhO <- &cmavo ( c o h i / p u h o / c o h u / m o h u / c a h o / c o h a / d e h a / b a h o / d i h a / z a h o ) &post_word
ZEhA <- &cmavo ( z e h u / z e h a / z e h i / z e h e ) &post_word
ZEI <- &cmavo ( z e i ) &post_word
ZI <- &cmavo ( z u / z a / z i ) &post_word
ZIhE <- &cmavo ( z i h e ) &post_word
ZO <- &cmavo ( z o ) &post_word
ZOI <- &cmavo ( z o i / l a h o ) &post_word
ZOhU <- &cmavo ( z o h u ) &post_word
"##,
);
