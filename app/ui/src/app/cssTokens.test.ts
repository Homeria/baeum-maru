// @vitest-environment node
/// <reference types="node" />
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const SRC_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);

function collectCssFiles(dir: string): string[] {
  const results: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      results.push(...collectCssFiles(full));
    } else if (entry.name.endsWith(".css")) {
      results.push(full);
    }
  }
  return results.sort();
}

test("모든 CSS 파일의 var(--...) 참조는 정의된 디자인 토큰만 쓴다", () => {
  const files = collectCssFiles(SRC_ROOT);
  expect(files.length).toBeGreaterThan(0);

  const defined = new Set<string>();
  for (const file of files) {
    const text = readFileSync(file, "utf8");
    for (const match of text.matchAll(/(--[a-z0-9-]+)\s*:/g)) {
      defined.add(match[1]);
    }
  }

  const undefinedRefs: string[] = [];
  for (const file of files) {
    const relative = path.relative(SRC_ROOT, file);
    const lines = readFileSync(file, "utf8").split("\n");
    lines.forEach((line, index) => {
      for (const match of line.matchAll(/var\((--[a-z0-9-]+)/g)) {
        if (!defined.has(match[1])) {
          undefinedRefs.push(`${relative}:${index + 1} ${match[1]}`);
        }
      }
    });
  }

  expect(undefinedRefs).toEqual([]);
});
