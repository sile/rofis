use orfail::OrFail;
use patricia_tree::PatriciaSet;
use std::path::Path;

#[derive(Debug)]
pub struct DirsIndex {
    index: PatriciaSet,
}

impl DirsIndex {
    pub fn build<P: AsRef<Path>>(root: P) -> orfail::Result<Self> {
        let root = root.as_ref().canonicalize().or_fail()?;
        let mut index = PatriciaSet::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(dir).or_fail()? {
                let entry = entry.or_fail()?;
                let file_type = entry.file_type().or_fail()?;
                let file_path = entry.path();
                if file_type.is_dir() {
                    if file_path.file_name().map_or(false, |name| {
                        name.to_str().map_or(false, |name| !name.starts_with("."))
                    }) {
                        let relative_path = file_path
                            .strip_prefix(&root)
                            .or_fail()?
                            .to_str()
                            .or_fail()?;
                        index.insert(relative_path.bytes().rev().collect::<Vec<_>>());
                        stack.push(file_path);
                    }
                }
            }
        }
        Ok(Self { index })
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }
}
