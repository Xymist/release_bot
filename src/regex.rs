use color_eyre::{Report, Result};
use regex::{Regex, RegexBuilder};
use tokio::sync::OnceCell;

static OLD_CLIENT_REGEXP: OnceCell<Regex> = OnceCell::const_new();
static NEW_CLIENT_REGEXP: OnceCell<Regex> = OnceCell::const_new();
static FEATURE_REGEXP: OnceCell<Regex> = OnceCell::const_new();
static MODULE_REGEXP: OnceCell<Regex> = OnceCell::const_new();

pub async fn client_details(haystack: &str) -> Option<String> {
    let client_regexp = old_client_regexp().await.ok()?;
    let old_captures = client_regexp.captures(haystack);
    let results = if old_captures.is_none() {
        let client_regexp = new_client_regexp().await.ok()?;
        client_regexp.captures(haystack).and_then(|c| c.get(1))
    } else {
        old_captures.and_then(|c| c.get(2))
    };

    results
        .map(|m| m.as_str())
        .map(|m| {
            m.replace('\n', ", ")
                .replace('&', "\\&")
                .replace('#', "\\#")
        })
        .filter(|m| *m != "_No response_" && !m.trim().is_empty())
}

pub async fn old_client_regexp() -> Result<&'static Regex> {
    OLD_CLIENT_REGEXP
        .get_or_try_init(|| async {
            Ok::<Regex, Report>(
                RegexBuilder::new(
                    r"### Have any clients (encountered|requested) this\?\n+(.*?)\n*(###|$)",
                )
                .dot_matches_new_line(true)
                .build()
                .unwrap(),
            )
        })
        .await
}

pub async fn new_client_regexp() -> Result<&'static Regex> {
    NEW_CLIENT_REGEXP
        .get_or_try_init(|| async {
            Ok::<Regex, Report>(
                RegexBuilder::new(
                    r"### List the company names of any affected clients\n+(.*?)\n*(###|$)",
                )
                .dot_matches_new_line(true)
                .build()
                .unwrap(),
            )
        })
        .await
}

pub async fn feature_regexp() -> Result<&'static Regex> {
    FEATURE_REGEXP
        .get_or_try_init(|| async {
            Ok::<Regex, Report>(Regex::new(r"(\[Feature\]|\[Epic\]|\[Request\]):")?)
        })
        .await
}

pub async fn module_details(haystack: &str) -> Option<Vec<&str>> {
    Some(
        module_regexp()
            .await
            .ok()?
            .captures(haystack)
            .and_then(|c| {
                c.get(2)
                    .map(|m| m.as_str())
                    .filter(|m| *m != "_No response_" && !m.trim().is_empty())
            })
            .unwrap_or("Unsure/Other")
            .split(", ")
            .collect(),
    )
}

pub async fn module_regexp() -> Result<&'static Regex> {
    MODULE_REGEXP
        .get_or_try_init(|| async {
            Ok::<Regex, Report>(Regex::new(
                r"### Which module\(s\) (is this bug related to|would developing this feature affect|would this epic affect|would making this change affect)\?\n+(.*?)\n*(###|$)",
            )?)
        })
        .await
}
