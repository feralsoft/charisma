import invoke from "./invoke.js";
import * as ast from "./ast.js";

let property_to_copy;

window.addEventListener("keydown", async (e) => {
  if (!document.querySelector(".focused")) return;

  if (e.key === "c" && e.metaKey) {
    // trying to copy
    property_to_copy = document.querySelector('[data-kind="property"].focused');
  } else if (e.key === "v" && e.metaKey) {
    if (!property_to_copy) return;
    let focused_editor = document.querySelector(".--editor.focused");

    // no use, copying into myself.. right?
    if (focused_editor.contains(property_to_copy)) return;

    // ok, lets paste

    let property_name = ast.property.name(property_to_copy);

    let property_value =
      ast.property.value(property_to_copy).dataset.stringValue;

    await invoke(focused_editor, "insert_property", {
      path: localStorage.getItem("current-path"),
      selector: focused_editor.dataset.selector,
      property: `${property_name}: ${property_value};`,
    });
  }
});
