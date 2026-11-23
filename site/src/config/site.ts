/**
 * 웹사이트 전역 설정.
 *
 * 저장소 이전이나 출시 전환 시 이 파일에서 주소와 상태를 바꾼다.
 */
export const SITE = {
  name: "배움마루",
  description:
    "소규모 교육·여가 프로그램 운영 기관을 위한 무료 수강 사무 프로그램",
  repositoryUrl: "https://github.com/Homeria/baeum-maru",
  releasesUrl: "https://github.com/Homeria/baeum-maru/releases",
  issuesUrl: "https://github.com/Homeria/baeum-maru/issues",
  /** ADR 0019: 출시 전까지는 안내 상자를 보여 주고 가짜 다운로드 버튼은 두지 않는다. */
  released: false,
  /** AGENTS.md: AI 도움은 사이트 소개에 한 번만 밝힌다. */
  aiNotice: "AI 도구의 도움을 받아 정리하고 제작자가 확인합니다.",
} as const;

/** 상단 메뉴 항목. */
export interface NavItem {
  label: string;
  path: string;
}

/** 공통 머리글과 바닥글에 쓰는 메뉴 목록. */
export const NAV_ITEMS: readonly NavItem[] = [
  { label: "홈", path: "/" },
  { label: "다운로드", path: "/download/" },
  { label: "이야기", path: "/posts/" },
  { label: "소개", path: "/about/" },
];

/**
 * GitHub Pages 하위 경로(base)를 붙인 내부 경로를 만든다.
 *
 * @param path `/`로 시작하는 사이트 안 경로. 예: `/download/`
 */
export function withBase(path: string): string {
  const base = import.meta.env.BASE_URL.replace(/\/$/, "");
  const clean = path.startsWith("/") ? path : `/${path}`;
  return `${base}${clean}`;
}
