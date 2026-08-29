import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);
const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const scriptPath = path.join(projectRoot, "scripts", "create-updater-manifest.mjs");

test("creates a manifest for every supported updater target", async () => {
  const fixtureRoot = await mkdtemp(path.join(os.tmpdir(), "any2bibtex-manifest-"));
  const releaseDir = path.join(fixtureRoot, "release-files");
  const changelogPath = path.join(fixtureRoot, "CHANGELOG.md");

  try {
    await mkdir(releaseDir);
    const artifacts = [
      "any2bibtex_1.2.3_aarch64.app.tar.gz",
      "any2bibtex_1.2.3_x64.app.tar.gz",
      "any2bibtex_1.2.3_x64-setup.exe",
      "any2bibtex_1.2.3_amd64.AppImage",
      "any2bibtex_1.2.3_arm64.AppImage",
    ];

    await Promise.all(
      artifacts.flatMap((name) => [
        writeFile(path.join(releaseDir, name), "artifact"),
        writeFile(path.join(releaseDir, `${name}.sig`), `signature-${name}`),
      ]),
    );
    await writeFile(
      changelogPath,
      "# Changelog\n\n## [1.2.3] - 2026-08-29\n\n- Release notes\n\n## [1.2.2]\n",
    );

    await execFileAsync(process.execPath, [scriptPath], {
      cwd: projectRoot,
      env: {
        ...process.env,
        CHANGELOG_PATH: changelogPath,
        GITHUB_REPOSITORY: "example/any2bibtex",
        RELEASE_DIR: releaseDir,
        RELEASE_VERSION: "1.2.3",
      },
    });

    const manifest = JSON.parse(await readFile(path.join(releaseDir, "latest.json"), "utf8"));
    assert.equal(manifest.version, "1.2.3");
    assert.deepEqual(Object.keys(manifest.platforms).sort(), [
      "darwin-aarch64",
      "darwin-x86_64",
      "linux-aarch64",
      "linux-x86_64",
      "windows-x86_64",
    ]);
    assert.match(
      manifest.platforms["darwin-x86_64"].url,
      /any2bibtex_1.2.3_x64.app.tar.gz$/,
    );
    assert.match(manifest.notes, /Release notes/);
  } finally {
    await rm(fixtureRoot, { force: true, recursive: true });
  }
});
