import { h } from "./html.js";

let close = (editor) =>
  h.button({
    class: "close",
    "@click"(_) {
      let group = editor.closest(".--editor-group");
      editor.remove();
      if (!group.querySelector(".--editor")) group.remove();
    },
  });

let minimize = (editor) =>
  h.button({
    class: "minimize",
    "@click"(_) {
      this.classList.toggle("active");
      if (this.classList.contains("active")) {
        let selector = editor.querySelector(
          '[data-attr="selector"] > [data-kind]',
        ).dataset.stringValue;
        this.closest("menu").append(
          h.div({ class: "selector-preview" }, selector),
        );
      } else {
        this.closest("menu")
          .querySelector(":scope > .selector-preview")
          .remove();
      }
    },
  });

let menu = (editor) => h.menu({}, close(editor), minimize(editor));

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
