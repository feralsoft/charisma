import { num_var } from "./helpers.js";
import { find, zip } from "./iter.js";
import * as ast from "./ast.js";

const { invoke } = window.__TAURI__.tauri;

let reload_lock = new Map();

function remove_deleted_properties(old_properties, new_properties) {
  let new_names = new Set(new_properties.map(ast.property.name));
  for (let deleted_property of old_properties.filter(
    (property) => !new_names.has(ast.property.name(property)),
  )) {
    deleted_property.remove();
  }
}

function insert_property(properties_elem, property_list, new_property) {
  let new_property_name = ast.property.name(new_property);
  for (let i = 0; i < property_list.length - 1; i++) {
    let current = property_list[i];
    let next = property_list[i + 1];
    if (new_property_name >= current && new_property_name <= next) {
      // insert at the ordered location
      current.after(new_property);
      return;
    }
  }
  // otherwise insert at the end
  properties_elem.append(new_property);
}

function insert_new_properties(existing_properties_elem, new_properties) {
  let existing_properties = Array.from(
    existing_properties_elem.querySelectorAll('[data-kind="property"]'),
  );
  let existing_names = new Set(existing_properties.map(ast.property.name));

  for (let new_property of new_properties) {
    if (!existing_names.has(ast.property.name(new_property))) {
      insert_property(
        existing_properties_elem,
        existing_properties,
        new_property,
      );
    }
  }
}

function no_duplicates(rule) {
  return ast.rule
    .properties(rule)
    .map(ast.property.name)
    .every(
      (name, i, self) =>
        !self.slice(0, i).includes(name) && !self.slice(i + 1).includes(name),
    );
}

function update_property_values(editor, new_properties) {
  for (let new_property of new_properties) {
    let name = ast.property.name(new_property);
    let existing_property = editor.querySelector(
      '[data-kind="property"]:has(> [data-attr="name"] [data-value="' +
        name +
        '"]',
    );
    if (!existing_property) continue;
    if (
      ast.property.value(existing_property) !== ast.property.value(new_property)
    ) {
      existing_property.replaceWith(new_property);
    }
  }
}

function rejuvenate_editor(existing_editor, new_rule_html) {
  let updated_rule = document.createElement("div");
  updated_rule.innerHTML = new_rule_html;
  assert(no_duplicates(updated_rule));

  let existing_properties = ast.rule.properties(existing_editor);
  let new_properties = ast.rule.properties(updated_rule);
  remove_deleted_properties(existing_properties, new_properties);
  update_property_values(existing_editor, new_properties);
  insert_new_properties(
    existing_editor.querySelector('[data-attr="properties"]'),
    new_properties,
  );
}

async function reload(editor) {
  if (reload_lock.get(editor)) return;
  reload_lock.set(editor, true);
  for (let editor_ of document.querySelectorAll(".--editor")) {
    if (editor_ === editor) continue;
    editor_.dispatchEvent(
      new CustomEvent("reload", { detail: { src: "reload-siblings" } }),
    );
  }

  let new_rule_html_string = await invoke("render_rule", {
    path: localStorage.getItem("current-path"),
    selector: editor.dataset.selector,
  });
  rejuvenate_editor(editor, new_rule_html_string);
  editor.dispatchEvent(new Event("loaded"));
  reload_lock.delete(editor);
}

function new_group(position) {
  let group = document.createElement("div");
  group.classList.add("--editor-group");

  // - 25 since the group has a padding of 25px
  let pos = snap_position({ x: position.x - 25, y: position.y - 25 });

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
  if (x % 25 < 9) x = x - (x % 25) - SNAP_OFFSET;
  else x = x + (25 - (x % 25)) - SNAP_OFFSET;

  if (y % 25 < 9) y = y - (y % 25) - SNAP_OFFSET;
  else y = y + (25 - (y % 25)) - SNAP_OFFSET;

  return { x, y };
}

function init(editor) {
  let { width: body_width, height: body_height } =
    document.body.getBoundingClientRect();

  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();

  put_in_group(editor, {
    x: body_width / 2 - editor_width / 3,
    y: body_height / 2 - editor_height / 3,
  });
  editor.addEventListener("reload", (_) => reload(editor));
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
  }
};
