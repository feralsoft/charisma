import { num_var, px_var } from "./helpers.js";
import { find } from "./iter.js";

const GRID_SIZE = 25;

function new_group(position) {
  let group = document.createElement("div");
  group.classList.add("--editor-group");

  let pos = snap_position({
    x: position.x - GRID_SIZE,
    y: position.y - GRID_SIZE,
  });

  group.style.setProperty("--x", pos.x);
  group.style.setProperty("--y", pos.y);
  document.querySelector(".canvas").append(group);
  return group;
}

function is_overlapping(group, editor, editor_position) {
  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();
  let editor_top = editor_position.y;
  let editor_left = editor_position.x;
  let editor_right = editor_left + editor_width;
  let editor_bottom = editor_top + editor_height;

  let { width: group_width, height: group_height } =
    group.getBoundingClientRect();
  let group_left = num_var(group, "--x");
  let group_top = num_var(group, "--y");
  let group_bottom = group_top + group_height;
  let group_right = group_left + group_width;

  return (
    group_bottom > editor_top &&
    editor_bottom > group_top &&
    editor_left < group_right &&
    editor_right > group_left
  );
}

function put_in_group(editor, position) {
  // position is top-left
  //
  // and we search for --editor-group with that position
  // but this fails easily in this case
  //
  // |----------------|
  // | dropped-editor |
  // |  |----------------|
  // |  | old  editor    |
  // |  |----------------|
  // |----------------|
  //
  // ^ here I dropped an editor over-top of an existing editor `old editor`
  // but since top left isn't within old editor, it won't snap..
  //
  // intersection observer requires you to register all the elements to watch afaik
  // this seems hard to maintain, so for now we will just loop over all elements & check overlap

  let group =
    find(document.querySelectorAll(".--editor-group"), (group) =>
      is_overlapping(group, editor, position),
    ) ?? new_group(position);

  group.append(editor);
}

const SNAP_OFFSET = 4;

export function snap_position({ x, y }) {
  if (x % GRID_SIZE < 9) x = x - (x % GRID_SIZE) - SNAP_OFFSET;
  else x = x + (GRID_SIZE - (x % GRID_SIZE)) - SNAP_OFFSET;

  if (y % GRID_SIZE < 9) y = y - (y % GRID_SIZE) - SNAP_OFFSET;
  else y = y + (GRID_SIZE - (y % GRID_SIZE)) - SNAP_OFFSET;

  return { x, y };
}

function init(editor) {
  let { width: body_width, height: body_height } =
    document.body.getBoundingClientRect();

  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();

  let x_offset = px_var(document.body, "--x-offset");
  let y_offset = px_var(document.body, "--y-offset");

  put_in_group(editor, {
    x: body_width / 2 - x_offset - editor_width / 2,
    y: body_height / 2 - y_offset - editor_height / 2,
  });
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
    editor.addEventListener("drag-finished", ({ detail: { position } }) =>
      put_in_group(editor, position),
    );
  });
});

window.assert = function (cond, msg) {
  if (!cond) {
    console.error(msg);
    debugger;
    throw msg;
  }
};
