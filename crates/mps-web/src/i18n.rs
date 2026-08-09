// i18n module — bridges dioxus-i18n Fluent infrastructure.
// The actual translations live in i18n/locales/*.ftl (Fluent format).
// The I18nConfig is initialized in lib.rs::app() via use_init_i18n.

pub use dioxus_i18n::prelude::*;
pub use dioxus_i18n::{t, te, tid};

/// Supported language identifiers.
pub mod langs {
    use unic_langid::LanguageIdentifier;
    pub const ZH_CN: LanguageIdentifier = unic_langid::langid!("zh-CN");
    pub const EN: LanguageIdentifier = unic_langid::langid!("en");

    /// All supported languages in drop-down order.
    pub const ALL: &[(&LanguageIdentifier, &str)] = &[(&ZH_CN, "中文"), (&EN, "English")];

    /// Parse a language string ("zh", "zh-CN", "en", "en-US") to a LanguageIdentifier.
    pub fn parse(s: &str) -> Option<LanguageIdentifier> {
        match s {
            "zh" | "zh-CN" | "zh-cn" => Some(ZH_CN),
            "en" | "en-US" | "en-us" => Some(EN),
            _ => LanguageIdentifier::from_bytes(s.as_bytes()).ok(),
        }
    }
}
