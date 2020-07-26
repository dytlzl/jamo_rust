extern crate jamo;

use jamo::hangul::{KoreanSentence};


fn main() {
    let sentence = KoreanSentence::new("원하시는 페이지를 찾을 수가 없습니다. 좋아요.");
    display(&sentence);
    let sentence = sentence.apply_rules();
    display(&sentence);
}

fn display(s: &KoreanSentence) {
    println!("[Roman]\n{}\n[Jamo]\n{}\n[Hangul]\n{}",
             s.roman(),
             s.jamo(),
             s.hangul_string(),
    );
}


