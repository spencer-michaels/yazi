use std::sync::Arc;

use globset::{Candidate, GlobSet};
use yazi_shared::url::AsUrl;

use crate::File;

/// Compiled glob patterns for unconditionally hiding files from listings.
///
/// Stores two separate `GlobSet` instances:
/// - `names`: for filename-only patterns (no `/`), matched against the file's name
/// - `paths`: for path patterns (contains `/`), matched against the file's full URL
#[derive(Clone, Debug, Default)]
pub struct Exclude {
	names: Option<Arc<GlobSet>>,
	paths: Option<Arc<GlobSet>>,
}

impl Exclude {
	pub fn new(names: Arc<GlobSet>, paths: Arc<GlobSet>) -> Self {
		Self {
			names: if names.is_empty() { None } else { Some(names) },
			paths: if paths.is_empty() { None } else { Some(paths) },
		}
	}

	/// Check if a filename matches any name-only exclude pattern.
	#[inline]
	pub fn matches_name(&self, name_bytes: &[u8]) -> bool {
		self.names.as_ref().is_some_and(|gs| gs.is_match_candidate(&Candidate::from_bytes(name_bytes)))
	}

	/// Check if a full path matches any path-based exclude pattern.
	#[inline]
	pub fn matches_path(&self, path_bytes: &[u8]) -> bool {
		self.paths.as_ref().is_some_and(|gs| gs.is_match_candidate(&Candidate::from_bytes(path_bytes)))
	}

	/// Check if a `File` matches any exclude pattern (name or path).
	#[inline]
	pub fn matches_file(&self, f: &File) -> bool {
		f.name().is_some_and(|n| self.matches_name(n.encoded_bytes()))
			|| self.paths.as_ref().is_some_and(|gs| {
				gs.is_match_candidate(&Candidate::from_bytes(f.url.as_url().loc().encoded_bytes()))
			})
	}

	#[inline]
	pub fn is_empty(&self) -> bool { self.names.is_none() && self.paths.is_none() }
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use globset::{Glob, GlobSetBuilder};

	use super::*;

	fn build_names(patterns: &[&str]) -> Arc<GlobSet> {
		let mut b = GlobSetBuilder::new();
		for p in patterns {
			b.add(Glob::new(p).unwrap());
		}
		Arc::new(b.build().unwrap())
	}

	fn build_paths(patterns: &[&str]) -> Arc<GlobSet> { build_names(patterns) }

	#[test]
	fn test_empty_exclude() {
		let e = Exclude::default();
		assert!(e.is_empty());
		assert!(!e.matches_name(b".DS_Store"));
		assert!(!e.matches_path(b"/foo/bar"));
	}

	#[test]
	fn test_exact_filename() {
		let e = Exclude::new(build_names(&[".DS_Store", "Thumbs.db"]), build_paths(&[]));
		assert!(e.matches_name(b".DS_Store"));
		assert!(e.matches_name(b"Thumbs.db"));
		assert!(!e.matches_name(b"readme.md"));
	}

	#[test]
	fn test_glob_extension() {
		let e = Exclude::new(build_names(&["*.pyc", "*.o"]), build_paths(&[]));
		assert!(e.matches_name(b"module.pyc"));
		assert!(e.matches_name(b"main.o"));
		assert!(!e.matches_name(b"main.py"));
		assert!(!e.matches_name(b"main.obj"));
	}

	#[test]
	fn test_directory_names() {
		let e = Exclude::new(build_names(&["node_modules", "__pycache__", ".git"]), build_paths(&[]));
		assert!(e.matches_name(b"node_modules"));
		assert!(e.matches_name(b"__pycache__"));
		assert!(e.matches_name(b".git"));
		assert!(!e.matches_name(b".gitignore"));
	}

	#[test]
	fn test_path_patterns() {
		let e = Exclude::new(build_names(&[]), build_paths(&["**/build/*.tmp"]));
		assert!(e.matches_path(b"/home/user/project/build/output.tmp"));
		assert!(!e.matches_path(b"/home/user/project/src/output.tmp"));
	}

	#[test]
	fn test_combined_name_and_path() {
		let e = Exclude::new(build_names(&[".DS_Store"]), build_paths(&["**/build/*.tmp"]));
		assert!(e.matches_name(b".DS_Store"));
		assert!(!e.matches_name(b"output.tmp"));
		assert!(e.matches_path(b"/project/build/output.tmp"));
		assert!(!e.is_empty());
	}
}
