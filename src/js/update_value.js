function plain_text_node(editor, name, text) {
  let node = document.createElement("div");
  node.classList.add("plain-text-node");
  node.innerText = text;
  node.contentEditable = true;
  setTimeout(() => window.getSelection().selectAllChildren(node));
  node.addEventListener("keydown", async (e) => {
    if (e.key === "Escape") {
      node.dispatchEvent(new Event("reload", { bubbles: true }));
    } else if (e.key === "Enter") {
      e.preventDefault();
      await fetch(url_for(editor, `/${name}/value`), {
        method: "POST",
        body: node.innerText,
      });
      node.dispatchEvent(new Event("reload", { bubbles: true }));
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
  for (let editor of document.querySelectorAll(".--editor")) {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  }
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});
