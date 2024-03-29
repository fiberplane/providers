use inflector::Inflector;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Casing {
    #[default]
    Original,
    CamelCase,
    PascalCase,
    SnakeCase,
    ScreamingSnakeCase,
}

impl Casing {
    pub fn format_string(&self, string: &str) -> String {
        match self {
            Self::Original => string.to_owned(),
            Self::CamelCase => string.to_camel_case(),
            Self::PascalCase => string.to_pascal_case(),
            Self::SnakeCase => string.to_snake_case(),
            Self::ScreamingSnakeCase => string.to_screaming_snake_case(),
        }
    }
}

impl TryFrom<&str> for Casing {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "camelCase" => Ok(Self::CamelCase),
            "PascalCase" => Ok(Self::PascalCase),
            "snake_case" => Ok(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            other => Err(format!("Unrecognized case format: {}", other)),
        }
    }
}
