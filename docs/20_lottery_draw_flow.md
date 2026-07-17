# 20. 추첨(Lottery) 실행 흐름

추첨을 백엔드·프론트·DB 분리 환경에서 **공정하고 재현 가능하게** 실행하는 방식을 정한다. 스키마는 이 흐름을 기준으로 구성한다.

## 배경

과거 PyQt 단일 앱에서는 화면에서 shuffle을 주기적으로 렌더링하는 것이 곧 추첨이었고, 저장 버튼으로 결과를 보관했다. 이제 백엔드·프론트·DB가 분리되므로, 추첨 결과의 **재현성·공정성·일관성**을 DB로 보장해야 한다.

## 핵심 원칙

**결과는 백엔드가 seed로 확정하고, 프론트는 그 결과를 향해 애니메이션(연출)만 한다.** (슬롯머신과 동일 — 당첨은 이미 정해져 있고 돌아가는 건 연출)

- "무엇이 뽑혔나"(백엔드) 와 "어떻게 보여주나"(프론트) 를 분리한다.
- **seed는 반드시 백엔드가 생성한다.** 프론트가 seed를 만들면 원하는 결과가 나올 때까지 고르거나(체리피킹), 알고리즘을 알면 특정인을 당첨시키는 seed를 역산할 수 있어 공정성이 깨진다.

## 실행 흐름 (미리보기 → 연출 → 확정)

```
1. 신청 마감            term.status = 'closed'  (추첨 대상 풀 고정)
2. GET  강의별 신청자 목록                       (프론트: 후보 렌더링)
3. POST /lottery/preview { term }               (백엔드: seed 생성 + 결과 계산, 저장 X)
        → 응답: { seed, 강의별[ 당첨 / 대기(순번) / 탈락 ] }
4. 프론트: 받은 결과를 향해 셔플 애니메이션 → 결과 렌더링, seed 보관
5. POST /lottery/commit { term, seed }          (저장 버튼)
        → 백엔드: 같은 seed로 재계산(결정적) → 원자적 저장 + registrations 반영
```

`seed`를 응답 → 저장 요청으로 그대로 전달하는 것이 핵심이다. commit이 동일 seed로 재계산하므로 **"본 것과 저장된 것이 100% 동일"**하게 보장된다.

## 왜 이 구조인가

- **공정성**: 랜덤의 원천(seed)을 서버가 쥐므로 아무도 결과를 미리 고를 수 없다.
- **재현성**: seed를 DB에 저장하므로 나중에 동일 추첨을 그대로 재현·검증할 수 있다.
- **일관성**: 프론트가 결과를 정하지 않으므로 화면과 DB가 어긋날 수 없다.

## commit = 원자적 트랜잭션

저장 한 번에 여러 도메인이 함께 바뀌어야 하며, 전부 성공하거나 전부 취소되어야 한다:

- `lottery_runs` + `lottery_run_targets`(정원·eligible 스냅샷) + `lottery_results` 기록
- `registrations.status` → selected/waitlisted/rejected 반영, `registrations.waitlist_order` = 추첨 순번(`result_order`)으로 초기화
- 감사 로그

lottery·registrations·operations를 엮는 cross-repo 원자 작업이므로 [문서 19](19_repository_transaction_and_audit.md)의 "도메인 repo가 커넥션 소유 + `insert_*(conn)` 주입" 패턴을 따른다. 동시 실행은 단일 서버 프로세스의 인메모리 잠금, 중복 저장은 도메인 UNIQUE 제약으로 방지한다(단일 프로세스라 DB 락/멱등성 테이블은 두지 않음).

## 미저장 미리보기 채택 (설계 결정)

preview는 **계산만 하고 저장하지 않는다.** 저장은 commit 때만 일어난다.

- 신청이 마감된 뒤 추첨하므로 대상 풀이 고정되어, seed만으로 재현이 성립한다.
- 버려지는 임시(prepared) 데이터가 생기지 않는다.
- 결과적으로 **저장된 `lottery_runs`는 항상 "완료된 추첨"** 이다. 그래서 스키마에서 `prepared`/`running` 같은 실행 라이프사이클 상태와 `started_at`/`completed_at`을 두지 않는다.

## 재추첨(re-roll) 정책 — 운영 결정 필요

preview를 다시 요청하면 새 seed로 다른 결과가 나온다. "맘에 들 때까지 돌리기"는 공정성을 해치므로 정책을 정해야 한다:

- **1회 고정(가장 공정)**: term당 한 번만. → `lottery_runs`에 `UNIQUE(term_id)` 추가.
- **재추첨 허용 + 전부 기록**: 모든 실행(seed·시각·실행자)을 감사에 남겨 투명성 확보.
- **승인 후 재추첨**: 사유를 남기고만 재추첨.

현재 스키마는 정책 미정이라 `UNIQUE(term_id)`를 걸지 않았다. 정해지면 반영한다.

## 스키마 매핑

| 요소 | 위치 | 역할 |
| --- | --- | --- |
| `lottery_runs.seed` | lottery | 백엔드 생성, 재현 소스 |
| `lottery_run_targets` | lottery | commit 시점 정원 + eligible 스냅샷 (gender_split은 `eligible_male`/`eligible_female`) |
| `lottery_results.result_order` | lottery | 추첨 순번(불변) → `registrations.waitlist_order` 초기화 |
| `lottery_results.result` | lottery | selected/waitlisted/rejected → `registrations.status` 반영 |

## 요약

1. 결과는 백엔드가 seed로 확정, 프론트는 결과를 향해 애니메이션.
2. seed는 백엔드 생성(프론트 금지). 응답 → commit으로 전달해 동일 재현.
3. preview는 미저장, commit만 원자적으로 저장 + registrations 반영.
4. 재추첨 정책은 운영 결정 사항.
