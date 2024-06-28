import "./js/delete_property.js";
import "./js/draggable_editor.js";
import "./js/focus.js";
import "./js/insert_property.js";
import "./js/menu.js";
import "./js/multi_editor.js";
import "./js/preview_var.js";
import "./js/search.js";
import "./js/toggle_property.js";
import "./js/undo.js";
import "./js/update_value.js";
import "./js/tab-cycle.js";

import { appWindow } from "https://unpkg.com/@tauri-apps/api/window";

document
  .getElementById("titlebar-minimize")
  .addEventListener("click", () => appWindow.minimize());
document
  .getElementById("titlebar-maximize")
  .addEventListener("click", () => appWindow.setFullscreen(true));
document
  .getElementById("titlebar-close")
  .addEventListener("click", () => appWindow.close());
