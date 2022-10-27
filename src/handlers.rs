use axum::{
    extract::{Extension, Form, Path},
    http::StatusCode,
    response::{Html, Redirect},
};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::AppError;
use crate::{repo, template};

pub(crate) async fn get_items(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
) -> Result<Html<String>, AppError> {
    let mut repo = lock_repo(&repomux)?;
    let mut items = repo.all()?;
    items.sort_by_key(|i| (i.modified, i.created));
    let viewmodel = template::ItemsList::from_items(&items)?;
    let body = viewmodel.to_string();
    Ok(Html(body))
}

pub(crate) async fn get_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
) -> Result<Html<String>, AppError> {
    let repo = lock_repo(&repomux)?;
    let item = repo.get(item_id)?;
    let viewmodel: template::Item = item.try_into()?;
    let body = viewmodel.to_string();
    Ok(Html(body))
}

pub(crate) async fn get_edit_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
) -> Result<Html<String>, AppError> {
    let repo = lock_repo(&repomux)?;
    let item = repo.get(item_id)?;
    let viewmodel: template::EditItem = item.try_into()?;
    let body = viewmodel.to_string();
    Ok(Html(body))
}

pub(crate) async fn post_edit_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
    Form(edits): Form<EditParams>,
) -> Result<Redirect, AppError> {
    let mut repo = lock_repo(&repomux)?;
    let mut item = repo.get(item_id)?;
    item.apply(&edits);
    let goto = Redirect::to(&format!("/item/{}", item.id));
    repo.update(&mut item)?;
    Ok(goto)
}

pub(crate) async fn post_delete_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
) -> Result<Redirect, AppError> {
    let mut repo = lock_repo(&repomux)?;
    repo.delete(&item_id)?;
    Ok(Redirect::to("/"))
}

pub(crate) async fn get_new_item() -> Html<String> {
    Html(template::NewItem::default().to_string())
}

pub(crate) async fn post_new_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Form(edits): Form<EditParams>,
) -> Result<Redirect, AppError> {
    let mut repo = lock_repo(&repomux)?;
    let item = repo.add(
        &edits.title,
        &edits.body,
        edits.important.is_some(),
        edits.urgent.is_some(),
    )?;
    Ok(Redirect::to(&format!("/item/{}", item.id)))
}

pub(crate) async fn get_deleted_items(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
) -> Result<Html<String>, AppError> {
    let mut repo = lock_repo(&repomux)?;
    let items = repo.deleted()?;
    let viewmodel = template::DeletedItems::try_from(items)?;
    let body = viewmodel.to_string();
    Ok(Html(body))
}

pub(crate) async fn get_deleted_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
) -> Result<Html<String>, AppError> {
    let repo = lock_repo(&repomux)?;
    let item = repo.get_deleted(item_id)?;
    let viewmodel: template::DeletedItem = item.try_into()?;
    let body = viewmodel.to_string();
    Ok(Html(body))
}

pub(crate) async fn restore_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
) -> Result<Redirect, AppError> {
    let mut repo = lock_repo(&repomux)?;
    repo.restore(&item_id)?;
    Ok(Redirect::to(&format!("/item/{}", item_id)))
}

// Helpers
fn lock_repo(repomux: &Arc<Mutex<repo::Repo>>) -> Result<MutexGuard<repo::Repo>, AppError> {
    repomux.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Couldn't lock the item repo: {:?}", e),
        )
    })
}

#[derive(serde::Deserialize)]
pub(crate) struct EditParams {
    pub title: String,
    pub body: String,
    pub important: Option<String>,
    pub urgent: Option<String>,
}

impl repo::Item {
    fn apply(&mut self, edits: &EditParams) {
        self.title.clear();
        self.title.insert_str(0, &edits.title);
        self.body.clear();
        self.body.insert_str(0, &edits.body);
        self.important = edits.important.is_some();
        self.urgent = edits.urgent.is_some();
    }
}
