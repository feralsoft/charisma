const { invoke } = window.__TAURI__.tauri;

window.addEventListener("keydown", async (e) => {
  if (e.key !== "Backspace") return;
  let property = document.querySelector('[data-kind="property"].focused');
  if (!property) return;
  if (property.contains(document.activeElement)) return;
  // can delete
  let name = property.querySelector('[data-attr="name"] > [data-value]').dataset
    .value;
  let value = property.querySelector(
    '[data-attr="value"] > [data-kind][data-string-value]',
  ).dataset.stringValue;
  let editor = property.closest(".--editor");
  await invoke("delete", {
      path: localStorage.getItem("current-path"),
      selector: editor.dataset.selector, name, value });
  editor.dispatchEvent(new Event("reload"));
});
