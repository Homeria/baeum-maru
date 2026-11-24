# 배움마루 웹사이트

기관 직원과 도입을 검토하는 사람을 위한 소개, 다운로드 안내, 블로그 웹사이트다. 위치와 배포 원칙은 [ADR 0019](../docs/adr/0019-website.md)를 따른다.

## 실행과 검사 명령

Node.js 22(`.node-version`)가 필요하다. 처음 한 번과 `package-lock.json`이 바뀌었을 때 패키지를 설치한다.

```sh
just site-setup                # 또는 npm ci --prefix site
```

자주 쓰는 명령:

```sh
just site-dev                  # 로컬 개발 서버 (npm --prefix site run dev)
just site-preview              # 빌드 결과물 로컬 미리보기 (npm --prefix site run preview)
just site-check                # 전체 검사, 커밋·PR 전에 실행 (npm --prefix site run check)
npm --prefix site run build    # 정적 사이트 빌드 (site/dist/)
```

`just site-check`(`npm --prefix site run check`)가 검사하는 것:

1. Prettier 형식 확인 (`format:check`)
2. Astro·TypeScript 타입 검사 (`astro check`)
3. 정적 사이트 빌드 (`astro build`)
4. 빌드 결과물(`site/dist/`)의 내부 링크와 이미지 경로 깨짐 확인 (`scripts/check-links.mjs`)

외부 링크는 네트워크 상태에 따라 거짓 실패가 나기 쉬워 CI에서 빼고 손으로 확인한다.

## 폴더 구조

```text
site/
├── README.md              # 이 안내 문서
├── TERMS.md               # 기술 용어의 쉬운 말 대응표
├── SCREENSHOTS.md         # 스크린샷별 화면 이름과 기준 PR 번호
├── astro.config.mjs       # Astro 설정 (GitHub Pages 하위 경로 포함)
├── scripts/
│   └── check-links.mjs    # 빌드 결과물의 내부 링크·이미지 검사
├── public/                # 그대로 복사되는 정적 파일 (favicon 등)
└── src/
    ├── config/            # 저장소 주소, 출시 전 여부 등 사이트 설정
    ├── layouts/           # 공통 머리글·메뉴·바닥글 틀
    ├── components/        # 안내 상자 등 재사용 부품
    ├── styles/            # 전역 스타일
    ├── assets/
    │   └── screenshots/   # 앱 화면 스크린샷
    ├── content/
    │   └── posts/         # 블로그 글 마크다운
    └── pages/             # 페이지별 경로
```

## 스크린샷 규칙

- 반드시 `just seed`로 넣은 더미 데이터 화면만 찍는다. 실제 회원 정보와 실제 기관 이름이 들어간 사진은 올리지 않는다.
- 유지보수자가 찍어 `site/src/assets/screenshots/`에 넣는다.
- 형식은 **WebP**를 기본으로 한다. 투명 배경이나 선명한 글자가 꼭 필요해 WebP가 깨질 때만 PNG를 쓴다.
- 파일 크기는 **사진당 300KB 이하**, 가로 크기는 **1600px 이하**로 맞춘다.
- 사진을 넣거나 바꾸면 [SCREENSHOTS.md](SCREENSHOTS.md)에 파일 이름, 화면 이름, 기준 PR 번호를 함께 적는다.

## 글쓰기와 용어

- 업무 용어는 [용어집](../docs/domain/GLOSSARY.md)을 그대로 쓴다. 다운로드 페이지와 블로그에서 배포 용어가 처음 나올 때는 용어집 "배포" 절의 풀이를 괄호로 붙인다.
- 기술 용어는 [TERMS.md](TERMS.md)의 쉬운 말 대응표를 따른다.
