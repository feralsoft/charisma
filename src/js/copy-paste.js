import { add_editor } from "./editor.js";

const { invoke } = window.__TAURI__.tauri;

// paste from out of the app
window.addEventListener("paste", async (e) => {
  if (document.querySelector(".focused")) return;
  let rule = e.clipboardData.getData("text");
  if (!rule) return;
  try {
    let selector = await invoke("load_rule", {
      path: localStorage.getItem("current-path"),
      rule,
    });
    await add_editor(selector);
  } catch (e) {
    console.error(e);
  }
});
