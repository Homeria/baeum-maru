import type { ErrorCode } from "./generated/ErrorCode";
import type { FieldError } from "./generated/FieldError";
import type { RuleViolationResponse } from "./generated/RuleViolationResponse";
import { clearSession, getSession, lockSession, noteRequest } from "./session";

/** 이 앱의 버전. 요청마다 호스트에 알린다. */
export const APP_VERSION = __APP_VERSION__;

const VERSION_HEADER = "x-baeum-version";

// 개발 중에는 Vite가 /api를 넘겨줌
// 호스트·접속 선택이 생길 때까지는 이 PC의 서버
const HOST_URL = import.meta.env.DEV ? "" : "http://127.0.0.1:47800";

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

type Method = "GET" | "POST" | "PATCH" | "DELETE";

/** 이 오류를 받으면 세션이 더는 쓸 수 없다. 로그인 화면으로 돌아간다. */
const SESSION_ENDED: readonly ErrorCode[] = [
  "LOGIN_REQUIRED",
  "ACCOUNT_UNAVAILABLE",
];

/**
 * 호스트에 요청을 보낸다. 실패하면 {@link HostError}를 던진다.
 *
 * ADR 0012: 모든 요청에 앱 버전을 싣는다.
 * ADR 0014: 로그인했으면 세션 증표를 싣는다. 호스트가 세션을 끝냈다고 답하면 세션을 비우고,
 * 잠겼다고 답하면 화면을 잠근다.
 */
async function send(
  method: Method,
  path: string,
  body?: unknown,
): Promise<Response> {
  const headers: Record<string, string> = { [VERSION_HEADER]: APP_VERSION };
  const session = getSession();
  if (session !== null) {
    headers.authorization = `Bearer ${session.token}`;
  }
  if (body !== undefined) {
    headers["content-type"] = "application/json";
  }

  let response: Response;
  try {
    response = await fetch(HOST_URL + path, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch {
    // Wi-Fi가 끊기거나 호스트가 꺼져 있음
    throw new HostError(
      0,
      "호스트에 연결하지 못했습니다. 연결을 확인해 주세요",
    );
  }

  // 401(로그인 필요, 화면 잠김, PIN 틀림)이 아니면 호스트가 세션을 받아 준 것
  if (session !== null && response.status !== 401) {
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
    throw error;
  }
  return response;
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

/** POST 요청을 보낸다. 성공하면 본문이 없다(204). */
export async function postNoContent(
  path: string,
  body?: unknown,
): Promise<void> {
  await send("POST", path, body);
}

/** DELETE 요청을 보낸다. 본문(예: 지우는 사유)은 선택. 성공하면 본문이 없다(204). */
export async function sendDelete(path: string, body?: unknown): Promise<void> {
  await send("DELETE", path, body);
}
