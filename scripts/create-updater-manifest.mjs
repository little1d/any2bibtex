import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";

const repository = process.env.GITHUB_REPOSITORY || "little1d/any2bibtex";
const releaseDir = path.resolve(process.env.RELEASE_DIR || "release-files");
const changelogPath = path.resolve(process.env.CHANGELOG_PATH || "CHANGELOG.md");
const version = process.env.RELEASE_VERSION?.replace(/^v/, "");

if (!version) {
  throw new Error("RELEASE_VERSION is required, for example RELEASE_VERSION=0.0.7");
}

const platformArtifacts = [
  {
    key: "darwin-aarch64",
    label: "macOS Apple Silicon",
    matches: (name) => name.endsWith("_aarch64.app.tar.gz"),
  },
  {
    key: "darwin-x86_64",
    label: "macOS Intel",
    matches: (name) => name.endsWith("_x64.app.tar.gz"),
  },
  {
    key: "windows-x86_64",
    label: "Windows x64",
    matches: (name) => name.endsWith("_x64-setup.exe"),
  },
  {
    key: "linux-x86_64",
    label: "Linux x64",
    matches: (name) => name.endsWith("_amd64.AppImage"),
  },
  {
    key: "linux-aarch64",
    label: "Linux ARM64",
    matches: (name) => name.endsWith("_arm64.AppImage"),
  },
];

async function listFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nestedFiles = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(directory, entry.name);
      return entry.isDirectory() ? listFiles(fullPath) : [fullPath];
    }),
  );
  return nestedFiles.flat();
}

function findArtifact(files, platform) {
  const matches = files.filter((file) => platform.matches(path.basename(file)));
  if (matches.length !== 1) {
    const available = files
      .map((file) => path.relative(releaseDir, file))
      .sort()
      .join("\n");
    throw new Error(
      `Expected one updater artifact for ${platform.label}, found ${matches.length}.\n` +
        `Available files:\n${available}`,
    );
  }
  return matches[0];
}

async function readSignature(artifactPath) {
  return (await readFile(`${artifactPath}.sig`, "utf8")).trim();
}

function releaseUrl(filePath) {
  const filename = encodeURIComponent(path.basename(filePath));
  return `https://github.com/${repository}/releases/latest/download/${filename}`;
}

async function readReleaseNotes() {
  const changelog = await readFile(changelogPath, "utf8");
  const escapedVersion = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = changelog.match(
    new RegExp(`## \\[${escapedVersion}\\][\\s\\S]*?(?=\\n## \\[|$)`),
  );
  return match ? match[0].trim() : `any2bibtex v${version}`;
}

const files = await listFiles(releaseDir);
const platforms = {};

for (const platform of platformArtifacts) {
  const artifact = findArtifact(files, platform);
  platforms[platform.key] = {
    signature: await readSignature(artifact),
    url: releaseUrl(artifact),
  };
}

const manifest = {
  version,
  notes: await readReleaseNotes(),
  pub_date: new Date().toISOString(),
  platforms,
};

await writeFile(path.join(releaseDir, "release-notes.md"), `${manifest.notes}\n`);
await writeFile(
  path.join(releaseDir, "latest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
);
