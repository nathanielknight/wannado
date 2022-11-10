function restoreItem() {
    let form = document.querySelector("form.restore-item");
    let response = window.confirm("Restore this item?")
    console.log(response)
    if (response) {
        form?.submit();
    }
}

hotkey('r', restoreItem);
