const { invoke } = window.__TAURI__.tauri;

function plain_text_node(editor, name, original_value) {
  let node = document.createElement("div");
  node.classList.add("plain-text-node");
  node.innerText = original_value;
  node.dataset.kind = "plain-text";
  node.dataset.stringValue = original_value;
  node.contentEditable = true;
  setTimeout(() => window.getSelection().selectAllChildren(node));
  node.addEventListener("keydown", async (e) => {
    if (e.key === "Escape") {
      node.blur();
    } else if (e.key === "Enter") {
      e.preventDefault();
      await invoke("update_value", {
        selector: editor.dataset.selector,
        name,
        original_value,
        value: node.innerText,
      });
      node.blur();
    }
  });
  node.addEventListener("blur", (_) => {
    node.dispatchEvent(new Event("reload", { bubbles: true }));
  });
  return node;
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
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
