use crate::repo;
use crate::{AppError, StatusCode};
use askama::Template;
use chrono::{DateTime, TimeZone, Utc, Local};

struct ViewItem {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub important: bool,
    pub urgent: bool,
    pub created: DateTime<Local>,
    pub modified: Option<DateTime<Local>>,
    pub deleted: Option<DateTime<Local>>,
}

impl<'a> TryFrom<&'a repo::Item> for ViewItem {
    type Error = AppError;

    fn try_from(item: &'a repo::Item) -> Result<ViewItem, Self::Error> {
        fn parse_ts(ts: i64) -> Result<DateTime<Local>, AppError> {
            Utc.timestamp_opt(ts, 0).single().ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error deserializing timestamp from {}", ts),
            )).map(|utc| utc.into())
        }
        let created = parse_ts(item.created)?;
        let modified = item.modified.map(parse_ts).transpose()?;
        let deleted = item.deleted.map(parse_ts).transpose()?;
        Ok(ViewItem {
            id: item.id,
            title: item.title.clone(),
            body: item.body.clone(),
            important: item.important,
            urgent: item.urgent,
            created,
            modified,
            deleted,
        })
    }
}

#[derive(Template)]
#[template(path = "items-list.html")]
pub struct ItemsList<'a> {
    important_and_urgent: Vec<&'a repo::Item>,
    important: Vec<&'a repo::Item>,
    urgent: Vec<&'a repo::Item>,
    other: Vec<&'a repo::Item>,
}

impl<'a> ItemsList<'a> {
    pub fn from_items(items: &'a Vec<repo::Item>) -> Result<ItemsList<'a>, AppError> {
        let mut items_list = ItemsList {
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
                (true, true) => items_list.important_and_urgent.push(item),
                (true, false) => items_list.important.push(item),
                (false, true) => items_list.urgent.push(item),
                (false, false) => items_list.other.push(item),
            }
        }
        Ok(items_list)
    }
}

#[derive(Template)]
#[template(path = "deleted-items-list.html")]
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
            items.reverse();
            Ok(DeletedItems { items })
        }
    }
}

#[derive(Template)]
#[template(path = "item.html")]
pub struct Item {
    item: ViewItem,
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
            ViewItem::try_from(&item).map(|item| Item { item })
        }
    }
}

#[derive(Template)]
#[template(path = "edit-item.html")]
pub struct EditItem {
    item: ViewItem,
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
            ViewItem::try_from(&item).map(|item| EditItem { item })
        }
    }
}

#[derive(Template, Default)]
#[template(path = "new-item.html")]
pub struct NewItem<'a> {
    title: Option<&'a str>,
    body: Option<&'a str>,
    important: Option<bool>,
    urgent: Option<bool>,
}

#[derive(Template)]
#[template(path = "deleted-item.html")]
pub struct DeletedItem {
    item: ViewItem,
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
            ViewItem::try_from(&item).map(|item| DeletedItem { item })
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
