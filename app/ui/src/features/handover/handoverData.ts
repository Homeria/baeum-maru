import type { HandoverFileResponse } from "../../api/generated/HandoverFileResponse";

/** 넘기기 파일 확인 화면의 줄들. 개인정보 없이 학기와 수만 보여 준다. */
export function fileLines(file: HandoverFileResponse): string[] {
  const terms =
    file.termNames.length === 0 ? "없음" : file.termNames.join(", ");
  return [
    `학기: ${terms}`,
    `회원 ${file.memberCount}명, 신청 ${file.applicationCount}건`,
  ];
}
