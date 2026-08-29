import { readFile } from "node:fs/promises";

const expectedVersion = (process.argv[2] || process.env.RELEASE_TAG || "").replace(/^v/, "");
const packageJson = JSON.parse(await readFile("package.json", "utf8"));
const tauriConfig = JSON.parse(await readFile("src-tauri/tauri.conf.json", "utf8"));
const cargoToml = await readFile("src-tauri/Cargo.toml", "utf8");
const cargoVersion = cargoToml.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m)?.[1];

const versions = {
  "package.json": packageJson.version,
  "src-tauri/tauri.conf.json": tauriConfig.version,
  "src-tauri/Cargo.toml": cargoVersion,
};

const uniqueVersions = new Set(Object.values(versions));
if (uniqueVersions.size !== 1 || uniqueVersions.has(undefined)) {
  throw new Error(
    `Version mismatch:\n${Object.entries(versions)
      .map(([file, version]) => `- ${file}: ${version || "missing"}`)
      .join("\n")}`,
  );
}

const [projectVersion] = uniqueVersions;
if (expectedVersion && projectVersion !== expectedVersion) {
  throw new Error(
    `Release tag v${expectedVersion} does not match project version ${projectVersion}.`,
  );
}

console.log(`Version ${projectVersion} is consistent across package, Tauri, and Cargo.`);
