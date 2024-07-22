const { invoke: tauri_invoke } = window.__TAURI__.tauri;

export default async function invoke(editor, path, args) {
  try {
    return await tauri_invoke(path, args);
  } catch (e) {
    editor.dispatchEvent(new CustomEvent("invoke-error", { detail: e }));
  }
}
