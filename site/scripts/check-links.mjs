// 빌드 결과물(dist/)의 내부 링크와 이미지 경로가 실제로 존재하는지 검사한다.
// 외부 링크(http/https)는 네트워크 흔들림으로 거짓 실패가 나기 쉬워 검사하지 않는다 (ADR 0019).

import fs from "node:fs";
import path from "node:path";

const DIST_DIR = path.resolve(import.meta.dirname, "..", "dist");
const BASE_PATH = "/baeum-maru";

const ATTR_PATTERN = /\b(?:href|src)=["']([^"']+)["']/g;
const SKIP_PREFIXES = [
  "http://",
  "https://",
  "mailto:",
  "tel:",
  "data:",
  "javascript:",
  "#",
];

function collectHtmlFiles(dir) {
  const result = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      result.push(...collectHtmlFiles(fullPath));
    } else if (entry.isFile() && entry.name.endsWith(".html")) {
      result.push(fullPath);
    }
  }
  return result;
}

function targetExists(distRelativePath) {
  const clean = distRelativePath.replace(/^\/+/, "");
  const exact = path.join(DIST_DIR, clean);
  if (fs.existsSync(exact) && fs.statSync(exact).isFile()) {
    return true;
  }
  const indexHtml = path.join(DIST_DIR, clean, "index.html");
  if (fs.existsSync(indexHtml) && fs.statSync(indexHtml).isFile()) {
    return true;
  }
  return false;
}

function checkTarget(htmlFile, rawRef) {
  if (SKIP_PREFIXES.some((prefix) => rawRef.startsWith(prefix))) {
    return null;
  }

  const withoutHash = rawRef.split("#")[0];
  const withoutQuery = withoutHash.split("?")[0];
  if (!withoutQuery) {
    return null;
  }

  let decoded;
  try {
    decoded = decodeURIComponent(withoutQuery);
  } catch {
    return `잘못된 경로 인코딩: ${rawRef}`;
  }

  if (decoded.startsWith("/")) {
    // GitHub Pages 하위 경로 배포에서 base가 빠진 루트 절대 경로는 깨진다.
    if (decoded !== BASE_PATH && !decoded.startsWith(`${BASE_PATH}/`)) {
      return `하위 경로(${BASE_PATH})가 빠진 절대 경로: ${rawRef}`;
    }
    const stripped = decoded.slice(BASE_PATH.length) || "/";
    if (!targetExists(stripped)) {
      return `대상을 찾을 수 없음: ${rawRef}`;
    }
    return null;
  }

  const currentDir = path.dirname(htmlFile);
  const resolved = path.resolve(currentDir, decoded);
  if (!resolved.startsWith(DIST_DIR)) {
    return `dist 바깥을 가리키는 상대 경로: ${rawRef}`;
  }
  const relativeToDist = path.relative(DIST_DIR, resolved);
  if (!targetExists(relativeToDist)) {
    return `대상을 찾을 수 없음: ${rawRef}`;
  }
  return null;
}

if (!fs.existsSync(DIST_DIR)) {
  console.error(`빌드 결과물 폴더가 없습니다: ${DIST_DIR}`);
  process.exit(1);
}

const htmlFiles = collectHtmlFiles(DIST_DIR);
const failures = [];
let checkedCount = 0;

for (const file of htmlFiles) {
  const html = fs.readFileSync(file, "utf8");
  const relFile = path.relative(DIST_DIR, file);
  for (const match of html.matchAll(ATTR_PATTERN)) {
    const rawRef = match[1];
    if (SKIP_PREFIXES.some((prefix) => rawRef.startsWith(prefix))) {
      continue;
    }
    checkedCount += 1;
    const error = checkTarget(file, rawRef);
    if (error) {
      failures.push(`${relFile}: ${error}`);
    }
  }
}

if (failures.length > 0) {
  console.error("깨진 내부 링크 또는 이미지 경로가 있습니다:");
  for (const item of failures) {
    console.error(`  - ${item}`);
  }
  process.exit(1);
}

console.log(
  `내부 링크·이미지 검사 통과 (HTML ${htmlFiles.length}개, 경로 ${checkedCount}개 확인)`,
);
