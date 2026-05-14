use std::{mem, ops::{Deref, DerefMut, Not}};

use hashbrown::{HashMap, HashSet};
use yazi_shared::{Id, path::{PathBufDyn, PathDyn}};

use super::{FilesSorter, Filter};
use crate::{Exclude, FILES_TICKET, File, SortBy};

#[derive(Default)]
pub struct Files {
	hidden:       Vec<File>,
	items:        Vec<File>,
	ticket:       Id,
	version:      u64,
	pub revision: u64,

	pub sizes: HashMap<PathBufDyn, u64>,

	sorter:      FilesSorter,
	filter:      Option<Filter>,
	show_hidden:   bool,
	show_excluded: bool,
	exclude:       Exclude,
}

impl Deref for Files {
	type Target = Vec<File>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl DerefMut for Files {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.items }
}

impl Files {
	pub fn new(show_hidden: bool, exclude: Exclude) -> Self {
		Self { show_hidden, exclude, ..Default::default() }
	}

	pub fn update_full(&mut self, files: Vec<File>) {
		self.ticket = FILES_TICKET.next();

		let (hidden, items) = self.split_files(files);
		if !(items.is_empty() && self.items.is_empty()) {
			self.revision += 1;
		}

		(self.hidden, self.items) = (hidden, items);
	}

	pub fn update_part(&mut self, files: Vec<File>, ticket: Id) {
		if !files.is_empty() {
			if ticket != self.ticket {
				return;
			}

			let (hidden, items) = self.split_files(files);
			if !items.is_empty() {
				self.revision += 1;
			}

			self.hidden.extend(hidden);
			self.items.extend(items);
			return;
		}

		self.ticket = ticket;
		self.hidden.clear();
		if !self.items.is_empty() {
			self.revision += 1;
			self.items.clear();
		}
	}

	pub fn update_size(&mut self, mut sizes: HashMap<PathBufDyn, u64>) {
		if sizes.len() <= 50 {
			sizes.retain(|k, v| self.sizes.get(k) != Some(v));
		}

		if sizes.is_empty() {
			return;
		}

		if self.sorter.by == SortBy::Size {
			self.revision += 1;
		}
		self.sizes.extend(sizes);
	}

	pub fn update_ioerr(&mut self) {
		self.ticket = FILES_TICKET.next();
		self.hidden.clear();
		self.items.clear();
	}

	pub fn update_creating(&mut self, files: Vec<File>) {
		if files.is_empty() {
			return;
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut todo: HashMap<_, _> = $src.into_iter().map(|f| (f.urn().to_owned(), f)).collect();
				for f in &$dist {
					if todo.remove(&f.urn()).is_some() && todo.is_empty() {
						break;
					}
				}
				if !todo.is_empty() {
					self.revision += $inc;
					$dist.extend(todo.into_values());
				}
			};
		}

