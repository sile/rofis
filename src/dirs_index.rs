use std::path::Path;

#[derive(Debug)]
pub struct DirsIndex {}

impl DirsIndex {
    pub fn build<P: AsRef<Path>>(root: P) -> orfail::Result<Self> {
        todo!()
    }
}
