const { invoke } = window.__TAURI__.tauri;

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

    let property_name = property_to_copy.querySelector(
      ':scope > [data-attr="name"] [data-value]',
    ).dataset.value;

    let property_value = property_to_copy.querySelector(
      ':scope > [data-attr="value"] > [data-kind][data-string-value]',
    ).dataset.stringValue;

    await invoke("insert_property", {
      selector: focused_editor.dataset.selector,
      property: `${property_name}: ${property_value};`,
    });
    focused_editor.dispatchEvent(new Event("reload", { bubbles: true }));
  }
});
