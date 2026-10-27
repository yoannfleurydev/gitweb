use git2::Config;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

const SECTION: &str = "gitweb.";

// Everything but the unreserved characters of the RFC 3986, so a value can be
// used anywhere in a url, including a query parameter.
const ENCODED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

// Percent-encode the value, slashes included.
pub fn encode(value: &str) -> String {
    utf8_percent_encode(value, ENCODED).to_string()
}

// Replace each {name} of the template by the value of its placeholder, or
// fail with the name of the first placeholder that is not available.
pub fn render(template: &str, placeholders: &[(&str, String)]) -> Result<String, String> {
    let mut url = String::new();
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        url.push_str(&rest[..start]);
        rest = &rest[start + 1..];

        // {{ is the way to write a { that is not a placeholder
        if let Some(escaped) = rest.strip_prefix('{') {
            url.push('{');
            rest = escaped;
            continue;
        }

        let end = rest.find('}').ok_or_else(|| rest.to_string())?;
        let name = &rest[..end];

        let (_, value) = placeholders
            .iter()
            .find(|(placeholder, _)| *placeholder == name)
            .ok_or_else(|| name.to_string())?;

        url.push_str(value);
        rest = &rest[end + 1..];
    }

    url.push_str(rest);

    Ok(url)
}

// Find the template configured as gitweb.<host>.<key>, the host being matched
// without taking care of the case. As in git, the last value found wins.
pub fn find(config: &Config, host: &str, key: &str) -> Option<String> {
    let suffix = format!(".{key}");
    let mut template = None;

    let mut entries = config
        .entries(Some(&format!("^gitweb\\..*\\.{key}$")))
        .ok()?;

    while let Some(Ok(entry)) = entries.next() {
        let (Ok(name), Ok(value)) = (entry.name(), entry.value()) else {
            continue;
        };

        let entry_host = name
            .strip_prefix(SECTION)
            .and_then(|name| name.strip_suffix(&suffix));

        if entry_host.is_some_and(|entry_host| entry_host.eq_ignore_ascii_case(host)) {
            template = Some(value.to_string());
        }
    }

    template
}
