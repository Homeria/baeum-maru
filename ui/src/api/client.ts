/** 이 앱의 버전. 요청마다 호스트에 알린다. */
export const APP_VERSION = __APP_VERSION__;

const VERSION_HEADER = "x-baeum-version";

/** 호스트가 실패 상태 코드로 응답했다. */
export class HostError extends Error {
  readonly status: number;

  constructor(status: number) {
    super(`호스트 응답 실패: ${status}`);
    this.status = status;
  }
}

/**
 * 호스트에 GET 요청을 보내고 JSON을 받는다.
 *
 * ADR 0012: 모든 요청에 앱 버전을 싣는다.
 */
export async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(path, {
    headers: { [VERSION_HEADER]: APP_VERSION },
  });
  if (!response.ok) {
    throw new HostError(response.status);
  }
  return (await response.json()) as T;
}
