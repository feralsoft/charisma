import compile from "@coil-lang/compiler/";
import { defineConfig } from "vite";
const fileRegex = /\.(coil)$/;

function coil() {
  return {
    name: "coil-lang",
    transform(src, id) {
      if (fileRegex.test(id)) {
        return {
          code: compile(src, "@coil-lang/compiler/"),
          map: null,
        };
      }
    },
  };
}

export default defineConfig({
  plugins: [coil()],
  build: { target: "esnext" },
});
