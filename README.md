# Wannado

> Keep track of what you want to do. Automate it however you like.

Wannado is a web app for keeping track of "items": things you want to do.
Each item has a title and a [CommonMark] body
for keeping notes, links, checklists,etc.
Each item can be Important and/or Urgent.
These items are stored in a [SQLite] database.

As well as this web app you can provide a script for Wannado to run at
regular intervals. This script can access the app's database, so it can
do whatever you want:

* Want to get rid of anything older than two weeks? Look for entries that old and
  delete them or mark them as urgent.
* Want to snooze items? Put `restore-on <iso-date>` in an item's note, delete
  it, and then find items to restore with a regex. (Item's deleted via the web
  app aren't actually deleted, just marked with a `deleted_at` timestamp.)
* Want stats? Write queries for whatever you're interested in and save them in a
  file, send them to yourself, or save them as their own item for easy access.

## Usage

You can control the address and port that Wannado binds to with a command-line
argument:

`wannado [address:port]`

The automation script is configured via environment variables:

* `WANNADO_SCRIPT` to specify the command to run (this gets passed to
  [`std::process::Command::new`])
* `WANNADO_SCRIPT_INTERVAL_IN_SECOND` controls how often the script should run
  (defaults to 5 minutes)

The script can be anything you like; I like to use a Python script because it
has SQLite, JSON, Regex and date/time support in the standard library, but it
can be literally anything you can invoke from your server.


## Security

Wannado does no authentication and makes no effort to prevent script injection;
it's not meant to be served to the general public.

If you want it to be accessible on the go, I can suggest two options:

* Run it in a [Tailscale] network to keep it private
* Run it behind an HTTPS and Basic Auth capable reverse-proxy like (e.g.
  [Caddy], for which an example configuration is included below)


## Hotkeys

| Key | Action |
|-|-|
| `h` | Go to Home page |
| `d` | Go to deleted items |
| `n` | Create a new item |

On the home page:

| Key | Action |
|-|-|
| `j` | Select next item |
| `k` | Select previous item |
| `Enter` | Go to selected item |

On an item page:

| Key | Action |
|-|-|
| `e` | Edit the item |
| `x` | Delete the item |


On a deleted item's page:

| Key | Action |
|-|-|
| `r` | Restore the item |

## Database Schema

The code describing Wannado's items is:

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Item {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub important: bool,
    pub urgent: bool,
    pub created: i64,  // Times stored as Unix Timestamps
    pub modified: Option<i64>,
    pub deleted: Option<i64>,
}
```

These items are stored as JSON encoded blobs in the database:

```sql
CREATE TABLE IF NOT EXISTS items (item BLOB NOT NULL)
```

The [SQLite JSON functions] can be used for queries and updates.

This format may change in the future.


[CommonMark]: https://commonmark.org/
[Tailscale]: https://tailscale.com/
[Caddy]: https://caddyserver.com/
[SQLite]: https://www.sqlite.org/index.html
[`std::process::Command::new`]: https://doc.rust-lang.org/std/process/struct.Command.html#method.new
[SQLite JSON functions]: https://www.sqlite.org/json1.html