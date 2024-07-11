import { open } from "@tauri-apps/api/dialog.js";
import { homeDir } from "@tauri-apps/api/path.js";
const { invoke } = window.__TAURI__.tauri;

document.addEventListener("DOMContentLoaded", (_) => {
  document
    .querySelector(".file-picker")
    .addEventListener("click", async (e) => {
      let path = await open({
        multiple: false,
        filters: [
          {
            name: "Css",
            extensions: ["css"],
          },
        ],
      });
      await invoke("open_path", { path });
      let home_dir = await homeDir();
      document.body.dataset.current_path = path;
      e.target.innerText = path.replace(home_dir, "");
    });
});
