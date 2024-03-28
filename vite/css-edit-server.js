import fs from "fs/promises";
import * as prettier from "prettier";

let css_db_path = "./css";

async function save_to_disk(path, code) {
  let dir = `${css_db_path}/${path.join("/")}`;
  await fs.mkdir(dir, { recursive: true });
  await fs.writeFile(`${dir}/index.css`, code, {});
}

// TODO: ignore hot-reload of css file while updating it
export function update_local_css_files() {
  return {
    name: "index-css",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url === "/save_to_file") {
          let body = "";
          req.on("data", (b) => (body += b.toString()));
          req.on("end", async () => {
            let { path, code } = JSON.parse(body);
            await save_to_disk(path, code);
          });
          next();
        } else if (req.url === "/update_css") {
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
