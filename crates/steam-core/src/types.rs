pub type AppId = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub app_id: AppId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub app_ids: Vec<AppId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectionChangeset {
    pub create: Vec<Collection>,
    pub rename: Vec<(String, String)>,
    pub delete: Vec<String>,
    pub add_members: Vec<(String, Vec<AppId>)>,
    pub remove_members: Vec<(String, Vec<AppId>)>,
}
