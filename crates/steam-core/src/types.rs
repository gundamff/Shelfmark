use serde::{Deserialize, Serialize};

pub type AppId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub app_id: AppId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub app_ids: Vec<AppId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionChangeset {
    pub create: Vec<Collection>,
    pub rename: Vec<(String, String)>,
    pub delete: Vec<String>,
    pub add_members: Vec<(String, Vec<AppId>)>,
    pub remove_members: Vec<(String, Vec<AppId>)>,
}

/// Numbers shown in the AutoCat preview and in the export queue.
///
/// Both are produced by [`CollectionChangeset::counts`] so the preview can
/// never disagree with what "加入待导出" actually queues.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangesetCounts {
    pub create: usize,
    pub rename: usize,
    pub delete: usize,
    pub add_members: usize,
    pub remove_members: usize,
}

impl CollectionChangeset {
    pub fn counts(&self) -> ChangesetCounts {
        ChangesetCounts {
            create: self.create.len(),
            rename: self.rename.len(),
            delete: self.delete.len(),
            add_members: self.add_members.len(),
            remove_members: self.remove_members.len(),
        }
    }
}
