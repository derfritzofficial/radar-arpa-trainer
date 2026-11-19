//! Zweisprachigkeit Deutsch / Englisch.
//!
//! `t("Deutsch", "English")` liefert den Text der aktiven Sprache,
//! `tf!("Deutsch {}", "English {}", x)` ist die formatierende Variante.
//! Beim Start wird die Systemsprache übernommen (Deutsch → Deutsch, sonst Englisch).

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    De,
    En,
}

impl Lang {
    pub fn label(self) -> &'static str {
        match self {
            Lang::De => "Deutsch",
            Lang::En => "English",
        }
    }
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(l: Lang) {
    LANG.store(if l == Lang::De { 0 } else { 1 }, Ordering::Relaxed);
}

pub fn lang() -> Lang {
    if LANG.load(Ordering::Relaxed) == 0 {
        Lang::De
    } else {
        Lang::En
    }
}

pub fn is_de() -> bool {
    lang() == Lang::De
}

/// Systemsprache ermitteln.
pub fn detect() -> Lang {
    match sys_locale::get_locale() {
        Some(l) if l.to_lowercase().starts_with("de") => Lang::De,
        _ => Lang::En,
    }
}

/// Text in der aktiven Sprache.
pub fn t(de: &'static str, en: &'static str) -> &'static str {
    if is_de() {
        de
    } else {
        en
    }
}

/// Formatierter Text in der aktiven Sprache.
macro_rules! tf {
    ($de:literal, $en:literal $(, $a:expr)* $(,)?) => {
        if $crate::i18n::is_de() {
            format!($de $(, $a)*)
        } else {
            format!($en $(, $a)*)
        }
    };
}
