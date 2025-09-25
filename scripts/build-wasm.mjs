// scripts/build-wasm.mjs (ESM)
// Builds every Rust crate under /kernels with wasm-pack (target: bundler)

import { readdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { spawn } from 'node:child_process';

const kernelsDir = join(process.cwd(), 'kernels');
const crates = readdirSync(kernelsDir, { withFileTypes: true }).filter((d) =>
  d.isDirectory()
);

if (crates.length === 0) {
  console.log('No kernels/* crates found. Skipping.');
  process.exit(0);
}

function run(cmd, args, cwd) {
  return new Promise((resolve, reject) => {
    // shell:true helps on Windows to resolve wasm-pack(.cmd)
    const child = spawn(cmd, args, { cwd, stdio: 'inherit', shell: true });
    child.on('close', (code) =>
      code === 0 ? resolve() : reject(new Error(`${cmd} exited ${code}`))
    );
  });
}

for (const c of crates) {
  const dir = join(kernelsDir, c.name);
  const pkgDir = join(dir, 'pkg');

  // Clean old /pkg (optional)
  try {
    rmSync(pkgDir, { recursive: true, force: true });
  } catch {}

  console.log(`\n==> Building ${c.name} ...`);
  await run(
    'wasm-pack',
    ['build', '--target', 'bundler', '--out-dir', 'pkg'],
    dir
  );
}

console.log('\n✓ All kernels built.');
