#!/usr/bin/env node

const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..', '..', '..');
const packageRoot = path.resolve(__dirname, '..');
const isWindows = process.platform === 'win32';
const prebuildDir = path.join(packageRoot, 'prebuilds');

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: 'inherit',
    ...options,
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function ensureStaticLibrary() {
  const cargoArgs = ['build', '--release', '-p', 'zkprov-ffi-c'];
  run('cargo', cargoArgs, { cwd: repoRoot });

  const staticName = isWindows ? 'zkprov.lib' : 'libzkprov.a';
  const staticPath = path.join(repoRoot, 'target', 'release', staticName);

  if (!fs.existsSync(staticPath)) {
    console.error(`Expected static library at ${staticPath}, but it was not found.`);
    process.exit(1);
  }

  return staticPath;
}

function runPrebuild(staticPath) {
  const binName = isWindows ? 'prebuildify.cmd' : 'prebuildify';
  const binPath = path.join(packageRoot, 'node_modules', '.bin', binName);

  if (!fs.existsSync(binPath)) {
    console.error('Could not find the prebuildify binary. Please run "npm install" first.');
    process.exit(1);
  }

  fs.rmSync(prebuildDir, { recursive: true, force: true });

  run(
    binPath,
    ['-t', '18.0.0', '-t', '20.0.0', '-t', '22.0.0'],
    {
      cwd: packageRoot,
      env: {
        ...process.env,
        ZKPROV_STATIC: staticPath,
      },
    },
  );
}

function normalizePrebuildNames() {
  if (!fs.existsSync(prebuildDir)) {
    console.warn('prebuildify did not produce a prebuilds directory.');
    return;
  }

  for (const entry of fs.readdirSync(prebuildDir)) {
    const targetDir = path.join(prebuildDir, entry);
    if (!fs.statSync(targetDir).isDirectory()) {
      continue;
    }

    const desiredPath = path.join(targetDir, 'zkprov.napi.node');
    if (fs.existsSync(desiredPath)) {
      continue;
    }

    const artifacts = fs.readdirSync(targetDir).filter((name) => name.endsWith('.node'));
    if (artifacts.length === 0) {
      console.warn(`No .node artifact found in ${targetDir}.`);
      continue;
    }

    if (artifacts.length > 1) {
      throw new Error(
        `Expected a single .node artifact in ${targetDir}, but found: ${artifacts.join(', ')}.`,
      );
    }

    const currentPath = path.join(targetDir, artifacts[0]);
    fs.renameSync(currentPath, desiredPath);
  }
}

function main() {
  const staticPath = ensureStaticLibrary();
  console.log(`Using static library at ${staticPath}`);
  runPrebuild(staticPath);
  normalizePrebuildNames();
}

main();
