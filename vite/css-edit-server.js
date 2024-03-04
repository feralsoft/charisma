import fs from "fs/promises";
import * as prettier from "prettier";

// TODO: ignore hot-reload of css file while updating it
export function update_local_css_files() {
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
        // why not else?
        next();
      });
    },
  };
}
