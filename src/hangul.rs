use core::char;
use std::collections::HashMap;
use std::iter::FromIterator;


const JAMO_OFFSET: usize = 0xac00;
const LEAD_OFFSET: usize = 0x1100;
const VOWEL_OFFSET: usize = 0x1161;
const TAIL_OFFSET: usize = 0x11a7;

const LEAD_DICT: [&str; 19] = [
    "g", "kk", "n", "d", "tt", "r", "m", "b", "pp", "s",
    "ss", "", "j", "tch", "ch", "k", "t", "p", "h", ];
const VOWEL_DICT: [&str; 21] = [
    "a", "ae", "ya", "yae", "eo", "e", "yeo", "ye", "o", "wa",
    "wae", "oe", "yo", "u", "weo", "we", "wi", "yu", "eu", "eui",
    "i", ];
const TAIL_DICT: [&str; 28] = [
    "", "g", "gg", "gs", "n", "nj", "nh", "d", "r", "rg",
    "rm", "rb", "rs", "rt", "rb", "rh", "m", "b", "bs", "s",
    "ss", "ng", "j", "ch", "k", "t", "p", "h", ];

fn reverse_dict(s: &[&'static str]) -> HashMap<&'static str, usize> {
    HashMap::from_iter(s.iter().enumerate().map(|(i, v)| (*v, i)))
}

#[derive(Clone)]
enum JamoPosition {
    Lead,
    Vowel,
    Tail,
}

#[derive(Clone)]
pub struct Jamo {
    usize: usize,
    position: JamoPosition,
}

impl Jamo {
    pub fn roman(&self) -> &'static str {
        return match self.position {
            JamoPosition::Lead => LEAD_DICT[self.usize],
            JamoPosition::Vowel => VOWEL_DICT[self.usize],
            JamoPosition::Tail => TAIL_DICT[self.usize],
        };
    }
    fn jamo_char_from_usize(u: usize, offset: usize) -> char {
        char::from_u32((u + offset) as u32).unwrap()
    }
    pub fn jamo_string(&self) -> String {
        return match self.position {
            JamoPosition::Lead => Self::jamo_char_from_usize(self.usize, LEAD_OFFSET).to_string(),
            JamoPosition::Vowel => Self::jamo_char_from_usize(self.usize, VOWEL_OFFSET).to_string(),
            JamoPosition::Tail => {
                if self.usize == 0 {
                    return String::new();
                }
                Self::jamo_char_from_usize(self.usize, TAIL_OFFSET).to_string()
            }
        };
    }
}

#[derive(Clone)]
pub struct Hangul {
    lead: Jamo,
    vowel: Jamo,
    tail: Jamo,
}


impl Hangul {
    pub fn new(c: char) -> Self {
        let rem = c as usize - JAMO_OFFSET;
        let lead = rem / 588;
        let vowel = rem % 588 / 28;
        let tail = rem % 28;
        Hangul {
            lead: Jamo { usize: lead, position: JamoPosition::Lead },
            vowel: Jamo { usize: vowel, position: JamoPosition::Vowel },
            tail: Jamo { usize: tail, position: JamoPosition::Tail },
        }
    }
    pub fn lead(&self) -> &Jamo {
        &self.lead
    }
    pub fn tail(&self) -> &Jamo {
        &self.tail
    }

    pub fn roman_string(&self) -> String {
        format!("{}{}{}", self.lead.roman(), self.vowel.roman(), self.tail.roman())
    }
    pub fn jamo_string(&self) -> String {
        format!("[{}][{}][{}]",
                self.lead.jamo_string(),
                self.vowel.jamo_string(),
                self.tail.jamo_string())
    }
    pub fn hangul_string(&self) -> String {
        format!("{}{}{}",
                self.lead.jamo_string(),
                self.vowel.jamo_string(),
                self.tail.jamo_string())
    }
}

#[derive(Clone)]
pub enum Letter {
    HangulLetter(Hangul),
    OtherLetter(char),
}

impl Letter {
    pub fn new(c: char) -> Letter {
        if JAMO_OFFSET <= (c as usize) && (c as usize) < 0xd74a {
            Letter::HangulLetter(Hangul::new(c))
        } else {
            Letter::OtherLetter(c)
        }
    }
    pub fn roman(&self) -> String {
        match self {
            Self::HangulLetter(l) => l.roman_string(),
            Self::OtherLetter(c) => c.to_string(),
        }
    }
    pub fn jamo(&self) -> String {
        match self {
            Self::HangulLetter(l) => l.jamo_string(),
            Self::OtherLetter(c) => c.to_string(),
        }
    }
    pub fn hangul_string(&self) -> String {
        match self {
            Self::HangulLetter(l) => l.hangul_string(),
            Self::OtherLetter(c) => c.to_string(),
        }
    }
    pub fn is_hangul(&self) -> bool {
        if let Self::HangulLetter(_) = self {
            return true;
        }
        false
    }
}

struct Rule {
    tail: &'static str,
    lead: &'static str,
    strategy: fn(/* old_tail */&'static str, /* old_lead */&'static str)
                 -> (/* new_tail */&'static str, /* new_lead */&'static str),
}

const RULES: [Rule; 5] = [ // under developing yet
    Rule {
        tail: "h",
        lead: "",
        strategy: |_, _| { ("", "") },
    },
    Rule { // 연음화
        tail: "*",
        lead: "",
        strategy: |t, _| { ("", t) },
    },
    Rule {
        tail: "b",
        lead: "n",
        strategy: |_, l| { ("m", l) },
    },
    Rule {
        tail: "n",
        lead: "h",
        strategy: |t, _| { ("", t) },
    },
    Rule {
        tail: "bs",
        lead: "*",
        strategy: |_, l| { if l == "" { ("p", "s") } else { ("p", l) } },
    },
];

#[derive(Clone)]
struct JamoContext {
    lead_rev_dict: HashMap<&'static str, usize>,
    vowel_rev_dict: HashMap<&'static str, usize>,
    tail_rev_dict: HashMap<&'static str, usize>,
}

pub struct KoreanSentence {
    payload: Vec<Letter>,
    context: JamoContext,
}

impl KoreanSentence {
    pub fn new(s: &str) -> Self {
        Self {
            payload: s.chars().map(
                |c| Letter::new(c)
            ).collect::<Vec<Letter>>(),
            context: JamoContext {
                lead_rev_dict: reverse_dict(&LEAD_DICT[..]),
                vowel_rev_dict: reverse_dict(&VOWEL_DICT[..]),
                tail_rev_dict: reverse_dict(&TAIL_DICT[..]),
            },
        }
    }

    pub fn roman(&self) -> String {
        self.payload.iter().map(|l| l.roman()).collect::<Vec<String>>().join("")
    }

    pub fn jamo(&self) -> String {
        self.payload.iter().map(|l| l.jamo()).collect::<Vec<String>>().join("")
    }

    pub fn hangul_string(&self) -> String {
        self.payload.iter().map(|l| l.hangul_string()).collect::<Vec<String>>().join("")
    }

    /// Returns a KoreanSentence applied the rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use jamo::hangul::KoreanSentence;
    /// let sentence = KoreanSentence::new("좋아요.");
    /// let new_sentence = sentence.applied();
    /// assert_eq!("조아요.", new_sentence.hangul_string());
    /// ```
    pub fn applied(&self) -> Self {
        Self { payload: self.applied_vec(self.payload[0].clone(), self.payload[1].clone(), &self.payload[2..]), context: self.context.clone() }
    }
    fn applied_vec(&self, a: Letter, b: Letter, rest: &[Letter]) -> Vec<Letter> {
        let (_a, _b) = self.apply_rules(a, b, &RULES[..]);
        if rest.len() == 0 {
            return vec![_a, _b]
        }
        [vec![_a], self.applied_vec(_b, rest[0].clone(), &rest[1..])].concat()
    }
    fn apply_rules(&self, a: Letter, b: Letter, rules: &[Rule]) -> (Letter, Letter) {
        if rules.len() == 0 {
            return (a, b)
        }
        if let Letter::HangulLetter(_a) = &a {
            if let Letter::HangulLetter(_b) = &b {
                let tail = _a.tail().roman();
                let lead = _b.lead().roman();
                if (rules[0].tail == "*" || tail == rules[0].tail) && (rules[0].lead == "*" || lead == rules[0].lead) {
                    let (new_tail, new_lead) = (rules[0].strategy)(tail, lead);
                    return self.apply_rules(
                        Letter::HangulLetter(
                            Hangul {
                                lead: _a.lead.clone(),
                                vowel: _a.vowel.clone(),
                                tail: Jamo { usize: self.context.tail_rev_dict[new_tail], position: JamoPosition::Tail } }),
                        Letter::HangulLetter(
                            Hangul {
                                lead: Jamo { usize: self.context.lead_rev_dict[new_lead], position: JamoPosition::Lead },
                                vowel: _b.vowel.clone(),
                                tail: _b.tail.clone() }),
                        &rules[1..]);
                }
            }
        }
        self.apply_rules(a, b, &rules[1..])
    }
}
