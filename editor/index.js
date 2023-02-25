import { basicSetup, EditorView } from "codemirror"
import { markdown } from "@codemirror/lang-markdown"

// Hide the existing body
let textarea = document.querySelector('textarea');
textarea.style.display = "none";

// Create a CodeMirror editor with the body's contents
let view = new EditorView({
    doc: textarea.value,
    extensions: [
        EditorView.lineWrapping,
        basicSetup,
        markdown({})
    ],
});
textarea.insertAdjacentElement("afterend", view.dom);


// When submitting the form, update the original body with the new contents
textarea.parentElement.onsubmit = function () {
    textarea.value = view.state.doc;
}


