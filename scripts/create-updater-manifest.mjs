import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";

const repo = "little1d/any2bibtex";
const releaseDir = path.resolve("release-files");
const changelogPath = path.resolve("CHANGELOG.md");
const version = process.env.RELEASE_VERSION?.replace(/^v/, "");

if (!version) {
  throw new Error("RELEASE_VERSION is required, for example RELEASE_VERSION=0.0.5");
}

async function listFiles(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(dir, entry.name);
      return entry.isDirectory() ? listFiles(fullPath) : [fullPath];
    }),
  );
  return files.flat();
}

function findArtifact(files, matcher) {
  const match = files.find((file) => matcher(path.basename(file)));
  if (!match) {
    throw new Error("Missing updater artifact for matcher");
  }
  return match;
}

async function readSignature(artifactPath) {
  return (await readFile(`${artifactPath}.sig`, "utf8")).trim();
}

function releaseUrl(filePath) {
  const filename = encodeURIComponent(path.basename(filePath));
  return `https://github.com/${repo}/releases/latest/download/${filename}`;
}

async function readLatestNotes() {
  const changelog = await readFile(changelogPath, "utf8");
  const match = changelog.match(/## \[[^\]]+\][\s\S]*?(?=\n## \[|$)/);
  return match ? match[0].trim() : `any2bibtex v${version}`;
}

const files = await listFiles(releaseDir);

const macArtifact = findArtifact(files, (name) => name.endsWith(".app.tar.gz"));
const windowsArtifact = findArtifact(files, (name) => name.endsWith(".exe"));
const linuxArtifact = findArtifact(files, (name) => name.endsWith(".AppImage"));

const manifest = {
  version,
  notes: await readLatestNotes(),
  pub_date: new Date().toISOString(),
  platforms: {
    "darwin-aarch64": {
      signature: await readSignature(macArtifact),
      url: releaseUrl(macArtifact),
    },
    "windows-x86_64": {
      signature: await readSignature(windowsArtifact),
      url: releaseUrl(windowsArtifact),
    },
    "linux-x86_64": {
      signature: await readSignature(linuxArtifact),
      url: releaseUrl(linuxArtifact),
    },
  },
};

await writeFile(
  path.join(releaseDir, "latest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
);
