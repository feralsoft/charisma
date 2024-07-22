import { px_var } from "./helpers.js";
import { h } from "./html.js";

let x_offset, y_offset;

function existing_offscreen_for(editor) {
  return document.querySelector(
    `.offscreen-editor-preview[data-selector="${editor.dataset.selector.replaceAll('"', '\\"')}"]`,
  );
}

function preview_offscreen(editor, x, y, placements) {
  let preview;
  if ((preview = existing_offscreen_for(editor))) {
    preview.style.setProperty("--x", x);
    preview.style.setProperty("--y", y);
    preview.dataset.placement = placements.join(" ");
  } else {
    let canvas = document.querySelector(".canvas");
    let preview = h.div({
      class: "offscreen-editor-preview",
      "data-selector": editor.dataset.selector,
      "data-placement": placements.join(" "),
      style: { "--x": x, "--y": y },
    });

    canvas.append(preview);
  }
}

function remove_preview(editor) {
  existing_offscreen_for(editor)?.remove();
}

function preview_offscreen_editors() {
  let new_x_offset = px_var(document.body, "--x-offset"),
    new_y_offset = px_var(document.body, "--y-offset");
  if (new_x_offset === x_offset && new_y_offset === y_offset) return;
  let { width, height } = document.body.getBoundingClientRect();
  x_offset = new_x_offset;
  y_offset = new_x_offset;

  for (let editor of document.querySelectorAll(".--editor")) {
    let { top, left, bottom, right } = editor.getBoundingClientRect();

    let placements = [];

    if (right < 0) placements.push("left");
    if (left > width) placements.push("right");
    if (bottom < 0) placements.push("top");
    if (top > height) placements.push("bottom");
    if (placements.length > 0) preview_offscreen(editor, left, top, placements);
    else remove_preview(editor);
  }
}

window.addEventListener("mousemove", preview_offscreen_editors);
