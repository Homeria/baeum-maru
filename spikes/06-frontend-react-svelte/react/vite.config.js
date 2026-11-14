// 버린 코드: 빌드 결과를 HTML 파일 하나로 만든다 (Windows에서 더블클릭으로 열기 위해).
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { viteSingleFile } from 'vite-plugin-singlefile';

export default defineConfig({
  plugins: [react(), viteSingleFile()],
  server: { fs: { allow: ['..'] } },
});
