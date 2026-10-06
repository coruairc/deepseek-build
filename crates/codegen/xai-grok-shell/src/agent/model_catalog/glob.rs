//! Compiled glob matcher shared by `allowed_models`, `disabled_models`, and `hidden_models`.

use globset::{Glob, GlobSet, GlobSetBuilder};

/// Compiled glob matcher matched against catalog key or model id.
pub(crate) struct ModelGlobSet(GlobSet);

impl ModelGlobSet {
    /// Compile a filter list (`Ok(None)` for `None`/empty). Fails **closed**: an invalid pattern returns `Err` listing every bad one.
    pub(crate) fn compile(patterns: Option<&[String]>) -> Result<Option<Self>, Vec<String>> {
        let patterns = match patterns {
            Some(patterns) if !patterns.is_empty() => patterns,
            Some(_) | None => return Ok(None),
        };

        let mut builder = GlobSetBuilder::new();
        let mut invalid = Vec::new();
        for pattern in patterns {
            match Glob::new(pattern) {
                Ok(glob) => {
                    builder.add(glob);
                }
                Err(_) => invalid.push(pattern.clone()),
            }
        }

        if !invalid.is_empty() {
            return Err(invalid);
        }
        builder
            .build()
            .map(|set| Some(Self(set)))
            .map_err(|error| vec![error.to_string()])
    }

    pub(crate) fn matches(&self, key: &str, model: &str) -> bool {
        self.0.is_match(key) || self.0.is_match(model)
    }

    pub(crate) fn matches_model(&self, model: &str) -> bool {
        self.0.is_match(model)
    }
}
