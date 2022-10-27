function gotoFocusedItem() {
    let focusedItem = document.querySelector('.focused-item');
    if (focusedItem != null) {
        goto(focusedItem.querySelector('a')?.href)
    }
}

function editFocusedItem() {
    let focusedItem = document.querySelector('.focused-item');
    if (focusedItem != null) {
        goto(focusedItem.querySelector('a.edit-item').href)
    }
}

function focusNextItem() {
    const items = Array.from(document.getElementsByClassName('item-control'));
    const focused = document.querySelector('.focused-item');
    if (focused == undefined) {
        document.querySelector('.item-control').classList.add('focused-item');
        return
    }
    const focusedItemIndex = items.indexOf(focused);
    const next = items[focusedItemIndex + 1];
    if (next != undefined) {
        setFocusedItem(next);
    }
}

function setFocusedItem(item) {
    Array.from(document.getElementsByClassName('focused-item')).forEach(item => {
        item.classList.remove('focused-item');
    })
    item.classList.add('focused-item')
}

function focusPreviousItem() {
    const items = Array.from(document.getElementsByClassName('item-control'));
    const focused = document.querySelector('.focused-item');
    if (focused == undefined) {
        document.querySelector('.item-control').classList.add('focused-item');
        return
    }
    const focusedItemIndex = items.indexOf(focused);
    const prev = items[focusedItemIndex - 1];
    if (prev != undefined) {
        setFocusedItem(prev);
    }
}

hotkey('j', focusNextItem);
hotkey('k', focusPreviousItem);
hotkey('Enter', gotoFocusedItem);
hotkey('e', editFocusedItem)

document.querySelector('.item-control')?.classList.add('focused-item');