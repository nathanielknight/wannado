use crate::repo;
use crate::{AppError, StatusCode};
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
    pub fn from_items(items: &'a Vec<repo::Item>) -> Result<Index<'a>, AppError> {
        let mut index = Index {
            important_and_urgent: Vec::new(),
            important: Vec::new(),
            urgent: Vec::new(),
            other: Vec::new(),
        };
        if items.iter().any(|i| i.deleted.is_some()) {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Deleted item ended up in list of items".to_owned(),
            ));
        }
        for item in items {
            match (item.important, item.urgent) {
                (true, true) => index.important_and_urgent.push(item),
                (true, false) => index.important.push(item),
                (false, true) => index.urgent.push(item),
                (false, false) => index.other.push(item),
            }
        }
        Ok(index)
    }
}

#[derive(Template)]
#[template(path = "deleted_index.html")]
pub struct DeletedItems {
    items: Vec<repo::Item>,
}

impl TryFrom<Vec<repo::Item>> for DeletedItems {
    type Error = AppError;

    fn try_from(mut items: Vec<repo::Item>) -> Result<Self, Self::Error> {
        if items.iter().any(|i| i.deleted.is_none()) {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Active item ended up in deleted list".to_owned(),
            ))
        } else {
            items.sort_by_key(|i| i.deleted);
            Ok(DeletedItems { items })
        }
    }
}

#[derive(Template)]
#[template(path = "item.html")]
pub struct Item {
    item: repo::Item,
}

impl TryFrom<repo::Item> for Item {
    type Error = AppError;

    fn try_from(item: repo::Item) -> Result<Self, Self::Error> {
        if item.deleted.is_some() {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Tried to render a deleted item.".to_owned(),
            ))
        } else {
            Ok(Item { item })
        }
    }
}

#[derive(Template)]
#[template(path = "edit_item.html")]
pub struct EditItem {
    item: repo::Item,
}

impl TryFrom<repo::Item> for EditItem {
    type Error = AppError;

    fn try_from(item: repo::Item) -> Result<Self, Self::Error> {
        if item.deleted.is_some() {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Tried to edit a deleted item".to_owned(),
            ))
        } else {
            Ok(EditItem { item })
        }
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

#[derive(Template)]
#[template(path = "deleted_item.html")]
pub struct DeletedItem {
    item: repo::Item,
}

impl TryFrom<repo::Item> for DeletedItem {
    type Error = AppError;

    fn try_from(item: repo::Item) -> Result<Self, Self::Error> {
        if item.deleted.is_none() {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Tried to edit a deleted item".to_owned(),
            ))
        } else {
            Ok(DeletedItem { item })
        }
    }
}

mod filters {
    //! Additional Askama filters.

    pub fn md(src: &str) -> askama::Result<String> {
        use pulldown_cmark::{html, Options, Parser};

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(src, options);
        let mut output = String::new();
        html::push_html(&mut output, parser);
        Ok(output)
    }
}
