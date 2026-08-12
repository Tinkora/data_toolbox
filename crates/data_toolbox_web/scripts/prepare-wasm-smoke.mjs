import { cp, lstat, mkdtemp, readdir, realpath, rename, rm } from "node:fs/promises";
import { basename, dirname, join, relative, resolve, sep } from "node:path";

const destination = resolve("web/pkg");
const artifact = process.env.WASM_SMOKE_PACKAGE;
const requiredFiles = ["package.json", "data_toolbox_web.js", "data_toolbox_web_bg.wasm"];

async function metadata(path) {
  try { return await lstat(path); }
  catch (error) { if (error.code === "ENOENT") return null; throw error; }
}

async function validateTree(root, current = root) {
  for (const entry of await readdir(current)) {
    const path = join(current, entry);
    const entryMetadata = await lstat(path);
    if (entryMetadata.isSymbolicLink()) throw new Error("WASM package contains a symbolic link: " + relative(root, path));
    if (entryMetadata.isDirectory()) await validateTree(root, path);
    else if (!entryMetadata.isFile()) throw new Error("WASM package contains a special file: " + relative(root, path));
  }
}

async function validatePackage(root) {
  const rootMetadata = await metadata(root);
  if (!rootMetadata?.isDirectory() || rootMetadata.isSymbolicLink()) throw new Error("WASM package must be a real directory");
  await validateTree(root);
  for (const requiredFile of requiredFiles) {
    const requiredMetadata = await metadata(join(root, requiredFile));
    if (!requiredMetadata?.isFile() || requiredMetadata.isSymbolicLink()) throw new Error("WASM package is missing " + requiredFile);
  }
}

function containsPath(parent, child) {
  const path = relative(parent, child);
  return path === "" || (path !== ".." && !path.startsWith(".." + sep));
}

async function installPackage(source) {
  await validatePackage(source);
  const targetMetadata = await metadata(destination);
  if (targetMetadata && (!targetMetadata.isDirectory() || targetMetadata.isSymbolicLink())) throw new Error("web/pkg must be a real directory");
  const targetParent = await realpath(dirname(destination));
  const canonicalDestination = join(targetParent, basename(destination));
  const canonicalSource = await realpath(source);
  if (containsPath(canonicalSource, canonicalDestination) || containsPath(canonicalDestination, canonicalSource)) {
    throw new Error("WASM source and destination must not overlap");
  }
  const staging = await mkdtemp(join(targetParent, ".pkg.staging-"));
  const previous = staging + "-previous";
  try {
    for (const entry of await readdir(canonicalSource)) {
      await cp(join(canonicalSource, entry), join(staging, entry), { recursive: true, force: false, errorOnExist: true });
    }
    await validatePackage(staging);
    if (targetMetadata) await rename(destination, previous);
    await rename(staging, destination);
    if (targetMetadata) await rm(previous, { recursive: true });
  } finally {
    await rm(staging, { recursive: true, force: true });
    await rm(previous, { recursive: true, force: true });
  }
}

if (artifact) await installPackage(artifact);
else await validatePackage(destination);
