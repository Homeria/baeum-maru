import type { MaskCharCode } from "../../../api/generated/MaskCharCode";
import type { RosterColumnCode } from "../../../api/generated/RosterColumnCode";
import type { RosterExportRequest } from "../../../api/generated/RosterExportRequest";
import type { RosterKindCode } from "../../../api/generated/RosterKindCode";

/** 명단 종류. 화면에 보여 줄 순서다(BR-P1). */
export const ROSTER_KINDS: readonly RosterKindCode[] = [
  "posted_by_course",
  "posted_by_member",
  "raw",
];

export const ROSTER_KIND_LABELS: Record<RosterKindCode, string> = {
  raw: "원본 명단",
  posted_by_course: "게시용 강좌별",
  posted_by_member: "게시용 회원별",
};

export const COLUMN_LABELS: Record<RosterColumnCode, string> = {
  course_name: "강좌",
  capacity: "정원",
  result: "결과",
  waitlist_number: "대기 순번",
  name: "이름",
  member_no: "회원번호",
  phone_number: "연락처",
  birth_date: "생년월일",
  gender: "성별",
  guaranteed: "우선(자리 보장 여부)",
};

export const MASK_CHARS: readonly MaskCharCode[] = ["circle", "asterisk", "x"];

export const MASK_CHAR_LABELS: Record<MaskCharCode, string> = {
  circle: "○",
  asterisk: "*",
  x: "X",
};

/**
 * 명단 종류마다 고를 수 있는 것. 호스트의 `RosterKind`와 같은 표다(BR-P2~P6).
 * 화면은 고를 칸만 보여 줄 뿐이고, 실제로 막는 것은 호스트다.
 */
interface KindRule {
  /** 늘 넣는 칸. */
  fixed: readonly RosterColumnCode[];
  /** 넣을지 고르는 칸. */
  optional: readonly RosterColumnCode[];
  /** 가릴지 고르는 칸. */
  maskable: readonly RosterColumnCode[];
  /** 넣으면 늘 가리는 칸. */
  alwaysMasked: readonly RosterColumnCode[];
}

/** 세 종류 모두 늘 넣는 칸. 강좌 이름과 정원은 시트 이름이나 제목에 들어간다. */
const FIXED: readonly RosterColumnCode[] = [
  "course_name",
  "capacity",
  "result",
  "waitlist_number",
  "name",
  "member_no",
];

const POSTED: KindRule = {
  fixed: FIXED,
  optional: ["phone_number", "birth_date", "gender"],
  maskable: ["name", "member_no"],
  alwaysMasked: ["phone_number", "birth_date"],
};

export const KIND_RULES: Record<RosterKindCode, KindRule> = {
  raw: {
    fixed: FIXED,
    optional: ["phone_number", "birth_date", "gender"],
    maskable: ["name", "member_no", "phone_number", "birth_date"],
    alwaysMasked: [],
  },
  posted_by_course: POSTED,
  posted_by_member: POSTED,
};

/** 창을 열 때 늘 꺼 두는 칸(BR-P9). */
const PERSONAL: readonly RosterColumnCode[] = [
  "phone_number",
  "birth_date",
  "gender",
];

/** 창에서 고른 것. 명단 종류마다 따로 둔다. */
export interface RosterChoices {
  /** 넣기로 고른 칸. 늘 넣는 칸은 적지 않는다. */
  include: readonly RosterColumnCode[];
  /** 가리기로 고른 칸. */
  mask: readonly RosterColumnCode[];
  maskChar: MaskCharCode;
}

/** 처음 쓰는 PC의 선택(BR-P2, BR-P6). */
export function defaultChoices(kind: RosterKindCode): RosterChoices {
  return {
    include: [],
    mask: kind === "raw" ? [] : ["name"],
    maskChar: "circle",
  };
}

/** PC에 기억할 곳. 브라우저의 `localStorage`. */
export type ChoiceStorage = Pick<Storage, "getItem" | "setItem">;

const STORAGE_KEY = "baeum.rosterExport";

