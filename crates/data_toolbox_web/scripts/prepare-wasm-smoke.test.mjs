import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  readlink,
  rm,
  symlink,
  writeFile
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const script = fileURLToPath(new URL("./prepare-wasm-smoke.mjs", import.meta.url));

async function createPackage(directory, marker, extraFiles = {}) {
  await mkdir(directory, { recursive: true });
  const files = {
    "package.json": JSON.stringify({ name: `fixture-${marker}` }),
    "data_toolbox_web.js": `export const marker = "${marker}";\n`,
    "data_toolbox_web_bg.wasm": marker,
    ...extraFiles
  };

  for (const [path, contents] of Object.entries(files)) {
    const target = join(directory, path);
    await mkdir(dirname(target), { recursive: true });
    await writeFile(target, contents);
  }
}

async function snapshot(directory) {
  const entries = [];

  async function visit(current) {
    for (const entry of await readdir(current, { withFileTypes: true })) {
      const path = join(current, entry.name);
      const name = relative(directory, path);
      if (entry.isDirectory()) {
        entries.push(["directory", name]);
        await visit(path);
      } else if (entry.isFile()) {
        entries.push(["file", name, (await readFile(path)).toString("base64")]);
      } else if (entry.isSymbolicLink()) {
        entries.push(["symlink", name, await readlink(path)]);
      } else {
        entries.push(["special", name]);
      }
    }
  }

  await visit(directory);
  return entries.sort((left, right) => left[1].localeCompare(right[1]));
}

async function assertNoTemporaryPackages(root) {
  const web = join(root, "web");
  const entries = await readdir(web);
  assert.deepEqual(
    entries.filter((entry) => entry.startsWith(".pkg.staging-")),
    []
  );
}

function runPrepare(cwd, artifact) {
  const env = { ...process.env };
  if (artifact) env.WASM_SMOKE_PACKAGE = artifact;
  else delete env.WASM_SMOKE_PACKAGE;

  return spawnSync(process.execPath, [script], { cwd, env, encoding: "utf8" });
}

async function fixture(t) {
  const root = await mkdtemp(join(tmpdir(), "data-toolbox-wasm-smoke-"));
  await mkdir(join(root, "web"));
  t.after(() => rm(root, { recursive: true, force: true }));
  return root;
}

test("rejects a missing artifact without changing the current package", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  await createPackage(destination, "old");
  const before = await snapshot(destination);

  const result = runPrepare(root, join(root, "missing-artifact"));

  assert.notEqual(result.status, 0, result.stdout + result.stderr);
  assert.deepEqual(await snapshot(destination), before);
  await assertNoTemporaryPackages(root);
});

test("rejects an artifact missing a required file", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  const artifact = join(root, "artifact");
  await createPackage(destination, "old");
  await createPackage(artifact, "new");
  await rm(join(artifact, "data_toolbox_web_bg.wasm"));
  const before = await snapshot(destination);

  const result = runPrepare(root, artifact);

  assert.notEqual(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stderr, /missing data_toolbox_web_bg\.wasm/);
  assert.deepEqual(await snapshot(destination), before);
  await assertNoTemporaryPackages(root);
});

test("rejects an artifact containing a symbolic link", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  const artifact = join(root, "artifact");
  await createPackage(destination, "old");
  await createPackage(artifact, "new");
  await symlink("data_toolbox_web.js", join(artifact, "linked.js"));
  const before = await snapshot(destination);

  const result = runPrepare(root, artifact);

  assert.notEqual(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stderr, /contains a symbolic link/);
  assert.deepEqual(await snapshot(destination), before);
  await assertNoTemporaryPackages(root);
});

test("rejects an artifact directory reached through a symbolic link", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  const artifact = join(root, "artifact");
  const linkedArtifact = join(root, "linked-artifact");
  await createPackage(destination, "old");
  await createPackage(artifact, "new");
  await symlink(artifact, linkedArtifact);
  const before = await snapshot(destination);

  const result = runPrepare(root, linkedArtifact);

  assert.notEqual(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stderr, /must be a real directory/);
  assert.deepEqual(await snapshot(destination), before);
  await assertNoTemporaryPackages(root);
});

test("rejects an artifact containing a special file", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  const artifact = join(root, "artifact");
  await createPackage(destination, "old");
  await createPackage(artifact, "new");
  execFileSync("/usr/bin/mkfifo", [join(artifact, "named-pipe")]);
  const before = await snapshot(destination);

  const result = runPrepare(root, artifact);

  assert.notEqual(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stderr, /contains a special file/);
  assert.deepEqual(await snapshot(destination), before);
  await assertNoTemporaryPackages(root);
});

test("installs a valid artifact when no package exists", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  const artifact = join(root, "artifact");
  await createPackage(artifact, "new", { "nested/metadata.txt": "copied" });
  const expected = await snapshot(artifact);

  const result = runPrepare(root, artifact);

  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.deepEqual(await snapshot(destination), expected);
  await assertNoTemporaryPackages(root);
});

test("replaces the previous package completely after validation", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  const artifact = join(root, "artifact");
  await createPackage(destination, "old", { "obsolete.txt": "remove" });
  await createPackage(artifact, "new", { "nested/metadata.txt": "copied" });
  const expected = await snapshot(artifact);

  const result = runPrepare(root, artifact);

  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.deepEqual(await snapshot(destination), expected);
  await assertNoTemporaryPackages(root);
});

test("rejects an artifact path that is the destination", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  await createPackage(destination, "local");
  const before = await snapshot(destination);

  const result = runPrepare(root, destination);

  assert.notEqual(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stderr, /must not overlap/);
  assert.deepEqual(await snapshot(destination), before);
  await assertNoTemporaryPackages(root);
});

test("validates the local package without replacing it when no artifact is set", async (t) => {
  const root = await fixture(t);
  const destination = join(root, "web/pkg");
  await createPackage(destination, "local", { "local-only.txt": "keep" });
  const before = await snapshot(destination);

  const result = runPrepare(root);

  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.deepEqual(await snapshot(destination), before);
  await assertNoTemporaryPackages(root);
});
