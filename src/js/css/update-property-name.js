import * as ast from "./ast.js";
import invoke from "../invoke.js";

function setup_property(property, editor) {
  property
    .querySelector(':scope > [data-attr="name"] > [data-value]')
    .addEventListener("dblclick", function (_) {
      let original_name = this.dataset.value;
      this.setAttribute("contenteditable", true);
      window.getSelection().selectAllChildren(this);
      this.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
          this.contenteditable = false;
        } else if (e.key === "Enter") {
          this.blur();
        }
      });
      this.addEventListener(
        "blur",
        async (_) => {
          await invoke(editor, "rename_property", {
            path: localStorage.getItem("current-path"),
            selector: editor.dataset.selector,
            is_commented: property.dataset.commented === "true",
            old_property_name: original_name,
            new_property_name: this.innerText.trim(),
            property_value: ast.property.value(property).dataset.stringValue,
          });
          this.setAttribute("contenteditable", false);
        },
        { once: true },
      );
    });
}

function init(editor) {
  for (let property of ast.rule.properties(editor))
    setup_property(property, editor);
  editor.addEventListener("new-property", ({ detail: { new_property } }) =>
    setup_property(new_property, editor),
  );
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    init(editor);
  });
});
