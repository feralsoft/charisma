import invoke from "./invoke.js";
import * as ast from "./ast.js";

async function toggle(e) {
  e.stopPropagation();
  let property = this.closest('[data-kind="property"]');
  let name = ast.property.name(property);
  let value = ast.property.value(property).dataset.stringValue;
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

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    for (let property of editor.querySelectorAll('[data-kind="property"]'))
      insert_comment_button(property);
    editor.addEventListener("new-property", ({ detail: { new_property } }) => {
      insert_comment_button(new_property);
    });
  });
});
