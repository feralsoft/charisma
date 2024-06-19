function init(editor) {
  let name = new URL(
    `${location.host}/src/${decodeURIComponent(editor.dataset.url).replaceAll("+", " ")}`,
  ).searchParams.get("highlight_property_name");
  if (!name) return;
  let elem = editor.querySelector(
    `[data-kind="property"]:has(> [data-attr="name"] > [data-value="${name}"])`,
  );
  elem.classList.add("highlighted");
  elem.scrollIntoView();
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let editor of document.querySelectorAll(".--editor")) {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  }
});
