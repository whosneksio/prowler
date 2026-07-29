#!/usr/bin/env bun
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { execSync } from "node:child_process";

const root = import.meta.dir;
const arg = process.argv[2];
const doTag = process.argv.includes("--tag");

if (!arg || arg.startsWith("-")) {
  console.error("Usage: bun run bump.ts <major|minor|patch|X.Y.Z> [--tag]");
  console.error("  --tag   also commit the bump and create a git tag (vX.Y.Z)");
  process.exit(1);
}

const pkgPath = join(root, "package.json");
const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
const current: string = pkg.version;

const m = current.match(/^(\d+)\.(\d+)\.(\d+)$/);
if (!m) {
  console.error(`Cannot parse current version "${current}" in package.json`);
  process.exit(1);
}
const [maj, min, pat] = [Number(m[1]), Number(m[2]), Number(m[3])];

let next: string;
if (arg === "major") next = `${maj + 1}.0.0`;
else if (arg === "minor") next = `${maj}.${min + 1}.0`;
else if (arg === "patch") next = `${maj}.${min}.${pat + 1}`;
else if (/^\d+\.\d+\.\d+$/.test(arg)) next = arg;
else {
  console.error(`Invalid argument "${arg}". Use major | minor | patch | X.Y.Z`);
  process.exit(1);
}

if (next === current) {
  console.error(`Version is already ${next} - nothing to do.`);
  process.exit(1);
}

console.log(`Bumping ${current} → ${next}`);

function writeJson(path: string, obj: unknown) {
  writeFileSync(path, JSON.stringify(obj, null, 2) + "\n");
}

pkg.version = next;
writeJson(pkgPath, pkg);

const tauriPath = join(root, "src-tauri", "tauri.conf.json");
const tauri = JSON.parse(readFileSync(tauriPath, "utf8"));
tauri.version = next;
writeJson(tauriPath, tauri);

const prowlerVersion = /(name = "prowler"\r?\nversion = )"[^"]*"/;
for (const rel of ["src-tauri/Cargo.toml", "src-tauri/Cargo.lock"]) {
  const p = join(root, rel);
  if (!existsSync(p)) continue;
  const text = readFileSync(p, "utf8");
  if (!prowlerVersion.test(text)) {
    console.warn(`  ! could not find prowler version in ${rel} - skipped`);
    continue;
  }
  writeFileSync(p, text.replace(prowlerVersion, `$1"${next}"`));
}

console.log(
  "Updated: package.json, src-tauri/tauri.conf.json, Cargo.toml, Cargo.lock",
);

if (doTag) {
  execSync("git add -A", { stdio: "inherit" });
  execSync(`git commit -m "Release v${next}"`, { stdio: "inherit" });
  execSync(`git tag v${next}`, { stdio: "inherit" });
  console.log(`\nCommitted and tagged v${next}.`);
  console.log("Push (triggers the release build):");
  console.log("  git push origin main --tags");
} else {
  console.log("\nNext:");
  console.log(`  git add -A && git commit -m "Release v${next}"`);
  console.log(`  git tag v${next}`);
  console.log("  git push origin main --tags");
}
