async function toggle() {
  let property = this.closest('[data-kind="property"]');
  let name = property.querySelector("[data-attr=name]").textContent;
  let is_commented = property.dataset.commented === "true";
  let action = is_commented ? "enable" : "disable";
  let editor = property.closest(".--editor");
  await fetch(url_for(editor, `/${name}/${action}`), { method: "POST" });
  property.dispatchEvent(new Event("reload", { bubbles: true }));
}

function insert_comment_button(src) {
  let button = document.createElement("button");
  button.innerHTML = "<div class='text'></div>";
  button.classList.add("toggle-comment");
  button.addEventListener("mousedown", toggle);
  src.prepend(button);
}

const PROPERTY_SELECTOR = "[data-attr=properties] > [data-kind=property]";

function init(editor) {
  for (let property of editor.querySelectorAll(PROPERTY_SELECTOR))
    insert_comment_button(property);
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
