// MPS Web — single-page documentation (SSR-only, no router).
//
// The site is served fully server-side; there is no client WASM bundle
// (no `dx` CLI in this environment). To avoid the Dioxus `Link` click
// interceptor silently eating navigation under SSR-only, the whole doc is
// ONE inline page: a sticky in-page table-of-contents with plain
// `<a href="#sec-...">` anchors (native scroll, zero JS, no hydration trap).

#![allow(non_snake_case, unused)]

mod i18n;
mod layouts;
mod pages;

mod metrics;

use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use unic_langid::langid;

use pages::home::Home;

pub fn main() {
    dioxus::launch(Home);
}

// CI guard: every i18n key referenced through the `t!` macro anywhere in the
// site must exist in BOTH locale files. A missing key only explodes at SSR
// render time (which took down the Pages export with a bare 500), so it is
// asserted here instead.
#[cfg(test)]
mod i18n_coverage {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    fn collect_rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn every_t_key_exists_in_both_locales() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        collect_rs_files(&src, &mut files);

        let mut texts = Vec::new();
        let mut used: BTreeSet<String> = BTreeSet::new();
        for file in &files {
            texts.push(fs::read_to_string(file).unwrap());
            let text = texts.last().unwrap();
            let bytes = text.as_bytes();
            let mut i = 0;
            while let Some(pos) = text[i..].find("t!(\"") {
                // Skip `format!(` / `assert!(` etc. — only a bare `t!(` counts,
                // so the character before must not be an identifier char.
                let abs = i + pos;
                if abs > 0 {
                    let prev = text.as_bytes()[abs - 1];
                    if prev.is_ascii_alphanumeric() || prev == b'_' {
                        i = abs + 4;
                        continue;
                    }
                }
                let start = abs + 4;
                let Some(len) = bytes[start..].iter().position(|&b| b == b'"') else {
                    break;
                };
                used.insert(text[start..start + len].to_string());
                i = start + len;
            }
        }
        assert!(!used.is_empty(), "no t! keys found — extraction is broken");

        for locale in ["zh-CN", "en"] {
            let text = fs::read_to_string(src.join(format!("i18n/locales/{locale}.ftl"))).unwrap();
            let defined: BTreeSet<String> = text
                .lines()
                .filter_map(|l| l.split_once(" = ").map(|(id, _)| id.to_string()))
                .collect();
            let missing: Vec<String> = used.difference(&defined).map(|k| k.to_string()).collect();
            assert!(
                missing.is_empty(),
                "{locale}.ftl is missing keys used in pages: {missing:?}"
            );
        }
    }
}
