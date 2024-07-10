import { h } from "./html.js";

let close = (group) =>
  h.button({ class: "close", "@click": (_) => group.remove() });

let minimize = () =>
  h.button({
    class: "minimize",
    "@click"() {
      this.classList.toggle("active");
    },
  });

let menu = (group) => h.div({ class: "menu" }, close(group), minimize());

function init(group) {
  if (group.querySelector(":scope > .menu")) return;
  group.prepend(menu(group));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor.closest(".--editor-group"));
    editor.addEventListener("drag-finished", () =>
      init(editor.closest(".--editor-group")),
    );
  });
});
