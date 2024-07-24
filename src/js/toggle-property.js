import invoke from "./invoke.js";

async function toggle(e) {
  e.stopPropagation();
  let property = this.closest('[data-kind="property"]');
  let name = property.querySelector('[data-attr="name"]').textContent;
  let value = property.querySelector('[data-attr="value"] > [data-kind]')
    .dataset.stringValue;
  let is_commented = property.dataset.commented === "true";
  let action = is_commented ? "enable" : "disable";
  let editor = property.closest(".--editor");
  await invoke(editor, action, {
    path: localStorage.getItem("current-path"),
    selector: editor.dataset.selector,
    name,
    value,
  });
}

function insert_comment_button(src) {
  let button = document.createElement("button");
  button.innerHTML = "<div class='icon'></div>";
  button.classList.add("toggle-comment");
  button.addEventListener("mousedown", toggle);
  src.prepend(button);
}

const PROPERTY_SELECTOR =
  "[data-attr=properties] > [data-kind=property]:not(:has(.toggle-comment))";

function init(editor) {
  for (let property of editor.querySelectorAll(PROPERTY_SELECTOR))
    insert_comment_button(property);
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
