const VAR_ID_SELECTOR = `[data-kind="function"]:has(> [data-attr="name"] [data-value="var"])
    > [data-attr="args"] [data-kind="dashed-id"]`;

function init(editor) {
  for (let variable of editor.querySelectorAll(VAR_ID_SELECTOR)) {
    let name = variable.querySelector("[data-value]").dataset.value;
    let property = editor.querySelector(
      `[data-kind="property"][data-property-kind="variable"][data-commented="false"]:has([data-attr="name"] [data-value="${name}"])`,
    );
    if (!property) {
      console.warn(`can't dereference ${name}`);
      return;
    }
    let value = property.querySelector('[data-attr="value"] [data-kind]');
    let cloned_value = value.cloneNode(true);
    cloned_value.classList.add("preview");
    for (let node of cloned_value.querySelectorAll("[contenteditable]"))
      node.removeAttribute("contenteditable");
    variable.append(cloned_value);
  }
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let editor of document.querySelectorAll(".--editor")) {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  }
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});
