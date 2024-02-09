import compile from "@coil-lang/compiler/";
import { defineConfig } from "vite";
import fs from "fs/promises";
import * as prettier from "prettier";

function coil() {
  return {
    name: "coil-lang",
    transform(src, id) {
      if (id.endsWith(".coil")) {
        return {
          code: compile(src, "@coil-lang/compiler/"),
          map: null,
        };
      }
    },
  };
}

// TODO: ignore hot-reload of css file while updating it
function updateIndexCss() {
  return {
    name: "index-css",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url === "/update_css") {
          let body = "";
          req.on("data", (b) => (body += b.toString()));
          req.on("end", async () => {
            let { css, file } = JSON.parse(body);
            css = await prettier.format(css, { semi: false, parser: "css" });
            fs.writeFile(file, css);
            next();
          });
        }
        next();
      });
    },
  };
}

export default defineConfig({
  plugins: [coil(), updateIndexCss()],
  build: { target: "esnext" },
});
