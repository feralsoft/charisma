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

function morph_value(old_value, new_value) {
  assert(new_value);
  old_value.dataset.value = new_value.dataset.value;
  old_value.innerHTML = new_value.innerHTML;
}

function morph_node(old_node, new_node) {
  if (old_node.dataset.kind !== new_node.dataset.kind) {
    old_node.replaceWith(new_node);
  } else {
    old_node.dataset.stringValue = new_node.dataset.stringValue;
    let old_value = old_node.querySelector(":scope > [data-value]");
    if (old_value) {
      let new_value = new_node.querySelector(":scope > [data-value]");
      assert(new_value);
      morph_value(old_value, new_value);
    } else {
      for (let attr of old_node.querySelectorAll(":scope > [data-attr]")) {
        let old_node = attr.querySelector(":scope > [data-kind]");
        if (old_node) {
          let new_sub_node = new_node.querySelector(
            `:scope > [data-attr="${attr.dataset.attr}"] > [data-kind]`,
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
      }
    }
  }
}

function update_property_values(editor, new_properties) {
  for (let new_property of new_properties) {
    let name = ast.property.name(new_property);
    let existing_property = editor.querySelector(`
      [data-kind="property"]:has(>
        [data-attr="name"] [data-value="${name}"]
      )
    `);

    if (!existing_property) continue;
    let existing_value = ast.property.value(existing_property);
    let new_value = ast.property.value(new_property);
    if (existing_value.dataset.stringValue !== new_value.dataset.stringValue) {
      morph_node(existing_value, new_value);
    }
  }
}

function update_commented_properties(existing_properties, new_properties) {
  for (let property of existing_properties) {
    let name = ast.property.name(property);
    let new_property = new_properties.find(
      (p) => ast.property.name(p) === name,
    );
    property.dataset.commented = new_property.dataset.commented;
  }
}

function rejuvenate_editor(existing_editor, new_rule_html) {
  let updated_rule = document.createElement("div");
  updated_rule.innerHTML = new_rule_html;
  assert(no_duplicates(updated_rule));

  let existing_properties = ast.rule.properties(existing_editor);
  let new_properties = ast.rule.properties(updated_rule);
  remove_deleted_properties(existing_properties, new_properties);
  // update after delete
  existing_properties = ast.rule.properties(existing_editor);
  update_property_values(existing_editor, new_properties);
  insert_new_properties(
    existing_editor.querySelector('[data-attr="properties"]'),
    new_properties,
  );
  update_commented_properties(existing_properties, new_properties);
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
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    editor.addEventListener("reload", reload);
  });
});
