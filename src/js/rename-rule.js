import { add_editor } from "./editor.js";
import { h, modifiers } from "./html.js";
import { move_cursor_to_end_of_element } from "./utils/contenteditable.js";

const { invoke } = window.__TAURI__.tauri;

function init(editor) {
  let selector = editor.querySelector('[data-attr="selector"]');

  selector.addEventListener("dblclick", (_) => {
    let query = selector.querySelector("[data-kind]");
    let cloned = query.cloneNode(true);
    let string = query.dataset.stringValue;

    query.replaceWith(
      h.div(
        {
          class: "plain-text-node",
          contenteditable: true,
          "@keydown"(e) {
            if (e.key === "Enter") {
              this.blur();
            } else if (e.key === "Escape") {
              this.replaceWith(cloned);
            }
          },
          async "@blur"(_) {
            let new_selector = this.textContent;
            // TODO: implement a way to revert from this
            await invoke("rename_rule", {
              path: localStorage.getItem("current-path"),
              old_selector: string,
              new_selector,
            });
            await add_editor(new_selector, editor.closest(".--editor-group"));
            editor.remove();
          },
          [modifiers.on_mount]() {
            move_cursor_to_end_of_element(this);
          },
        },
        string,
      ),
    );
  });
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");

  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    init(editor);
  });
});
