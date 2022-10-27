function restoreItem() {
    let form = document.querySelector("form.restore-item");
    let response = window.confirm("Delete this item? This can't be undone")
    console.log(response)
    if (response) {
        form?.submit();
    }
}

hotkey('r', restoreItem);