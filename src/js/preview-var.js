const VAR_ID_SELECTOR = `[data-kind="function"]:has(> [data-attr="name"] [data-value="var"])
    > [data-attr="args"] [data-kind="dashed-id"]`;

function init(editor) {
  for (let variable of editor.querySelectorAll(VAR_ID_SELECTOR)) {
    let name = variable.querySelector("[data-value]").dataset.value;
    let property = editor.querySelector(
      `[data-kind="property"][data-property-kind="variable"][data-commented="false"]:has([data-attr="name"] [data-value="${name}"])`,
    );
    if (!property) {
      variable.classList.add("cant-deref");
      return;
    }
    let value = property.querySelector('[data-attr="value"] [data-kind]');
    let cloned_value = value.cloneNode(true);
    cloned_value.classList.add("preview");
    variable.append(cloned_value);
  }
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
