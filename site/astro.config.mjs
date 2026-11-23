import { defineConfig } from "astro/config";

// [ADR 0019]: GitHub Pages 저장소 하위 경로(/baeum-maru)로 정적 빌드한다.
export default defineConfig({
  site: "https://homeria.github.io",
  base: "/baeum-maru",
  output: "static",
  trailingSlash: "always",
});
