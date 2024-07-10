import { h, modifiers } from "./html.js";
const { invoke } = window.__TAURI__.tauri;

function plain_text_node(editor, name, original_value) {
  return h.div(
    {
      class: "plain-text-node",
      "data-kind": "plain-text",
      "data-string-value": original_value,
      contenteditable: true,
      async "@keydown"(e) {
        if (e.key === "Escape") {
          this.blur();
        } else if (e.key === "Enter") {
          e.preventDefault();
          await invoke("update_value", {
            selector: editor.dataset.selector,
            name,
            original_value,
            value: this.innerText,
          });
          this.blur();
        }
        [];
      },
      async "@blur"(_) {
        this.dispatchEvent(new Event("reload", { bubbles: true }));
      },
      [modifiers.on_mount]() {
        window.getSelection().selectAllChildren(this);
      },
    },
    original_value,
  );
}

const VALUE_SELECTOR =
  '[data-attr="properties"] > [data-kind="property"] > [data-attr="value"] > [data-kind]';

function init(editor) {
  for (let value of editor.querySelectorAll(VALUE_SELECTOR)) {
    let name = value
      .closest('[data-kind="property"]')
      .querySelector('[data-attr="name"]').innerText;
    value.addEventListener("dblclick", (_) => {
      value.replaceWith(
        plain_text_node(editor, name, value.dataset.stringValue),
      );
    });
  }
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
