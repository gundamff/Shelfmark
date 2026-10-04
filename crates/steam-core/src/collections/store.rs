use crate::types::Collection;
use crate::Result;

pub trait CollectionStore {
    fn load(&self) -> Result<Vec<Collection>>;
    fn save(&self, collections: &[Collection]) -> Result<()>;
}