interface Remembered {
  kind?: unknown;
  choices?: Partial<Record<string, unknown>>;
}

function readRemembered(storage: ChoiceStorage): Remembered {
  try {
    const parsed: unknown = JSON.parse(storage.getItem(STORAGE_KEY) ?? "{}");
    return typeof parsed === "object" && parsed !== null ? parsed : {};
  } catch {
    // 고장 난 값은 처음 쓰는 PC처럼
    return {};
  }
}

/** 지난번에 고른 명단 종류. 고를 수 없는 종류였으면 첫 번째로 고를 수 있는 종류. */
export function rememberedKind(
  storage: ChoiceStorage,
  allowed: readonly RosterKindCode[],
): RosterKindCode {
  const { kind } = readRemembered(storage);
  return allowed.find((candidate) => candidate === kind) ?? allowed[0];
}

/**
 * 이 PC가 기억한 선택. 연락처, 생년월일, 성별은 기억했어도 끈 채로 돌려준다(BR-P9).
 * 기억한 것이 없거나 지금 규칙에 맞지 않는 값은 버리고 처음 선택을 쓴다.
 */
export function rememberedChoices(
  storage: ChoiceStorage,
  kind: RosterKindCode,
): RosterChoices {
  const saved = readRemembered(storage).choices?.[kind];
  const fallback = defaultChoices(kind);
  if (typeof saved !== "object" || saved === null) {
    return fallback;
  }
  const { include, mask, maskChar } = saved as Record<string, unknown>;
  const rule = KIND_RULES[kind];
  const pick = (value: unknown, from: readonly RosterColumnCode[]) =>
    Array.isArray(value)
      ? from.filter((column) => value.includes(column))
      : null;
  return {
    include: (pick(include, rule.optional) ?? fallback.include).filter(
      (column) => !PERSONAL.includes(column),
    ),
    mask: pick(mask, rule.maskable) ?? fallback.mask,
    maskChar:
      MASK_CHARS.find((candidate) => candidate === maskChar) ??
      fallback.maskChar,
  };
}

/** 내보낸 선택을 이 PC에 기억한다. 연락처, 생년월일, 성별을 넣었는지는 기억하지 않는다. */
export function rememberChoices(
  storage: ChoiceStorage,
  kind: RosterKindCode,
  choices: RosterChoices,
): void {
  const remembered = readRemembered(storage);
  const saved: RosterChoices = {
    ...choices,
    include: choices.include.filter((column) => !PERSONAL.includes(column)),
  };
  storage.setItem(
    STORAGE_KEY,
    JSON.stringify({
      kind,
      choices: { ...remembered.choices, [kind]: saved },
    }),
  );
}

/** 칸을 넣었는지. 늘 넣는 칸이면 `true`. */
export function isIncluded(
  kind: RosterKindCode,
  choices: RosterChoices,
  column: RosterColumnCode,
): boolean {
  return (
    KIND_RULES[kind].fixed.includes(column) || choices.include.includes(column)
  );
}

/** 호스트에 보낼 요청. 넣지 않은 칸의 가림은 보내지 않는다. */
export function exportRequest(
  kind: RosterKindCode,
  choices: RosterChoices,
): RosterExportRequest {
  return {
    kind,
    include: [...choices.include],
    mask: choices.mask.filter((column) => isIncluded(kind, choices, column)),
    maskChar: choices.maskChar,
  };
}

/** 저장 창에 미리 채울 파일 이름. 예: `2027년 1학기 게시용 강좌별.xlsx` */
export function rosterFileName(termName: string, kind: RosterKindCode): string {
  return `${termName} ${ROSTER_KIND_LABELS[kind]}.xlsx`;
}

/** 목록에서 `column`을 켜거나 끈다. 순서는 `order`를 따른다. */
export function toggle(
  list: readonly RosterColumnCode[],
  column: RosterColumnCode,
  on: boolean,
  order: readonly RosterColumnCode[],
): RosterColumnCode[] {
  return order.filter((candidate) =>
    candidate === column ? on : list.includes(candidate),
  );
}
