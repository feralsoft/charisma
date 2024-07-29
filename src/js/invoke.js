import { once } from "./dom.js";

const { invoke: tauri_invoke } = window.__TAURI__.tauri;

export default async function invoke(editor, path, args) {
  try {
    let result = await tauri_invoke(path, args);
    let loaded = new Promise((r) => once(editor, "loaded", r));
    editor.dispatchEvent(new Event("reload"));
    await loaded;
    return result;
  } catch (e) {
    editor.dispatchEvent(new CustomEvent("invoke-error", { detail: e }));
  }
}
