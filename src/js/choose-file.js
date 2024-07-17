import { open } from "../node_modules/@tauri-apps/api/dialog.js";
import { homeDir } from "../node_modules/@tauri-apps/api/path.js";

async function set_path_text(path, file_picker) {
  let home_dir = await homeDir();
  document.body.dataset.current_path = path;
  file_picker.innerText = path.replace(home_dir, "~/");
}

async function prompt_file_selector(file_picker) {
  let path = await open({
    multiple: false,
    filters: [
      {
        name: "Css",
        extensions: ["css"],
      },
    ],
  });
  localStorage.setItem("current-path", path);
  await set_path_text(path, file_picker);
}

document.addEventListener("DOMContentLoaded", (_) => {
  let file_picker = document.querySelector(".file-picker");

  file_picker.addEventListener("click", (_) =>
    prompt_file_selector(file_picker),
  );

  if (!localStorage.getItem("current-path")) prompt_file_selector(file_picker);
  else
    file_picker.innerHTML = set_path_text(
      localStorage.getItem("current-path"),
      file_picker,
    );
});
