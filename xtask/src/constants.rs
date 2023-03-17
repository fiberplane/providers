use console::Emoji;

pub(crate) static BUILD: Emoji<'_, '_> = Emoji("🔨 ", "");
pub(crate) static CHECK: Emoji<'_, '_> = Emoji("✅ ", "");
pub(crate) static ERROR: Emoji<'_, '_> = Emoji("🤒 ", "");
pub(crate) static NOTE: Emoji<'_, '_> = Emoji("📝 ", "");
pub(crate) static OPTIMIZE: Emoji<'_, '_> = Emoji("📦 ", "");
pub(crate) static SUCCESS: Emoji<'_, '_> = Emoji("✨ ", "");
pub(crate) static WARN: Emoji<'_, '_> = Emoji("⚠️ ", "");
pub(crate) static WORKING: Emoji<'_, '_> = Emoji("🔧 ", "");

pub(crate) const PROVIDERS: &[&str] = &[
    "cloudwatch",
    "elasticsearch",
    "https",
    "loki",
    "prometheus",
    "sample",
    "sentry",
];
