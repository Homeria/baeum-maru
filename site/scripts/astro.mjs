// Windows·Linux·macOS 모두에서 Astro 텔레메트리를 끈 채 실행한다.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const astroPkgDir = path.resolve(
  import.meta.dirname,
  "..",
  "node_modules",
  "astro",
);
const pkg = JSON.parse(
  fs.readFileSync(path.join(astroPkgDir, "package.json"), "utf8"),
);
const binRel = typeof pkg.bin === "string" ? pkg.bin : pkg.bin.astro;
const astroBin = path.resolve(astroPkgDir, binRel);

const result = spawnSync(
  process.execPath,
  [astroBin, ...process.argv.slice(2)],
  {
    stdio: "inherit",
    env: { ...process.env, ASTRO_TELEMETRY_DISABLED: "1" },
  },
);

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 1);
