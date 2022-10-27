function editItem() {
    let link = document.querySelector("a.edit-item");
    goto(link?.href)
}

function deleteItem() {
    let form = document.querySelector("form.delete-item");
    let response = window.confirm("Delete this item? This can't be undone")
    console.log(response)
    if (response) {
        form?.submit();
    }

}

hotkey('e', editItem);
hotkey('x', deleteItem);