use color_eyre::{Report, Result};
use regex::{Regex, RegexBuilder};
use tokio::sync::OnceCell;

static CLIENT_REGEXP: OnceCell<Regex> = OnceCell::const_new();
static FEATURE_REGEXP: OnceCell<Regex> = OnceCell::const_new();
static MODULE_REGEXP: OnceCell<Regex> = OnceCell::const_new();

pub async fn client_regexp() -> Result<&'static Regex> {
    CLIENT_REGEXP
        .get_or_try_init(|| async {
            Ok::<Regex, Report>(
                RegexBuilder::new(
                    r"### Have any clients (encountered|requested) this\?\n+(.*?)\n+###",
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

pub async fn module_regexp() -> Result<&'static Regex> {
    MODULE_REGEXP
        .get_or_try_init(|| async {
            Ok::<Regex, Report>(Regex::new(
                r"### Which module\(s\) (is this bug related to|would developing this feature affect|would this epic affect|would making this change affect)\?\n+(.*?)\n*(###|$)",
            )?)
        })
        .await
}
