use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Dict(HashMap<String, String>);

fn load_dict(name: &str) -> Dict {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(manifest_dir).join("src/i18n").join(name);
    let s = fs::read_to_string(path).unwrap_or_else(|_| panic!("missing i18n file: {}", name));
    serde_json::from_str(&s).unwrap_or_else(|_| panic!("invalid json in: {}", name))
}

static ZH: Lazy<Dict> = Lazy::new(|| load_dict("zh.json"));
static EN: Lazy<Dict> = Lazy::new(|| load_dict("en.json"));

/// Return translation for `key` in given `lang` ("zh" or "en").
/// Falls back to key wrapped in brackets if missing.
pub fn t(key: &str, lang: &str) -> String {
    let dict = if lang == "en" { &EN.0 } else { &ZH.0 };
    dict.get(key).cloned().unwrap_or_else(|| format!("[missing:{}]", key))
}

/// Generate `<script id="i18n-dict" type="application/json">` with full dictionary for given lang.
pub fn i18n_dict_script(lang: &str) -> topcoat::view::Unescaped<String> {
    let dict = if lang == "en" { &EN.0 } else { &ZH.0 };
    let json = serde_json::to_string(dict).unwrap();
    topcoat::view::Unescaped::new_unchecked(format!(
        "<script id=\"i18n-dict\" type=\"application/json\">{}</script>",
        json
    ))
}