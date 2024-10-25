use camxes_rs::peg::grammar::PEG;

const CMAXES_GRAMMAR: (&str, &str) = (
    "text",
    "text <- any_word+

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
");

fn main() {
    use env_logger;
    env_logger::builder().init();

    let (start, grammar) = CMAXES_GRAMMAR;
    let p = PEG::new(start, grammar).unwrap();
    println!("{:#?}", p.parse("coi do"));
    println!("{}", p);
}