import { h } from "./html.js";
import * as ast from "./ast.js";
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

function hex(color) {
  return rgb_to_hex(
    nth_arg_value(color, 1),
    nth_arg_value(color, 2),
    nth_arg_value(color, 3),
  );
}

function init(editor) {
  for (let color of editor.querySelectorAll(COLOR_SELECTOR)) {
    color
      .closest('[data-kind="property"]')
      .addEventListener("reload-value", function (_) {
        color = this.querySelector(`[data-attr="value"] ${COLOR_SELECTOR}`);
        this.querySelector(".property-color-picker").value = hex(color);
      });
    let lock = false;
    color.after(
      h.input({
        type: "color",
        class: "property-color-picker",
        value: hex(color),
        async "@input"(e) {
          if (lock) return;
          await invoke(editor, "update_value", {
            path: localStorage.getItem("current-path"),
            selector: editor.dataset.selector,
            name: ast.property.name(color.closest('[data-kind="property"]')),
            original_value: color.dataset.stringValue,
            value: hex_to_rgb(e.target.value),
          });
          lock = false;
        },
      }),
    );
  }
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    init(editor);
  });
});
