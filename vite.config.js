import { defineConfig } from "vite";
import coil from "./vite/coil";
import { update_local_css_files } from "./vite/css-edit-server";

export default defineConfig({
  plugins: [coil(), update_local_css_files()],
  build: { target: "esnext" },
});
