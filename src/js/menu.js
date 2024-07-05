import { h } from "./html.js";

let close = (editor) =>
  h("button", {
    class: "close",
    "@click"(_) {
      let group = editor.closest(".--editor-group");
      editor.remove();
      if (!group.querySelector(".--editor")) group.remove();
    },
  });

let minimize = (editor) =>
  h("button", {
    class: "minimize",
    "@click"(_) {
      this.dataset.selector = editor.querySelector(
        '[data-attr="selector"] > [data-kind]',
      ).dataset.stringValue;
      this.classList.toggle("active");
    },
  });

let info = () =>
  h("button", {
    class: "info",
    "@click"(_) {
      this.classList.toggle("active");
    },
  });

let menu = (editor) =>
  h("div", { class: "menu" }, close(editor), minimize(editor), info());

function init(editor) {
  editor.prepend(menu(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
