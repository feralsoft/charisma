import { h } from "./html.js";
import invoke from "./invoke";

const COLOR_SELECTOR =
  '[data-kind="function"]:has(> [data-attr="name"] [data-value="rgb"])';

function hex_to_rgb(hex) {
  let r = hex.slice(1, 3);
  let g = hex.slice(3, 5);
  let b = hex.slice(5, 7);
  return `rgb(${parseInt(r, 16)}, ${parseInt(g, 16)}, ${parseInt(b, 16)})`;
}

function rgb_to_hex(r, g, b) {
  r = Number(r).toString(16).padStart(2, "0");
  g = Number(g).toString(16).padStart(2, "0");
  b = Number(b).toString(16).padStart(2, "0");
  return `#${r}${g}${b}`;
}

function nth_arg_value(fn, n) {
  return fn.querySelector(
    `:scope > [data-attr="args"] [data-kind]:nth-child(${n}) [data-value]`,
  ).dataset.value;
}

function init(editor) {
  for (let color of editor.querySelectorAll(COLOR_SELECTOR)) {
    let picker;
    if (
      (picker = color.parentElement.querySelector(".property-color-picker"))
    ) {
      let r = nth_arg_value(color, 1);
      let g = nth_arg_value(color, 2);
      let b = nth_arg_value(color, 3);
      picker.value = rgb_to_hex(r, g, b);
    } else {
      let name = color
        .closest('[data-kind="property"]')
        .querySelector(':scope > [data-attr="name"] [data-value]')
        .dataset.value;
      let r = nth_arg_value(color, 1);
      let g = nth_arg_value(color, 2);
      let b = nth_arg_value(color, 3);
      let lock = false;
      color.after(
        h.input({
          type: "color",
          class: "property-color-picker",
          value: rgb_to_hex(r, g, b),
          async "@input"(e) {
            if (lock) return;
            let color = this.parentElement.querySelector(COLOR_SELECTOR);
            await invoke(editor, "update_value", {
              path: localStorage.getItem("current-path"),
              selector: editor.dataset.selector,
              name,
              original_value: color.dataset.stringValue,
              value: hex_to_rgb(e.target.value),
            });
            lock = false;
          },
        }),
      );
    }
  }
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    init(editor);
    editor.addEventListener("loaded", () => init(editor));
  });
});
