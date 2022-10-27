const hotkeys = {}

function goto(path) {
    if (path == undefined) {
        return
    }
    let current = new URL(document.location);
    let going = new URL(path, document.location);
    if (current.href !== going.href) {
        document.location.assign(going);
    }
}

function hotkey(key, fn) {
    hotkeys[key] = fn;
}

function handleHotkey(evt) {
    console.log(evt);
    if (evt.key === 'Escape') {
        // This lets you escape and access the other hotkey.
        document.activeElement.blur();
        return;
    }
    if (evt.target != document.body) {
        // If we're focusing an element, let it handle things.
        console.debug("bailing")
        return
    }
    const handler = hotkeys[evt.key];
    if (handler != undefined) {
        console.log(`handler for ${evt.key}`)
        handler();
    }
}

document.body.addEventListener('keydown', handleHotkey, false);

function followLink(selector) {
    let link = document.querySelector(selector);
    goto(link?.href);
}

function goHome() {
    followLink("a#home");
}

function newItem() {
    followLink("a#new-item");
}

function deletedItems() {
    followLink("a#deleted-items")
}

hotkey('h', goHome);
hotkey('n', newItem);
hotkey('d', deletedItems);
