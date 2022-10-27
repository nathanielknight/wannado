use super::{AppError, StatusCode};
use chrono::{DateTime, Local};
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Item {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub important: bool,
    pub urgent: bool,
    pub created: DateTime<Local>,
    pub modified: Option<DateTime<Local>>,
    pub deleted: Option<DateTime<Local>>,
}

impl Item {
    fn serialize(&self) -> Result<String, AppError> {
        serde_json::to_string(&self).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Item serializeation failed: {:?}", e),
            )
        })
    }

    fn deserialize(str: &str) -> Result<Self, AppError> {
        serde_json::from_str(str).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Item deserialization failed: {:?}\n\nRaw Item:\n{}", e, str),
            )
        })
    }

    fn modified(&mut self) {
        self.modified = Some(Local::now());
    }

    fn delete(&mut self) {
        self.deleted = Some(Local::now());
    }

    fn restore(&mut self) {
        self.deleted = None;
    }
}

pub struct Repo {
    cxn: Connection,
}

/// Public methods of Repo
impl Repo {
    pub fn new(cxn: Connection) -> Self {
        Repo { cxn }
    }

    pub fn init(&mut self) -> rusqlite::Result<()> {
        self.cxn
            .execute(
                "CREATE TABLE IF NOT EXISTS items (item BLOB NOT NULL)",
                params![],
            )
            .map(|_| ())
    }

    pub fn add(
        &mut self,
        title: &str,
        body: &str,
        important: bool,
        urgent: bool,
    ) -> Result<Item, AppError> {
        let tx = self.cxn.transaction().map_err(convert_db_error)?;

        // Get new id
        let id: u32 = tx
            .query_row(
                "INSERT INTO items (item) VALUES ('') RETURNING rowid",
                params![],
                |r| r.get(0),
            )
            .map_err(convert_db_error)?;
        let item = Item {
            id,
            title: String::from(title),
            body: String::from(body),
            important,
            urgent,
            created: Local::now(),
            modified: None,
            deleted: None,
        };
        tx.execute(
            "UPDATE items SET item = ? WHERE rowid = ?",
            params![item.serialize()?, id],
        )
        .map_err(convert_db_error)?;
        tx.commit().map_err(convert_db_error)?;
        Ok(item)
    }

    pub fn get(&self, id: u32) -> Result<Item, AppError> {
        self.get_any(id).and_then(|i| {
            if i.deleted.is_some() {
                Err((StatusCode::NOT_FOUND, "Item has been deleted".to_owned()))
            } else {
                Ok(i)
            }
        })
    }
    pub fn get_deleted(&self, id: u32) -> Result<Item, AppError> {
        self.get_any(id).and_then(|i| {
            if i.deleted.is_none() {
                Err((StatusCode::NOT_FOUND, "No such deleted item".to_owned()))
            } else {
                Ok(i)
            }
        })
    }

    fn get_any(&self, id: u32) -> Result<Item, AppError> {
        self.cxn
            .query_row(
                "SELECT item FROM items WHERE rowid = ?",
                params![id],
                |r| -> Result<String, rusqlite::Error> { r.get(0) },
            )
            .optional()
            .map_err(convert_db_error)?
            .ok_or((StatusCode::NOT_FOUND, String::from("No such item")))
            .and_then(|s| Item::deserialize(&s))
    }

    // Get all un-deleted items
    pub fn all(&mut self) -> Result<Vec<Item>, AppError> {
        let active_items = self
            .active_and_deleted()?
            .into_iter()
            .filter(|item| item.deleted.is_none())
            .collect();
        Ok(active_items)
    }

    // Items that have been deleted
    pub fn deleted(&mut self) -> Result<Vec<Item>, AppError> {
        let deleted_items = self
            .active_and_deleted()?
            .into_iter()
            .filter(|item| item.deleted.is_some())
            .collect();
        Ok(deleted_items)
    }

    fn active_and_deleted(&mut self) -> Result<Vec<Item>, AppError> {
        let serialized_items = self.all_items_raw().map_err(convert_db_error)?;
        serialized_items
            .iter()
            .map(|s| Item::deserialize(s))
            .into_iter()
            .collect()
    }

    fn all_items_raw(&mut self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.cxn.prepare("SELECT item FROM items")?;
        let result = stmt.query_map([], |r| r.get(0))?;
        result.collect()
    }

    pub fn update(&mut self, item: &mut Item) -> Result<(), AppError> {
        let cmd = "UPDATE items SET item = ? WHERE rowid = ?";
        item.modified();
        self.cxn
            .execute(cmd, params![item.serialize()?, item.id])
            .map_err(convert_db_error)
            .map(|_| ())
    }

    pub fn delete(&mut self, id: &u32) -> Result<(), AppError> {
        let mut item = self.get(*id)?;
        item.delete();
        self.update(&mut item)?;
        Ok(())
    }

    pub fn restore(&mut self, id: &u32) -> Result<(), AppError> {
        let mut item = self.get_deleted(*id)?;
        item.restore();
        self.update(&mut item)?;
        Ok(())
    }
}

// Helpers
fn convert_db_error(err: rusqlite::Error) -> AppError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {:?}", err),
    )
}

#[test]
fn test_repo() -> Result<(), AppError> {
    fn compare_item_fields(item1: &Item, item2: &Item) {
        assert!(item1.id == item2.id);
        assert!(item1.title == item2.title);
        assert!(item1.body == item2.body);
        assert!(item1.important == item2.important);
        assert!(item1.urgent == item2.urgent);
    }

    let cxn = Connection::open_in_memory().map_err(convert_db_error)?;
    let mut repo = Repo::new(cxn);
    repo.init().map_err(convert_db_error)?;

    let mut item = repo.add("Test Item", "Test item body.", true, false)?;
    assert!(item.modified.is_none());
    assert!(item.deleted.is_none());

    let retrieved = repo.get(item.id)?;
    compare_item_fields(&item, &retrieved);
    assert!(item.modified.is_none());
    assert!(item.deleted.is_none());
    assert!(retrieved.modified.is_none());
    assert!(retrieved.deleted.is_none());

    item.title = String::from("Updated title");
    repo.update(&mut item)?;
    assert!(item.modified.is_some());
    assert!(item.deleted.is_none());

    let updated = repo.get(item.id)?;
    compare_item_fields(&item, &updated);
    assert!(updated.modified.is_some());
    assert!(updated.deleted.is_none());

    repo.delete(&item.id)?; // It's too bad this doesn't mark the item as deleted...
    assert!(repo.get(item.id).is_err());
    dbg!(repo.get_deleted(item.id));
    assert!(repo.get_deleted(item.id).is_ok());

    Ok(())
}
