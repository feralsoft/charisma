import fs from "fs/promises";
import { parse_rule } from "./parser/css-parser.js";
import { run } from "./parser/parse-utils.js";

export async function render(req, next, res, url) {
  let file = (await fs.readFile(`./css/${url}/index.css`)).toString();
  let ast = parse_rule[run](file)[0];
  console.log(ast);
  res.end(`
  <pre>
    INFO:
    url: ${url}
    file: ${file}
  </pre>

  <div>
  </div>  
  `);
}
