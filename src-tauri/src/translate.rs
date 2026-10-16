//! Translating the current page through Google Translate's web proxy.
//!
//! `https://translate.google.com/translate?…&u=<page>` answers with a redirect to
//! `<host>.translate.goog`, which serves the page translated. `hl` is passed too because
//! otherwise Google picks the proxy's interface language from the requester's IP address.

use url::Url;

/// Languages offered in settings: (code, English name). Any valid code works in the URL.
pub const LANGUAGES: &[(&str, &str)] = &[
    ("ar", "Arabic"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("de", "German"),
    ("el", "Greek"),
    ("en", "English"),
    ("es", "Spanish"),
    ("fa", "Persian"),
    ("fi", "Finnish"),
    ("fr", "French"),
    ("he", "Hebrew"),
    ("hi", "Hindi"),
    ("hu", "Hungarian"),
    ("id", "Indonesian"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("nl", "Dutch"),
    ("no", "Norwegian"),
    ("pl", "Polish"),
    ("pt", "Portuguese"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("sv", "Swedish"),
    ("th", "Thai"),
    ("tr", "Turkish"),
    ("uk", "Ukrainian"),
    ("vi", "Vietnamese"),
    ("zh-CN", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
];

/// A language code of the form `ll`, `lll`, `ll-RR`, or `ll-Script` (`^[a-z]{2,3}(-[A-Za-z]{2,4})?$`).
pub fn is_valid_lang(lang: &str) -> bool {
    let (primary, region) = match lang.split_once('-') {
        Some((primary, region)) => (primary, Some(region)),
        None => (lang, None),
    };
    let primary_ok =
        (2..=3).contains(&primary.len()) && primary.chars().all(|c| c.is_ascii_lowercase());
    let region_ok = region
        .is_none_or(|r| (2..=4).contains(&r.len()) && r.chars().all(|c| c.is_ascii_alphabetic()));
    primary_ok && region_ok
}

/// The Google Translate proxy URL that shows `page` translated into `lang`, or `None` when the
/// page can't or needn't be translated: not a web page, already a translation, Google Translate
/// itself, or an invalid language code.
pub fn translate_url(page: &Url, lang: &str) -> Option<Url> {
    if !matches!(page.scheme(), "http" | "https") || !is_valid_lang(lang) {
        return None;
    }
    let host = page.host_str()?;
    if host == "translate.goog"
        || host.ends_with(".translate.goog")
        || host == "translate.google.com"
    {
        return None;
    }
    Url::parse_with_params(
        "https://translate.google.com/translate",
        &[
            ("sl", "auto"),
            ("tl", lang),
            ("hl", lang),
            ("u", page.as_str()),
        ],
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("test URL parses")
    }

    #[test]
    fn builds_the_proxy_url_with_the_page_encoded() {
        let got = translate_url(&url("https://example.com/a?b=1"), "tr").expect("translatable");
        assert_eq!(
            got.as_str(),
            "https://translate.google.com/translate?sl=auto&tl=tr&hl=tr&u=https%3A%2F%2Fexample.com%2Fa%3Fb%3D1"
        );
    }

    #[test]
    fn skips_pages_that_cannot_or_need_not_be_translated() {
        assert_eq!(
            translate_url(&url("https://example-com.translate.goog/a"), "en"),
            None
        );
        assert_eq!(
            translate_url(&url("https://translate.google.com/?sl=auto"), "en"),
            None
        );
        assert_eq!(
            translate_url(&url("tauri://localhost/index.html"), "en"),
            None
        );
        assert_eq!(translate_url(&url("about:blank"), "en"), None);
        assert_eq!(translate_url(&url("https://example.com/"), "english"), None);
    }

    #[test]
    fn language_codes() {
        for ok in ["en", "tr", "fil", "zh-CN", "zh-TW", "sr-Latn"] {
            assert!(is_valid_lang(ok), "{ok}");
        }
        for bad in [
            "", "e", "EN", "english", "zh-", "zh-CN-x", "z1", "en_US", "en-12",
        ] {
            assert!(!is_valid_lang(bad), "{bad}");
        }
    }

    #[test]
    fn every_offered_language_is_valid_and_unique() {
        let mut codes: Vec<&str> = LANGUAGES.iter().map(|(code, _)| *code).collect();
        assert!(codes.iter().all(|c| is_valid_lang(c)));
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), LANGUAGES.len());
        assert!(codes.contains(&"en"));
    }
}
