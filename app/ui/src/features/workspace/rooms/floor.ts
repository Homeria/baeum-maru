/** 고를 수 있는 층. 지하 2층부터 5층까지, 0층은 없다. 호스트의 `Floor`와 같은 범위. */
export const FLOORS: readonly number[] = [-2, -1, 1, 2, 3, 4, 5];

/** 층 번호를 화면 글자로 바꾼다. `-1`은 "지하 1층", 층이 없으면 빈 글자. */
export function floorLabel(floor: number | null): string {
  if (floor === null) {
    return "";
  }
  return floor < 0 ? `지하 ${-floor}층` : `${floor}층`;
}
