import { h } from "./html.js";

let close = () =>
  h.button({
    class: "close",
    "@click"(_) {
      let editor = this.closest(".--editor");
      let group = editor.closest(".--editor-group");
      editor.remove();
      if (!group.querySelector(".--editor")) group.remove();
    },
  });

let minimize = () =>
  h.button({
    class: "minimize",
    "@click"(_) {
      this.classList.toggle("active");
      if (this.classList.contains("active")) {
        let editor = this.closest(".--editor");
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

function init(editor) {
  editor.prepend(
    h.menu({}, ...[close(), minimize()].map((btn) => h.li({}, btn))),
  );
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
  });
});
