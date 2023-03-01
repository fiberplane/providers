use console::Emoji;

pub(crate) static BUILD: Emoji<'_, '_> = Emoji("🔨 ", "");
pub(crate) static OPTIMIZE: Emoji<'_, '_> = Emoji("📦 ", "");
pub(crate) static SUCCESS: Emoji<'_, '_> = Emoji("✨ ", "");

pub(crate) const PROVIDERS: &[&str] = &[
    "cloudwatch",
    "elasticsearch",
    "https",
    "loki",
    "prometheus",
    "sample",
    "sentry",
];