		let (hidden, items) = self.split_files(files);
		if !items.is_empty() {
			go!(self.items, items, 1);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden, 0);
		}
	}

	#[cfg(unix)]
	pub fn update_deleting(&mut self, urns: HashSet<PathBufDyn>) -> Vec<usize> {
		use yazi_shared::path::PathLike;
		if urns.is_empty() {
			return vec![];
		}

		let (mut hidden, mut items) = if let Some(filter) = &self.filter {
			urns.into_iter().partition(|u| {
				self.is_urn_excluded(u)
					|| (!self.show_hidden && u.is_hidden())
					|| !filter.matches(u)
			})
		} else if self.show_hidden {
			if self.exclude.is_empty() {
				(HashSet::new(), urns)
			} else {
				urns.into_iter().partition(|u| self.is_urn_excluded(u))
			}
		} else {
			urns.into_iter().partition(|u| u.is_hidden() || self.is_urn_excluded(u))
		};

		let mut deleted = Vec::with_capacity(items.len());
		if !items.is_empty() {
			let mut i = 0;
			self.items.retain(|f| {
				let b = items.remove(&f.urn());
				if b {
					deleted.push(i);
				}
				i += 1;
				!b
			});
		}
		if !hidden.is_empty() {
			self.hidden.retain(|f| !hidden.remove(&f.urn()));
		}

		self.revision += deleted.is_empty().not() as u64;
		deleted
	}

	#[cfg(windows)]
	pub fn update_deleting(&mut self, mut urns: HashSet<PathBufDyn>) -> Vec<usize> {
		let mut deleted = Vec::with_capacity(urns.len());
		if !urns.is_empty() {
			let mut i = 0;
			self.items.retain(|f| {
				let b = urns.remove(&f.urn());
				if b {
					deleted.push(i)
				}
				i += 1;
				!b
			});
		}
		if !urns.is_empty() {
			self.hidden.retain(|f| !urns.remove(&f.urn()));
		}

		self.revision += deleted.is_empty().not() as u64;
		deleted
	}

	pub fn update_updating(
		&mut self,
		files: HashMap<PathBufDyn, File>,
	) -> (HashMap<PathBufDyn, File>, HashMap<PathBufDyn, File>) {
		if files.is_empty() {
			return Default::default();
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut b = true;
				for i in 0..$dist.len() {
					if let Some(f) = $src.remove(&$dist[i].urn()) {
						b = b && $dist[i].cha.hits(f.cha);
						b = b && $dist[i].urn() == f.urn();

						$dist[i] = f;
						if $src.is_empty() {
							break;
						}
					}
				}
				self.revision += if b { 0 } else { $inc };
			};
		}

		let (mut hidden, mut items) = if let Some(filter) = &self.filter {
			files.into_iter().partition(|(_, f)| {
				self.is_excluded(f)
					|| (f.is_hidden() && !self.show_hidden)
					|| !filter.matches(f.urn())
			})
		} else if self.show_hidden {
			if self.exclude.is_empty() {
				(HashMap::new(), files)
			} else {
				files.into_iter().partition(|(_, f)| self.is_excluded(f))
			}
		} else {
			files.into_iter().partition(|(_, f)| f.is_hidden() || self.is_excluded(f))
		};

		if !items.is_empty() {
			go!(self.items, items, 1);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden, 0);
		}
		(hidden, items)
	}

	pub fn update_upserting(&mut self, files: HashMap<PathBufDyn, File>) {
		if files.is_empty() {
			return;
		}

		self.update_deleting(
			files.iter().filter(|&(u, f)| u != f.urn()).map(|(_, f)| f.urn().into()).collect(),
		);

		let (hidden, items) = self.update_updating(files);
		if hidden.is_empty() && items.is_empty() {
			return;
		}

		if !hidden.is_empty() {
			self.hidden.extend(hidden.into_values());
		}
		if !items.is_empty() {
			self.revision += 1;
			self.items.extend(items.into_values());
		}
	}

	pub fn catchup_revision(&mut self) -> bool {
		if self.version == self.revision {
			return false;
		}

		self.version = self.revision;
		self.sorter.sort(&mut self.items, &self.sizes);
		true
	}

	fn split_files(&self, files: impl IntoIterator<Item = File>) -> (Vec<File>, Vec<File>) {
		if let Some(filter) = &self.filter {
			files.into_iter().partition(|f| {
				self.is_excluded(f)
					|| (f.is_hidden() && !self.show_hidden)
					|| !filter.matches(f.urn())
			})
		} else if self.show_hidden {
			if self.exclude.is_empty() {
				(vec![], files.into_iter().collect())
			} else {
				files.into_iter().partition(|f| self.is_excluded(f))
			}
		} else {
			files.into_iter().partition(|f| f.is_hidden() || self.is_excluded(f))
		}
	}

	#[inline]
	fn is_excluded(&self, f: &File) -> bool {
		!self.show_excluded && self.exclude.matches_file(f)
	}

	#[inline]
	fn is_urn_excluded(&self, urn: &PathBufDyn) -> bool {
		use yazi_shared::path::PathLike;
		!self.show_excluded && urn.name().is_some_and(|n| self.exclude.matches_name(n.encoded_bytes()))
	}
}

