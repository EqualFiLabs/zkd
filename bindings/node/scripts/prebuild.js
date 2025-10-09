#!/usr/bin/env node
// Cross-platform prebuild with aggressive logging for CI.

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const fsp = fs.promises;

const packageRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(__dirname, '..', '..', '..');
const targetDir = path.join(repoRoot, 'target', 'release');

// Where the Rust static lib should be (MSVC .lib on Windows; .a on *nix)
const libName = process.platform === 'win32' ? 'zkprov.lib' : 'libzkprov.a';
const libPath = path.join(targetDir, libName);

// Node addon build output location (node-gyp default)
const nodeBuildDir = path.join(packageRoot, 'build', 'Release');
const candidateNames = [
  'zkprov.node',
  'binding.node',
];
const prebuildRoot = path.join(packageRoot, 'prebuilds');
const prebuildName = 'zkprov.napi.node';

function logDir(title, dir) {
  try {
    const list = fs.readdirSync(dir);
    console.log(`\n== ${title}: ${dir}`);
    list.forEach((f) => console.log(' -', f));
  } catch (e) {
    console.log(`\n== ${title}: ${dir} (unreadable: ${e.message})`);
  }
}

function fileExists(p) {
  try {
    return fs.statSync(p).isFile();
  } catch (err) {
    if (err.code !== 'ENOENT') {
      console.warn(`Error checking file ${p}:`, err);
    }
    return false;
  }
}

function findBuiltNode() {
  for (const name of candidateNames) {
    const p = path.join(nodeBuildDir, name);
    if (fileExists(p)) return p;
  }
  return null;
}

async function run(cmd, args, opts = {}) {
  console.log(`\n$ ${cmd} ${args.join(' ')}`);
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, {
      stdio: 'inherit',
      shell: process.platform === 'win32',
      ...opts,
    });
    child.on('exit', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${cmd} exited with code ${code}`));
    });
    child.on('error', reject);
  });
}

(async () => {
  try {
    // 1) Build Rust static lib
    await run('cargo', ['build', '-p', 'zkprov-ffi-c', '--release'], { cwd: repoRoot });

    if (!fileExists(libPath)) {
      logDir('target/release contents', targetDir);
      throw new Error(`Static library not found at ${libPath}`);
    }
    console.log(`Using static library at ${libPath}`);

    // 2) Build Node addon (ensure MSVC detection on Windows)
    const env = { ...process.env };
    const normalizedLibPath = process.platform === 'win32' ? libPath.replace(/\\/g, '/') : libPath;
    if (process.platform === 'win32') {
      // Help node-gyp find VS2022 on GitHub runners
      env.GYP_MSVS_VERSION = env.GYP_MSVS_VERSION || '2022';
      // Ensure Release config
      env.CONFIGURATION = env.CONFIGURATION || 'Release';
    }
    // Let binding.gyp pick up the static lib path if needed
    env.ZKPROV_STATIC_LIB = env.ZKPROV_STATIC_LIB || normalizedLibPath;
    env.ZKPROV_STATIC = env.ZKPROV_STATIC || normalizedLibPath;

    // Ensure deps installed
    await run('npm', ['ci'], { cwd: packageRoot, env });

    // Run node-gyp using the locally installed binary to avoid npm pre/post script recursion
    const nodeGypBin = path.join(
      packageRoot,
      'node_modules',
      '.bin',
      process.platform === 'win32' ? 'node-gyp.cmd' : 'node-gyp',
    );
    if (!fs.existsSync(nodeGypBin)) {
      throw new Error(`node-gyp binary not found at ${nodeGypBin}`);
    }
    await run(nodeGypBin, ['rebuild'], { cwd: packageRoot, env });

    // 3) Verify .node artifact and copy/pack prebuilds if applicable
    const built = findBuiltNode();
    if (!built) {
      logDir('Node build dir', nodeBuildDir);
      logDir('bindings/node dir', packageRoot);
      throw new Error('Failed to locate built .node file in build/Release');
    }
    console.log(`Built addon: ${built}`);

    // 4) Copy the built addon into the prebuild layout expected by npm
    const platform = process.platform;
    const arch = process.arch;
    const destDir = path.join(prebuildRoot, `${platform}-${arch}`);
    const destPath = path.join(destDir, prebuildName);

    console.log(`Preparing prebuild directory at ${destDir}`);
    await fsp.rm(destDir, { recursive: true, force: true }).catch((err) => {
      console.warn(`Failed to remove existing prebuild dir ${destDir}:`, err.message);
    });
    await fsp.mkdir(destDir, { recursive: true });
    await fsp.copyFile(built, destPath);
    console.log(`Copied prebuild artifact to ${destPath}`);

    console.log('\nPrebuild completed successfully.');
  } catch (err) {
    console.error('\nFATAL prebuild error:', err && err.stack ? err.stack : err);
    // Extra diagnostics
    logDir('Repo root', repoRoot);
    logDir('Target release', targetDir);
    logDir('Node build/Release', nodeBuildDir);
    process.exit(1);
  }
})();
