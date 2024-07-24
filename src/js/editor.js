const { invoke } = window.__TAURI__.tauri;

export async function add_editor(selector, anchor = null) {
  let editor = document.createElement("div");
  editor.classList.add("--editor");
  editor.dataset.selector = selector;
  editor.setAttribute("spellcheck", false);
  let html = await invoke("render_rule", {
    path: localStorage.getItem("current-path"),
    selector,
  });
  editor.innerHTML = html;
  let canvas = document.querySelector(".canvas");
  canvas.append(editor);
  focus(editor);
  canvas.dispatchEvent(
    new CustomEvent("new-editor", { detail: { editor, anchor } }),
  );
}
