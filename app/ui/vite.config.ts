import { readFileSync } from "node:fs";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import babel from "@rolldown/plugin-babel";
import { defineConfig } from "vite";

// [ADR 0012]: 버전 번호는 workspace Cargo.toml 한 곳에서 읽는다
function workspaceVersion(): string {
  const cargoToml = readFileSync(
    new URL("../Cargo.toml", import.meta.url),
    "utf8",
  );
  const version = /\[workspace\.package\][^[]*?\nversion\s*=\s*"([^"]+)"/.exec(
    cargoToml,
  )?.[1];
  if (!version) {
    throw new Error(
      "Cargo.toml [workspace.package]에서 version을 찾지 못했습니다",
    );
  }
  return version;
}

export default defineConfig({
  plugins: [react(), babel({ presets: [reactCompilerPreset()] })],
  define: {
    __APP_VERSION__: JSON.stringify(workspaceVersion()),
  },
  server: {
    // 개발 중에는 이 PC에서 띄운 서버로 보냄
    proxy: {
      "/api": "http://127.0.0.1:47800",
    },
  },
});
