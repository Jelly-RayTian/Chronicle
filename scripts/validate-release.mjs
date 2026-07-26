import { readFileSync, existsSync } from 'node:fs';
import process from 'node:process';

const readJson = (path) => JSON.parse(readFileSync(path, 'utf8'));
const packageVersion = readJson('package.json').version;
const lockVersion = readJson('package-lock.json').version;
const tauriVersion = readJson('src-tauri/tauri.conf.json').version;
const cargoToml = readFileSync('src-tauri/Cargo.toml', 'utf8');
const cargoLock = readFileSync('src-tauri/Cargo.lock', 'utf8');
const cargoVersion = cargoToml.match(/^\[package\][\s\S]*?^version = "([^"]+)"/m)?.[1];
const cargoLockVersion = cargoLock.match(
  /^\[\[package\]\]\r?\nname = "chronicle"\r?\nversion = "([^"]+)"/m,
)?.[1];

const versions = {
  'package.json': packageVersion,
  'package-lock.json': lockVersion,
  'src-tauri/Cargo.toml': cargoVersion,
  'src-tauri/Cargo.lock': cargoLockVersion,
  'src-tauri/tauri.conf.json': tauriVersion,
};

const uniqueVersions = new Set(Object.values(versions));
if (uniqueVersions.size !== 1 || uniqueVersions.has(undefined)) {
  throw new Error(`Release versions are not aligned: ${JSON.stringify(versions)}`);
}

if (!/^\d+\.\d+\.\d+$/.test(packageVersion)) {
  throw new Error(`Release version must be stable semver, received ${packageVersion}`);
}

const tagArgumentIndex = process.argv.indexOf('--tag');
const tag =
  tagArgumentIndex >= 0 ? process.argv[tagArgumentIndex + 1] : process.env.GITHUB_REF_NAME;
if (tag && tag !== `v${packageVersion}`) {
  throw new Error(`Tag ${tag} does not match manifest version v${packageVersion}`);
}

const releaseNotes = `docs/release-notes-v${packageVersion}.md`;
if (!existsSync(releaseNotes)) {
  throw new Error(`Missing ${releaseNotes}`);
}

process.stdout.write(
  `Release metadata is aligned at v${packageVersion}${tag ? ` for tag ${tag}` : ''}.\n`,
);