impl Files {
	// --- Items
	#[inline]
	pub fn position(&self, urn: PathDyn) -> Option<usize> { self.iter().position(|f| urn == f.urn()) }

	// --- Ticket
	#[inline]
	pub fn ticket(&self) -> Id { self.ticket }

	// --- Sorter
	#[inline]
	pub fn sorter(&self) -> &FilesSorter { &self.sorter }

	pub fn set_sorter(&mut self, sorter: FilesSorter) {
		if self.sorter != sorter {
			self.sorter = sorter;
			self.revision += 1;
		}
	}

	// --- Filter
	#[inline]
	pub fn filter(&self) -> Option<&Filter> { self.filter.as_ref() }

	pub fn set_filter(&mut self, filter: Option<Filter>) -> bool {
		if self.filter == filter {
			return false;
		}

		self.filter = filter;
		if self.filter.is_none() {
			let take = mem::take(&mut self.hidden);
			let (hidden, items) = self.split_files(take);

			self.hidden = hidden;
			if !items.is_empty() {
				self.items.extend(items);
				self.sorter.sort(&mut self.items, &self.sizes);
			}
			return true;
		}

		let it = mem::take(&mut self.items).into_iter().chain(mem::take(&mut self.hidden));
		(self.hidden, self.items) = self.split_files(it);
		self.sorter.sort(&mut self.items, &self.sizes);
		true
	}

	// --- Show hidden
	pub fn set_show_hidden(&mut self, state: bool) {
		if self.show_hidden == state {
			return;
		}

		self.show_hidden = state;
		if self.show_hidden && self.hidden.is_empty() {
			return;
		} else if !self.show_hidden && self.items.is_empty() {
			return;
		}

		let take =
			if self.show_hidden { mem::take(&mut self.hidden) } else { mem::take(&mut self.items) };
		let (hidden, items) = self.split_files(take);

		self.hidden.extend(hidden);
		if !items.is_empty() {
			self.revision += 1;
			self.items.extend(items);
		}
	}

