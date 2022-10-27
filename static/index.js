function editFocusedItem() {
    let focusedItem = document.querySelector('.focused-item');
    if (focusedItem != null) {
        goto(focusedItem.querySelector('a.edit-item').href)
    }
}

hotkey('e', editFocusedItem)
