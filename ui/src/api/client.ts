import type { ErrorCode } from "./generated/ErrorCode";
import type { FieldError } from "./generated/FieldError";
import type { RuleViolationResponse } from "./generated/RuleViolationResponse";
import {
  clearSession,
  getSession,
  isStaffActive,
  lockSession,
  noteRequest,
} from "./session";

/** 이 앱의 버전. 요청마다 호스트에 알린다. */
export const APP_VERSION = __APP_VERSION__;

const VERSION_HEADER = "x-baeum-version";

/** 기기 증표를 싣는 머리글 (BR-K14). */
const DEVICE_HEADER = "x-baeum-device";

/** 화면이 스스로 한 요청에 싣는 머리글. 자동 잠금 시간을 늘리지 않는다 (BR-K8). */
const QUIET_HEADER = "x-baeum-quiet";

/** ADR 0004: 호스트의 포트는 47800으로 고정이다. */
const HOST_PORT = 47800;

// 이 PC의 서버. 개발 중에는 Vite가 /api를 넘겨줌
const THIS_PC_URL = import.meta.env.DEV ? "" : `http://127.0.0.1:${HOST_PORT}`;

let hostUrl = THIS_PC_URL;
let deviceToken: string | null = null;
let onDeviceNotApproved: (() => void) | null = null;

/**
 * 접속 모드(ADR 0021)에서 다른 PC의 호스트로 요청을 보내게 한다.
 *
 * `token`은 승인을 요청하며 받은 기기 증표다. 요청마다 싣는다. 호스트가 승인받지 않은 기기라고
 * 답하면 `onRejected`를 부른다(예: 승인을 취소당함).
 */
export function connectToHost(
  address: string,
  token: string | null,
  onRejected: () => void,
): void {
  hostUrl = `http://${address}:${HOST_PORT}`;
  deviceToken = token;
  onDeviceNotApproved = onRejected;
}

/**
 * 호스트 요청이 실패했다.
 *
 * `message`는 직원에게 그대로 보여 줘도 되는 문장이다.
 */
export class HostError extends Error {
  /** HTTP 상태 코드. 호스트에 닿지 못했으면 0. */
  readonly status: number;
  /** 호스트가 보낸 오류 종류. 호스트의 오류 모양이 아니면 `null`. */
  readonly code: ErrorCode | null;
  /** 어긴 업무 규칙 번호. 예: `BR-M2` */
  readonly rule: string | null;
  /** 형식에 맞지 않는 칸들. */
  readonly fields: readonly FieldError[];
  /** 어긴 신청 규칙들(`RULES_BROKEN`). */
  readonly violations: readonly RuleViolationResponse[];

  constructor(
    status: number,
    message: string,
    code: ErrorCode | null = null,
    rule: string | null = null,
    fields: readonly FieldError[] = [],
    violations: readonly RuleViolationResponse[] = [],
  ) {
    super(message);
    this.name = "HostError";
    this.status = status;
    this.code = code;
    this.rule = rule;
    this.fields = fields;
    this.violations = violations;
  }
}

/**
 * 실패 응답의 상태 코드와 본문으로 {@link HostError}를 만든다.
 *
 * 본문이 호스트의 오류 모양(docs/conventions.md: 오류 응답 모양)이 아니면 일반 문구를 쓴다.
 */
export function hostErrorFrom(status: number, body: unknown): HostError {
  if (isErrorBody(body)) {
    return new HostError(
      status,
      body.message,
      body.code,
      body.rule,
      body.fields,
      body.violations ?? [],
    );
  }
  return new HostError(
    status,
    `호스트가 요청을 처리하지 못했습니다 (${status})`,
  );
}

interface ErrorBody {
  code: ErrorCode;
  message: string;
  rule: string | null;
  fields: FieldError[];
  /** 옛 호스트는 보내지 않는다. */
  violations?: RuleViolationResponse[];
}

function isErrorBody(body: unknown): body is ErrorBody {
  if (typeof body !== "object" || body === null) {
    return false;
  }
  const { code, message, rule, fields } = body as Record<string, unknown>;
  return (
    typeof code === "string" &&
    typeof message === "string" &&
    (rule === null || typeof rule === "string") &&
    Array.isArray(fields)
  );
}

type Method = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

/** {@link send}에 더 줄 것. */
interface SendOptions {
  /** 요청을 그만둘 때 쓴다. */
  signal?: AbortSignal;
  /** 화면이 스스로 한 요청인지. 주지 않으면 직원이 방금 화면을 만졌는지로 정한다. */
  quiet?: boolean;
}

/** 이 오류를 받으면 세션이 더는 쓸 수 없다. 로그인 화면으로 돌아간다. */
const SESSION_ENDED: readonly ErrorCode[] = [
  "LOGIN_REQUIRED",
  "ACCOUNT_UNAVAILABLE",
];

