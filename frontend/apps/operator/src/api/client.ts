import createClient from 'openapi-fetch'
import type { paths } from './schema'

// 경로 키에 이미 /api/v1 접두사가 들어있어 baseUrl은 비운다.
// 세션 쿠키는 same-origin(개발은 vite 프록시)으로 자동 전송된다.
export const api = createClient<paths>({ credentials: 'include' })
