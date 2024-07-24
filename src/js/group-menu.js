import { h } from "./html.js";

let close = () =>
  h.button({
    class: "close",
    "@click"(_) {
      this.closest(".--editor-group").remove();
    },
  });

let minimize = () =>
  h.button({
    class: "minimize",
    "@click"() {
      this.classList.toggle("active");
    },
  });

let menu = () =>
  h.menu({}, ...[close(), minimize()].map((btn) => h.li({}, btn)));

function init(group) {
  if (group.querySelector(":scope > menu")) return;
  group.prepend(menu());
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    init(editor.closest(".--editor-group"));
    editor.addEventListener("drag-finished", () =>
      init(editor.closest(".--editor-group")),
    );
  });
});
