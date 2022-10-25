use chrono::{DateTime, Local};
use std::collections::HashMap;

use super::{AppError, StatusCode};

#[derive(Clone)]
pub struct Item {
    id: u32,
    title: String,
    body: String,
    important: bool,
    urgent: bool,
    created: DateTime<Local>,
}

pub struct Repo {
    items: HashMap<u32, Item>,
}

// Implementation details of Repo
impl Repo {
    fn next_id(&self) -> Result<u32, AppError> {
        let biggest = self.items.keys().max().map(|r| *r).unwrap_or(0);
        Ok(biggest + 1)
    }
}

/// Public methods of Repo
impl Repo {
    pub fn new() -> Self {
        Repo {
            items: HashMap::new(),
        }
    }

    pub fn add(
        &mut self,
        title: &str,
        body: &str,
        important: bool,
        urgent: bool,
    ) -> Result<Item, AppError> {
        let item = Item {
            id: self.next_id()?,
            title: String::from(title),
            body: String::from(body),
            important,
            urgent,
            created: Local::now(),
        };
        self.items.insert(item.id, item.clone());
        Ok(item)
    }

    pub fn get(&self, id: u32) -> Option<&Item> {
        self.items.get(&id)
    }

    pub fn all(&self) -> Result<Vec<&Item>, AppError> {
        let items = self.items.values().collect();
        Ok(items)
    }

    pub fn upsert(&mut self, item: &Item) -> Result<(), AppError> {
        self.items.insert(item.id, item.clone());
        Ok(())
    }

    pub fn delete(&mut self, id: &u32) -> Result<(), AppError> {
        self.items
            .remove(id)
            .ok_or_else(|| (StatusCode::NOT_FOUND, String::from("No such item")))
            .map(|_| ())
    }
}

#[test]
fn test_repo() -> Result<(), AppError> {
    let mut repo = Repo::new();
    let mut item = repo.add("Test Item", "Test item body.", true, false)?;
    let retrieved = repo.get(item.id).unwrap();

    fn check_items(item1: &Item, item2: &Item) {
        assert!(item1.id == item2.id);
        assert!(item1.title == item2.title);
        assert!(item1.body == item2.body);
        assert!(item1.important == item2.important);
        assert!(item1.urgent == item2.urgent);
    }

    check_items(&item, retrieved);

    item.title = String::from("Updated title");

    repo.upsert(&item)?;

    let updated = repo.get(item.id).unwrap();

    check_items(&item, updated);

    Ok(())
}