/**
 * 호스트에 요청을 보낸다. 실패하면 {@link HostError}를 던진다.
 *
 * ADR 0012: 모든 요청에 앱 버전을 싣는다.
 * BR-K14: 접속 모드면 기기 증표를 싣는다.
 * ADR 0014: 로그인했으면 세션 증표를 싣는다. 호스트가 세션을 끝냈다고 답하면 세션을 비우고,
 * 잠겼다고 답하면 화면을 잠근다.
 * BR-K8: 직원이 화면을 만지지 않는 동안의 요청은 조용한 요청으로 보내 잠금 시간을 늘리지 않는다.
 */
async function send(
  method: Method,
  path: string,
  body?: unknown,
  options: SendOptions = {},
): Promise<Response> {
  const quiet = options.quiet ?? !isStaffActive(Date.now());
  const headers: Record<string, string> = { [VERSION_HEADER]: APP_VERSION };
  if (deviceToken !== null) {
    headers[DEVICE_HEADER] = deviceToken;
  }
  if (quiet) {
    headers[QUIET_HEADER] = "1";
  }
  const session = getSession();
  if (session !== null) {
    headers.authorization = `Bearer ${session.token}`;
  }
  if (body !== undefined) {
    headers["content-type"] = "application/json";
  }

  let response: Response;
  try {
    response = await fetch(hostUrl + path, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: options.signal,
    });
  } catch {
    // Wi-Fi가 끊기거나 호스트가 꺼져 있음
    throw new HostError(
      0,
      "호스트에 연결하지 못했습니다. 연결을 확인해 주세요",
    );
  }

  // 401(로그인 필요, 화면 잠김, PIN 틀림)이 아니면 호스트가 세션을 받아 준 것.
  // 조용한 요청은 호스트도 시각을 늘리지 않으니 화면도 늘리지 않는다
  if (session !== null && response.status !== 401 && !quiet) {
    noteRequest();
  }

  if (!response.ok) {
    const errorBody: unknown = await response.json().catch(() => null);
    const error = hostErrorFrom(response.status, errorBody);
    // 증표를 실은 요청만. 로그인 요청의 ACCOUNT_UNAVAILABLE은 세션과 상관없다
    if (session !== null && error.code !== null) {
      if (SESSION_ENDED.includes(error.code)) {
        clearSession();
      } else if (error.code === "SESSION_LOCKED") {
        lockSession();
      }
    }
    if (error.code === "DEVICE_NOT_APPROVED" && onDeviceNotApproved !== null) {
      clearSession();
      onDeviceNotApproved();
    }
    throw error;
  }
  return response;
}

/**
 * 오래 열어 두는 GET 요청(SSE)을 연다. 응답 본문은 부른 쪽이 읽는다.
 *
 * 조용한 요청으로 보낸다. 알림 연결을 열거나 다시 여는 것은 직원이 한 요청이 아니다 (BR-K8).
 */
export function openStream(
  path: string,
  signal: AbortSignal,
): Promise<Response> {
  return send("GET", path, undefined, { signal, quiet: true });
}

/** GET 요청을 보내고 JSON을 받는다. */
export async function getJson<T>(path: string): Promise<T> {
  const response = await send("GET", path);
  return (await response.json()) as T;
}

/** POST 요청으로 JSON을 보내고 JSON을 받는다. */
export async function postJson<T>(path: string, body: unknown): Promise<T> {
  const response = await send("POST", path, body);
  return (await response.json()) as T;
}

/** PATCH 요청으로 JSON을 보내고 JSON을 받는다. */
export async function patchJson<T>(path: string, body: unknown): Promise<T> {
  const response = await send("PATCH", path, body);
  return (await response.json()) as T;
}

/** PUT 요청으로 JSON을 보내고 JSON을 받는다. */
export async function putJson<T>(path: string, body: unknown): Promise<T> {
  const response = await send("PUT", path, body);
  return (await response.json()) as T;
}

/** POST 요청을 보낸다. 성공하면 본문이 없다(204). */
export async function postNoContent(
  path: string,
  body?: unknown,
): Promise<void> {
  await send("POST", path, body);
}

/** POST 요청으로 JSON을 보내고 파일 내용(바이트)을 받는다. 예: Excel 명단 */
export async function postForFile(
  path: string,
  body: unknown,
): Promise<Uint8Array> {
  const response = await send("POST", path, body);
  return new Uint8Array(await response.arrayBuffer());
}

/** DELETE 요청을 보낸다. 본문(예: 지우는 사유)은 선택. 성공하면 본문이 없다(204). */
export async function sendDelete(path: string, body?: unknown): Promise<void> {
  await send("DELETE", path, body);
}
