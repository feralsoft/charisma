import * as ast from "./css/ast.js";
const { invoke } = window.__TAURI__.tauri;

let reload_lock = new Map();

function remove_deleted_properties(editor, new_rule) {
  let new_names = new Set(
    Array.from(new_rule.querySelectorAll('[data-kind="property"]')).map(
      ast.property.name,
    ),
  );
  for (let property of editor.querySelectorAll('[data-kind="property"]')) {
    let name = ast.property.name(property);
    if (property.dataset.commented === "true") {
      // if the property is commented out & then we change the value or remove it, we'll just remove it from the editor
      // and in `insert_property` it'll be added back if it was uncommented + changed value
      let property_in_new_rule =
        new_rule.querySelector(`[data-kind="property"][data-commented="true"]:has(
        > [data-attr="name"] [data-value="${CSS.escape(name)}"]
        ):has(> [data-attr="value"] > [data-string-value="${CSS.escape(
          ast.property.value(property).dataset.stringValue,
        )}"])`);
      if (!property_in_new_rule) property.remove();
    } else {
      let uncommented_property_with_the_same_name = new_rule.querySelector(
        `[data-kind="property"][data-commented="false"]:has(> [data-attr="name"] [data-value="${CSS.escape(name)}"])`,
      );

      if (!uncommented_property_with_the_same_name) property.remove();
    }
  }
}

function insert_property(editor, new_property) {
  let new_property_name = ast.property.name(new_property);
  let properties = editor.querySelectorAll('[data-kind="property"]');

  for (let i = 0; i < properties.length - 1; i++) {
    let current = properties[i];
    let next = properties[i + 1];

    if (
      new_property_name >= ast.property.name(current) &&
      new_property_name <= ast.property.name(next)
    ) {
      // insert at the ordered location
      current.after(new_property);
      return;
    }
  }

  if (properties[properties.length - 1])
    properties[properties.length - 1].after(new_property);
  else editor.querySelector('[data-attr="properties"]').append(new_property);
}

function insert_new_properties(editor, new_properties) {
  for (let new_property of new_properties) {
    let existing_property = editor.querySelector(`
      [data-kind="property"][data-commented="${new_property.dataset.commented}"]:has(
        > [data-attr="name"] [data-value="${CSS.escape(ast.property.name(new_property))}"]
      ):has(> [data-attr="value"] > [data-string-value="${CSS.escape(
        ast.property.value(new_property).dataset.stringValue,
      )}"])`);
    if (!existing_property) {
      register_event(editor, "new-property", { new_property });
      insert_property(editor, new_property);
    }
  }
}

// we can't dispatch events during morphing/updating since it might happen
// before all the updates have been applied
let _event_queue = [];
function register_event(node, kind, detail = {}) {
  _event_queue.push({ node, kind, detail });
}
function exec_events() {
  for (let { node, kind, detail } of _event_queue)
    node.dispatchEvent(new CustomEvent(kind, { detail }));
  _event_queue = [];
}

function morph_value(old_value, new_value) {
  assert(new_value);
  register_event(old_value, "reload-value");
  old_value.dataset.value = new_value.dataset.value;
  old_value.innerHTML = new_value.innerHTML;
}

function morph_node(old_node, new_node) {
  if (old_node.dataset.kind !== new_node.dataset.kind) {
    register_event(old_node.parentElement, "reload-value");
    old_node.replaceWith(new_node.cloneNode(true));
  } else {
    register_event(old_node, "reload-value");
    old_node.dataset.stringValue = new_node.dataset.stringValue;
    let old_value = old_node.querySelector(":scope > [data-value]");
    if (old_value) {
      let new_value = new_node.querySelector(":scope > [data-value]");
      assert(new_value);
      morph_value(old_value, new_value);
    } else {
      for (let attr of old_node.querySelectorAll(":scope > [data-attr]")) {
        let n = 1;
        for (let old_node of attr.querySelectorAll(":scope > [data-kind]")) {
          if (old_node) {
            let new_sub_node = new_node.querySelector(
              `:scope > [data-attr="${attr.dataset.attr}"] > [data-kind]:nth-child(${n})`,
            );
            assert(new_sub_node);
            morph_node(old_node, new_sub_node);
          } else {
            let old_value = attr.querySelector(":scope > [data-value]");
            assert(old_value);
            let new_value = new_value.querySelector(
              `:scope > [data-attr="${attr.dataset.attr}"] > [data-value]`,
            );
            assert(new_value);
            morph(old_value, new_value);
          }
          n++;
        }
      }
    }
  }
}

function update_property_values(editor, new_properties) {
  for (let new_property of new_properties) {
    if (new_property.dataset.commented === "true") continue;
    let name = ast.property.name(new_property);
    let existing_property = editor.querySelector(`
      [data-kind="property"][data-commented="false"]:has(>
        [data-attr="name"] [data-value="${name}"]
      )
    `);

    if (!existing_property) continue;
    let existing_value = ast.property.value(existing_property);
    let new_value = ast.property.value(new_property);
    if (existing_value.classList.contains("plain-text-node")) {
      register_event(existing_property, "reload-value");
      existing_value.replaceWith(new_value.cloneNode(true));
    } else if (
      existing_value.dataset.stringValue !== new_value.dataset.stringValue
    ) {
      register_event(existing_property, "reload-value");
      morph_node(existing_value, new_value);
    }
  }
}

function update_comment_status(editor, updated_rule) {
  for (let property of editor.querySelectorAll('[data-kind="property"]')) {
    let name = ast.property.name(property);
    let string_value = ast.property.value(property).dataset.stringValue;
    let new_property = updated_rule.querySelector(`
      [data-kind="property"]:has(
        > [data-attr="name"] [data-value="${CSS.escape(name)}"]
      ):has(
        > [data-attr="value"] > [data-string-value="${CSS.escape(string_value)}"]
      )
    `);

    if (!new_property) continue;
    property.dataset.commented = new_property.dataset.commented;
  }
}

/*
OLD:
  display: flex;
  gap: 1px;

NEW:
  display: flex;
  // gap: 1px;
  gap: 2px;

*/

function no_more_than_1_property_of_a_name_allowed_to_be_uncommented(editor) {
  let count = {};
  for (let property of editor.querySelectorAll('[data-kind="property"]')) {
    if (property.dataset.commented === "false") {
      let name = ast.property.name(property);
      if (name in count) return false;
      count[name] = 1;
    }
  }
  return true;
}

function rejuvenate_editor(existing_editor, new_rule_html) {
  let updated_rule = document.createElement("div");
  updated_rule.innerHTML = new_rule_html;

  let new_properties = ast.rule.properties(updated_rule);
  remove_deleted_properties(existing_editor, updated_rule);
  update_comment_status(existing_editor, updated_rule);
  assert(
    no_more_than_1_property_of_a_name_allowed_to_be_uncommented,
    "more than 1 property of a given name is uncommented, that is no good",
  );
  // now we can update values since there is only allowed 1 uncommented value per name at a time
  update_property_values(existing_editor, new_properties);
  insert_new_properties(existing_editor, new_properties);
  exec_events();
}

async function reload() {
  let editor = this;
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

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    editor.addEventListener("reload", reload);
  });
});
