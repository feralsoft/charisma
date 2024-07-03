import "./js/delete-property.js";
import "./js/draggable-editor.js";
import "./js/focus.js";
import "./js/insert-property.js";
import "./js/menu.js";
import "./js/multi-editor.js";
import "./js/preview-var.js";
import "./js/search.js";
import "./js/tab-cycle.js";
import "./js/toggle-property.js";
import "./js/undo.js";
import "./js/update-value.js";
import "./js/group-menu.js";

import { appWindow } from "https://unpkg.com/@tauri-apps/api/window";

appWindow.onResized(async (_) => {
  document.body.dataset.isFullscreen = await appWindow.isFullscreen();
});

document
  .getElementById("titlebar-minimize")
  .addEventListener("click", () => appWindow.minimize());
document.getElementById("titlebar-maximize").addEventListener("click", () => {
  document.body.dataset.isFullscreen = true;
  appWindow.setFullscreen(true);
});
document
  .getElementById("titlebar-close")
  .addEventListener("click", () => appWindow.close());
