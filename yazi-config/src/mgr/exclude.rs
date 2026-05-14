use std::sync::Arc;

use globset::{GlobBuilder, GlobSetBuilder};
use serde::Deserialize;
use yazi_fs::Exclude;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(try_from = "Vec<String>")]
pub struct MgrExclude(Exclude);

impl MgrExclude {
	pub fn get(&self) -> Exclude { self.0.clone() }
}

impl TryFrom<Vec<String>> for MgrExclude {
	type Error = anyhow::Error;

	fn try_from(patterns: Vec<String>) -> Result<Self, Self::Error> {
		if patterns.is_empty() {
			return Ok(Self(Exclude::default()));
		}

		let mut names = GlobSetBuilder::new();
		let mut paths = GlobSetBuilder::new();

		for pattern in &patterns {
			if pattern.contains('/') {
				// Path pattern: auto-prefix with **/ if not already rooted
				let prefixed =
					if pattern.starts_with('/') || pattern.starts_with("**/") {
						pattern.clone()
					} else {
						format!("**/{pattern}")
					};
				let glob = GlobBuilder::new(&prefixed)
					.literal_separator(true)
					.backslash_escape(false)
					.build()?;
				paths.add(glob);
			} else {
				// Filename pattern: match against name only
				let glob = GlobBuilder::new(pattern)
					.literal_separator(false)
					.backslash_escape(false)
					.build()?;
				names.add(glob);
			}
		}

		Ok(Self(Exclude::new(Arc::new(names.build()?), Arc::new(paths.build()?))))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_empty_patterns() {
		let result = MgrExclude::try_from(vec![]);
		assert!(result.is_ok());
		assert!(result.unwrap().get().is_empty());
	}

	#[test]
	fn test_valid_name_patterns() {
		let patterns = vec![".DS_Store".to_owned(), "*.pyc".to_owned()];
		let result = MgrExclude::try_from(patterns);
		assert!(result.is_ok());
		let exclude = result.unwrap().get();
		assert!(exclude.matches_name(b".DS_Store"));
		assert!(exclude.matches_name(b"foo.pyc"));
		assert!(!exclude.matches_name(b"readme.md"));
	}

	#[test]
	fn test_valid_path_patterns() {
		let patterns = vec!["build/*.tmp".to_owned()];
		let result = MgrExclude::try_from(patterns);
		assert!(result.is_ok());
		let exclude = result.unwrap().get();
		// Should not match as a name pattern
		assert!(!exclude.matches_name(b"output.tmp"));
		// Should match as a path pattern
		assert!(exclude.matches_path(b"/project/build/output.tmp"));
		assert!(!exclude.matches_path(b"/project/src/output.tmp"));
	}

	#[test]
	fn test_invalid_pattern() {
		let patterns = vec!["[invalid".to_owned()];
		let result = MgrExclude::try_from(patterns);
		assert!(result.is_err());
	}

	#[test]
	fn test_mixed_patterns() {
		let patterns = vec![".DS_Store".to_owned(), "build/*.o".to_owned()];
		let result = MgrExclude::try_from(patterns);
		assert!(result.is_ok());
		let exclude = result.unwrap().get();
		assert!(exclude.matches_name(b".DS_Store"));
		assert!(exclude.matches_path(b"/project/build/main.o"));
		assert!(!exclude.matches_path(b"/project/src/main.o"));
	}
}
