import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    // 개발 중 /api 요청을 백엔드로 프록시 → 세션 쿠키 same-origin 유지
    proxy: {
      '/api': 'http://127.0.0.1:18080',
    },
  },
})
