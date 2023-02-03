/// Sluggify a label, making sure it has only alphanumeric characters and underscores.
///
/// Uses the fallback function if the label would sluggify to an empty string.
pub fn sluggify(label: &str, fallback: impl Fn() -> String) -> String {
    static MAX_LEN: usize = 255;
    let candidate = label
        .to_ascii_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
    let candidate = candidate.trim_matches(|c: char| !c.is_ascii_lowercase());
    let candidate = &candidate[..MAX_LEN.min(candidate.len())];
    if !candidate.is_empty() {
        candidate.to_string()
    } else {
        fallback()
    }
}
