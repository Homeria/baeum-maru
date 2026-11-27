import type { ContactResultCode } from "../../../api/generated/ContactResultCode";
import type { ResultMemberResponse } from "../../../api/generated/ResultMemberResponse";
import type { WithdrawnResponse } from "../../../api/generated/WithdrawnResponse";
import type { WithdrawnFromCode } from "../../../api/generated/WithdrawnFromCode";

/** 연락 결과의 화면 이름. 용어집의 이름과 같다(BR-W2). */
export const CONTACT_LABELS: Record<ContactResultCode, string> = {
  accepted: "수락",
  declined: "거절",
  unreachable: "연락 안 됨",
};

/** 포기 전 상태의 화면 이름. */
export const WITHDRAWN_FROM_LABELS: Record<WithdrawnFromCode, string> = {
  selected: "당첨에서 포기",
  waitlisted: "대기에서 포기",
};

/** 결과 목록의 한 줄. 예: `회원1(12-12345)` */
export function memberLabel(member: ResultMemberResponse): string {
  return `${member.memberName}(${member.memberNo})`;
}

/**
 * 이름 검색 칸의 글자와 맞는 회원인지. 이름이나 회원번호의 일부면 맞다.
 *
 * 앞뒤 공백과 사이 띄어쓰기는 무시한다. 빈 글자는 아무도 맞지 않는다.
 */
export function matchesQuery(
  member: ResultMemberResponse,
  query: string,
): boolean {
  const key = withoutSpaces(query);
  if (key === "") {
    return false;
  }
  return (
    withoutSpaces(member.memberName).includes(key) ||
    member.memberNo.includes(key)
  );
}

function withoutSpaces(text: string): string {
  return text.replace(/\s+/g, "");
}

/** 왼쪽 강좌 목록의 빈자리 글자. 예: `빈자리 2` */
export function openSeatsLabel(openSeats: number): string {
  return openSeats === 0 ? "빈자리 없음" : `빈자리 ${openSeats}`;
}

/**
 * 잘못 처리 바로잡기로 되돌아갈 곳(BR-W8). 예: `당첨`, `대기 2번`
 *
 * 당첨에서 포기했으면 당첨으로, 대기에서 포기했으면 추첨 때 받은 순번으로 간다.
 */
export function restoreLabel(entry: WithdrawnResponse): string {
  if (entry.withdrawnFrom === "selected" || entry.waitlistNumber === null) {
    return "당첨";
  }
  return `대기 ${entry.waitlistNumber}번`;
}
