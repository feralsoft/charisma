import "./js/css/manipulate-numbers.js";
import "./js/css/delete-property.js";
import "./js/drag-editor.js";
import "./js/drag-group.js";
import "./js/focus.js";
import "./js/css/insert-property.js";
import "./js/menu.js";
import "./js/multi-editor.js";
import "./js/css/search.js";
import "./js/css/toggle-property.js";
import "./js/undo.js";
import "./js/css/update-value.js";
import "./js/group-menu.js";
import "./js/drag-board.js";
import "./js/css/copy-property.js";
import "./js/css/choose-file.js";
import "./js/css/copy-paste.js";
import "./js/display-error.js";
import "./js/reload.js";
import "./js/off-screen.js";
import "./js/css/color-picker.js";
import "./js/css/find-property.js";
import "./js/css/rename-rule.js";
import "./js/css/update-property-name.js";

import { appWindow } from "./node_modules/@tauri-apps/api/window.js";

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
