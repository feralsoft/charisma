import { focus } from "../focus.js";
import invoke from "../invoke.js";
import * as ast from "./ast.js";

window.addEventListener("keydown", async (e) => {
  if (e.key !== "Backspace") return;
  let property = document.querySelector('[data-kind="property"].focused');
  if (!property) return;
  if (property.contains(document.activeElement)) return;
  // can delete
  let editor = property.closest(".--editor");
  await invoke(editor, "delete", {
    path: localStorage.getItem("current-path"),
    selector: editor.dataset.selector,
    name: ast.property.name(property),
    value: ast.property.value(property).dataset.stringValue,
  });
  focus(editor);
});
