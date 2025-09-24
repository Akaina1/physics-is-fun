// scripts/build-wasm.js
import { readdirSync } from "fs";
import { execSync } from "child_process";
import path from "path";

const kernelsDir = path.join(process.cwd(), "kernels");
const crates = readdirSync(kernelsDir, { withFileTypes: true })
  .filter(d => d.isDirectory());

for (const c of crates) {
  const dir = path.join(kernelsDir, c.name);
  console.log(`Building ${c.name}...`);
  execSync("wasm-pack build --target bundler --out-dir pkg", { cwd: dir, stdio: "inherit" });
}
