const { invoke } = window.__TAURI__.tauri;

class HistoryStack {
  static MAX_SIZE = 1000;
  _stack = [];
  is_empty() {
    return this._stack.length === 0;
  }
  push(editor) {
    if (this._stack.length === HistoryStack.MAX_SIZE) this._stack.shift();
    this._stack.push({
      selector: editor
        .querySelector(
          '[data-attr="selector"] > [data-kind][data-string-value]',
        )
        .dataset.stringValue.trim(),
      properties: get_properties(editor),
    });
  }
  pop() {
    return this._stack.pop();
  }
  drain() {
    this._stack = [];
  }
}

let undo_stack = new HistoryStack();
let redo_stack = new HistoryStack();

const UNDO_SRC_TO_IGNORE = ["undo", "reload-siblings"];

function init(editor) {
  editor.addEventListener("reload", (e) => {
    if (UNDO_SRC_TO_IGNORE.includes(e.detail?.src)) return;
    redo_stack.drain();
    undo_stack.push(editor);
  });
}

function get_properties(editor) {
  let properties = [];
  for (let property of editor.querySelectorAll(
    '[data-attr="properties"] > [data-kind="property"]',
  )) {
    let is_commented = property.dataset.commented === "true";
    let name = property.querySelector('[data-attr="name"] > [data-value]')
      .dataset.value;
    let value = property.querySelector(
      '[data-attr="value"] > [data-kind][data-string-value]',
    ).dataset.stringValue;
    properties.push({ is_commented, name, value });
  }
  return properties;
}

window.addEventListener("keydown", async (e) => {
  if (e.key === "z" && e.metaKey && e.shiftKey) {
    if (redo_stack.is_empty()) return;
    let { selector, properties } = redo_stack.pop();
    let editor = document.querySelector(
      `.--editor:has([data-attr='selector'] > [data-string-value*='${selector}']`,
    );
    undo_stack.push(editor);
    await invoke("replace_all_properties", { selector, properties });
    editor.dispatchEvent(
      new CustomEvent("reload", { detail: { src: "undo" } }),
    );
  } else if (e.key === "z" && e.metaKey) {
    if (undo_stack.is_empty()) return;
    let { selector, properties } = undo_stack.pop();
    let editor = document.querySelector(
      `.--editor:has([data-attr='selector'] > [data-string-value*='${selector}']`,
    );
    redo_stack.push(editor);
    await invoke("replace_all_properties", { selector, properties });
    editor.dispatchEvent(
      new CustomEvent("reload", { detail: { src: "undo" } }),
    );
  }
});

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});
