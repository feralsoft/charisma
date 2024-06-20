function init(editor) {
  let name = new URL(editor.dataset.url).searchParams.get(
    "highlight_property_name",
  );
  if (!name) return;
  let elem = editor.querySelector(
    `[data-kind="property"]:has(> [data-attr="name"] > [data-value="${name}"])`,
  );
  elem.classList.add("highlighted");
  elem.scrollIntoView();
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
