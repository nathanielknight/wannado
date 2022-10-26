use crate::repo;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    important_and_urgent: Vec<&'a repo::Item>,
    important: Vec<&'a repo::Item>,
    urgent: Vec<&'a repo::Item>,
    other: Vec<&'a repo::Item>,
}

impl<'a> Index<'a> {
    pub fn from_items(items: &'a Vec<&repo::Item>) -> Index<'a> {
        let mut index = Index {
            important_and_urgent: Vec::new(),
            important: Vec::new(),
            urgent: Vec::new(),
            other: Vec::new(),
        };
        for item in items {
            match (item.important, item.urgent) {
                (true, true) => index.important_and_urgent.push(item),
                (true, false) => index.important.push(item),
                (false, true) => index.urgent.push(item),
                (false, false) => index.other.push(item),
            }
        }
        index
    }
}

#[derive(Template)]
#[template(path = "item.html")]
pub struct Item<'a> {
    item: &'a repo::Item,
}

impl<'a> From<&'a repo::Item> for Item<'a> {
    fn from(item: &'a repo::Item) -> Self {
        Item { item }
    }
}

#[derive(Template)]
#[template(path = "edit_item.html")]
pub struct EditItem<'a> {
    item: &'a repo::Item,
}

impl<'a> From<&'a repo::Item> for EditItem<'a> {
    fn from(item: &'a repo::Item) -> Self {
        EditItem { item }
    }
}

#[derive(Template, Default)]
#[template(path = "new_item.html")]
pub struct NewItem<'a> {
    title: Option<&'a str>,
    body: Option<&'a str>,
    important: Option<bool>,
    urgent: Option<bool>,
}