	// --- Show excluded
	pub fn set_show_excluded(&mut self, state: bool) {
		if self.show_excluded == state {
			return;
		}

		self.show_excluded = state;
		if self.show_excluded && self.hidden.is_empty() {
			return;
		} else if !self.show_excluded && self.items.is_empty() {
			return;
		}

		let take =
			if self.show_excluded { mem::take(&mut self.hidden) } else { mem::take(&mut self.items) };
		let (hidden, items) = self.split_files(take);

		self.hidden.extend(hidden);
		if !items.is_empty() {
			self.revision += 1;
			self.items.extend(items);
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{path::PathBuf, sync::Arc};

	use globset::{Glob, GlobSetBuilder};

	use super::*;
	use crate::Exclude;

	fn make_file(path: &str) -> File { File::from_dummy(PathBuf::from(path), None) }

	fn make_exclude(name_patterns: &[&str], path_patterns: &[&str]) -> Exclude {
		let mut names = GlobSetBuilder::new();
		for p in name_patterns {
			names.add(Glob::new(p).unwrap());
		}
		let mut paths = GlobSetBuilder::new();
		for p in path_patterns {
			paths.add(Glob::new(p).unwrap());
		}
		Exclude::new(Arc::new(names.build().unwrap()), Arc::new(paths.build().unwrap()))
	}

	#[cfg(unix)]
	#[test]
	fn test_excluded_always_hidden_even_with_show_hidden() {
		let exclude = make_exclude(&[".DS_Store"], &[]);
		let mut files = Files::new(true, exclude); // show_hidden = true

		let input = vec![make_file("/tmp/.DS_Store"), make_file("/tmp/readme.md")];
		files.update_full(input);

		assert_eq!(files.items.len(), 1);
		assert_eq!(files.items[0].name().unwrap().to_string_lossy(), "readme.md");
	}

	#[cfg(unix)]
	#[test]
	fn test_excluded_plus_hidden_both_hidden() {
		let exclude = make_exclude(&["*.pyc"], &[]);
		let mut files = Files::new(false, exclude); // show_hidden = false

		let input = vec![
			make_file("/tmp/.hidden"),
			make_file("/tmp/module.pyc"),
			make_file("/tmp/readme.md"),
		];
		files.update_full(input);

		// .hidden is hidden (dotfile), module.pyc is excluded, only readme.md visible
		assert_eq!(files.items.len(), 1);
		assert_eq!(files.items[0].name().unwrap().to_string_lossy(), "readme.md");
	}

	#[cfg(unix)]
	#[test]
	fn test_show_hidden_toggle_does_not_reveal_excluded() {
		let exclude = make_exclude(&[".DS_Store"], &[]);
		let mut files = Files::new(false, exclude);

		let input = vec![
			make_file("/tmp/.hidden"),
			make_file("/tmp/.DS_Store"),
			make_file("/tmp/visible.txt"),
		];
		files.update_full(input);

		// show_hidden=false: only visible.txt shown
		assert_eq!(files.items.len(), 1);

		// Toggle show_hidden on
		files.set_show_hidden(true);

		// .hidden should appear, .DS_Store should stay hidden
		assert_eq!(files.items.len(), 2);
		let names: Vec<_> =
			files.items.iter().filter_map(|f| f.name().map(|n| n.to_string_lossy().into_owned())).collect();
		assert!(names.contains(&"visible.txt".to_owned()));
		assert!(names.contains(&".hidden".to_owned()));
		assert!(!names.contains(&".DS_Store".to_owned()));
	}

	#[test]
	fn test_no_exclude_same_as_default() {
		let mut files = Files::new(true, Exclude::default());
		let input = vec![make_file("/tmp/a.txt"), make_file("/tmp/b.txt")];
		files.update_full(input);
		assert_eq!(files.items.len(), 2);
	}

	#[test]
	fn test_path_pattern_excludes_matching_path_only() {
		// Pattern **/build/output/*.tmp should only match .tmp files under build/output/
		let exclude = make_exclude(&[], &["**/build/output/*.tmp"]);
		let mut files = Files::new(true, exclude);

		let input = vec![
			make_file("/project/build/output/cache.tmp"),
			make_file("/project/build/output/debug.tmp"),
			make_file("/project/build/output/result.bin"),
			make_file("/project/src/notes.tmp"),
		];
		files.update_full(input);

		// cache.tmp and debug.tmp excluded; result.bin and notes.tmp visible
		assert_eq!(files.items.len(), 2);
		let names: Vec<_> =
			files.items.iter().filter_map(|f| f.name().map(|n| n.to_string_lossy().into_owned())).collect();
		assert!(names.contains(&"result.bin".to_owned()));
		assert!(names.contains(&"notes.tmp".to_owned()));
	}

	#[test]
	fn test_mixed_name_and_path_patterns() {
		// Name pattern for .DS_Store (everywhere) + path pattern for build/output/*.tmp
		let exclude = make_exclude(&[".DS_Store"], &["**/build/output/*.tmp"]);
		let mut files = Files::new(true, exclude);

		let input = vec![
			make_file("/project/.DS_Store"),
			make_file("/project/build/output/cache.tmp"),
			make_file("/project/build/output/result.bin"),
			make_file("/project/src/main.rs"),
		];
		files.update_full(input);

		// .DS_Store excluded by name, cache.tmp excluded by path
		assert_eq!(files.items.len(), 2);
		let names: Vec<_> =
			files.items.iter().filter_map(|f| f.name().map(|n| n.to_string_lossy().into_owned())).collect();
		assert!(names.contains(&"result.bin".to_owned()));
		assert!(names.contains(&"main.rs".to_owned()));
	}
}
