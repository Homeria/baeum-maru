# 19. Repository 트랜잭션 · 감사(Audit) 규약

router → service → repository → sqlite3 레이어 구조에서 **트랜잭션 경계와 감사 로그를 어디서 어떻게 다룰지**에 대한 결정을 기록한다. 아직 코드에 전면 반영하지 않았으며(브랜치 범위와 맞지 않음), 이후 repository/service를 구현할 때 이 규약을 따른다.

## 0. 배경과 목표

- **sqlite3 의존은 repository 레이어에만 둔다.** service/router는 sqlite3, `connection`, `transaction`을 직접 다루지 않는다. (아키텍처 테스트가 `service`의 `db`/`sqlite3` import를 금지하는 것과 동일한 목표.)
- 그러면서도 **여러 테이블에 걸친 쓰기의 원자성**(전부 아니면 전무)을 보장해야 한다.
- **감사 로그**는 거의 모든 상태 변경에 따라붙는 횡단 관심사다.
- 대상 규모가 작으므로(동시 사용자 2~5명, SQLite, 커넥션 풀 없음) 성능보다 **가독성과 일관성**을 우선한다.

## 1. 기본 원칙: 커넥션은 repository 함수가 얻는다

Python `sqlite3`는 커넥션 풀이 없고 이 규모에선 함수 호출마다 커넥션을 여닫아도 부담이 없다. 따라서 기본 스타일은 **"repository 함수가 자기 커넥션을 얻어 처리"** 한다.

### 1-1. 단건 쓰기
한 함수에서 커넥션 하나 → execute → commit.

```python
def add_building(name, description=None):
    with get_db_connection() as conn:
        try:
            cur = conn.execute(
                "INSERT INTO buildings(name, description) VALUES (?, ?)",
                (name, description),
            )
            conn.commit()
            return cur.lastrowid
        except sqlite3.IntegrityError as e:
            conn.rollback()
            raise ConflictError(...) from e
```

### 1-2. 단일 repository 안의 원자적 다중 쓰기
같은 repository가 소유한 여러 테이블을 원자적으로 쓸 때는 **한 함수 안에서 여러 번 execute 후 마지막에 commit 한 번**. 중간에 예외가 나면 rollback.

```python
def create_registration(member_id, offering_id):
    with get_db_connection() as conn:
        try:
            reg_id = conn.execute("INSERT INTO registrations ...", ...).lastrowid
            conn.execute("INSERT INTO registration_status_history ...", ...)
            conn.commit()
            return reg_id
        except sqlite3.Error:
            conn.rollback()
            raise
```

> **금지 패턴:** 원자적이어야 하는 작업을 `insert_a()`(내부 commit) + `insert_b()`(내부 commit)처럼 **커밋하는 함수를 연달아 호출**하지 않는다. 커넥션이 둘로 갈라져 반쪽 커밋 상태가 생긴다. 원자 단위는 반드시 한 트랜잭션 안에 모은다.

## 2. Cross-repository 원자 쓰기: 커넥션 주입

여러 repository가 소유한 테이블을 한 트랜잭션으로 묶어야 할 때는, **도메인 repository가 커넥션을 소유하고, 다른 repository의 "참여형 함수"에 그 커넥션을 주입**한다. 커넥션은 repository 층 안에서만 오가며 service로 넘어가지 않는다.

실행 순서:

1. 도메인 repository가 `with get_db_connection()`으로 **커넥션 획득** 후 자기 테이블에 execute.
2. 그 **커넥션을 다른 repository의 참여형 함수에 인자로 주입**한다.
3. 참여형 함수는 주입받은 커넥션에 execute만 하고 **commit 하지 않고 return**한다.
4. 제어가 도메인 repository로 돌아오면 **여기서 한 번만 commit** 한다. (중간 예외 시 rollback → 전체 취소)

```python
# operation_repository.py — 참여형 부품 (commit 안 함)
def insert_audit_log(conn, *, actor, action, resource_type, resource_id, summary, metadata=None):
    conn.execute("INSERT INTO audit_logs(...) VALUES (...)", (...))

# registration_repository.py — 트랜잭션 소유자
def create_registration(member_id, offering_id, audit):   # audit은 plain 데이터(dict/dataclass)
    with get_db_connection() as conn:                     # 1) 커넥션 획득
        try:
            reg_id = conn.execute("INSERT INTO registrations ...", ...).lastrowid  # 1) 도메인 쓰기
            operation_repo.insert_audit_log(conn, **audit)                          # 2) conn 주입
            conn.commit()                                 # 4) 감사까지 끝났으니 여기서 commit
            return reg_id
        except sqlite3.Error:
            conn.rollback()
            raise
```

```python
# service — sqlite3/conn을 전혀 만지지 않는다
def submit_registration(member_id, offering_id, actor):
    audit = {
        "actor": actor,
        "action": "registration.create",
        "resource_type": "registration",
        "summary": "수강신청 접수",
    }
    return registration_repo.create_registration(member_id, offering_id, audit)
```

- 커밋은 **커넥션을 연 도메인 repository가 마지막에 한 번**. 주입받은 참여형 함수는 절대 commit/rollback 하지 않는다.
- service는 "무엇을 감사할지"(action·summary 등 도메인 지식)만 **plain 데이터로 하향 전달**한다. sqlite3는 repository 층에만 머문다.

## 3. Repository 함수의 두 갈래

횡단 관심사(대표적으로 감사)를 제공하는 repository는 함수를 두 형태로 둔다. 핵심 로직은 공유한다.

| 형태 | 트랜잭션 | 용도 |
| --- | --- | --- |
| `insert_x(conn, ...)` (참여형) | 주입받은 커넥션에 execute만, commit 안 함 | 다른 repository 트랜잭션에 **합류** |
| `add_x(...)` (단독형) | 자기 커넥션을 열고 commit | 남의 트랜잭션이 없을 때 **단독 실행** |

```python
def add_audit_log(**kwargs):
    with get_db_connection() as conn:
        try:
            insert_audit_log(conn, **kwargs)
            conn.commit()
        except sqlite3.Error:
            conn.rollback()
            raise
```

## 4. 감사(Audit) 규칙

### 4-1. 무엇을 감사하는가
- **감사 O — 상태 변경(쓰기):** 회원/강좌/개설/건물·장소 생성·수정·삭제, 수강신청·취소·상태변경, 추첨 실행, 출석 기록·수정, 로그인/로그아웃·접속코드 발급·폐기, 백업/복원, 엑셀 가져오기/내보내기.
- **감사 X — 읽기·잡음:** 피드·상세·검색·목록 조회, 세션 heartbeat, `last_seen_at` 갱신, health/OpenAPI.

### 4-2. 어느 트랜잭션에 기록하는가
- **성공한 상태 변경** → 업무 쓰기와 **같은 트랜잭션** (`insert_audit_log(conn)`).
  - "변경은 됐는데 감사 없음" 또는 "감사엔 있는데 변경은 롤백됨"을 원천 차단한다.
- **실패·거부된 시도, 읽기 감사, 보안 이벤트** → **별도 트랜잭션** (`add_audit_log(...)`).
  - 업무 트랜잭션이 롤백되므로 그 안에 넣으면 실패 기록까지 사라진다. 반드시 밖에서 별도 커밋해야 한다.
  - 예: 마감된 강좌 신청 거부, 로그인 실패, 개인정보 열람 기록.

### 4-3. 행위자(actor) 배선
`actor_kind`, `actor_user_id`, `actor_display_name`, `actor_access_code_id`는 인증 세션에서 나와 **router → service → repository**로 내려간다. 매번 인자로 끌고 다니지 말고 **`ActorContext` 같은 작은 dataclass 하나로 묶어** 전달한다. sqlite3가 아니라 순수 데이터이므로 레이어 규칙을 어기지 않는다.

## 5. 원자성으로 묶을지는 업무 결정

무엇을 "전부 아니면 전무"로 볼지는 기술 규칙이 아니라 **업무 설계**다. 현재 기준으로 원자적으로 다룰 후보:

- **다중 강좌 수강신청** — 한 요청의 여러 신청을 전부 성공/전부 취소로 볼지, 되는 것만 처리(부분 성공)할지 결정한다.
- **추첨 실행** — 반드시 원자적. `lottery_runs`/`lottery_run_targets`/`lottery_results` 생성 + `registrations.status` 갱신 + 상태이력 + 감사를 한 트랜잭션으로 묶는다. 추첨 **알고리즘**(seed 기반 배정 계산)은 DB 접근 없는 **순수 로직으로 service에** 두고, 계산된 "배정 계획"을 plain 데이터로 repository에 넘겨 한 트랜잭션에 persist한다. 중간 레이스는 단일 서버 프로세스의 인메모리 잠금으로 방지한다.
- **중복 요청 방지** — 생성류 요청의 중복 저장은 도메인 UNIQUE 제약(예: `registrations`의 member+offering)과 인메모리 가드로 막는다. 서버가 단일 프로세스라 DB 멱등성 테이블은 두지 않는다.

## 6. 트레이드오프와 향후 여지

- Cross-repo 트랜잭션은 **repository → repository 의존**을 만든다(예: `registration_repository` → `operation_repository`). 같은 레이어 간 의존이므로 허용되며, 감사 같은 횡단 관심사에선 정상적인 fan-in이다. (금지되는 것은 repository → service/api 같은 상향 의존.)
- 지금은 `with get_db_connection()` + 수동 `commit/rollback`을 기본으로 한다. **같은 try/commit/except/rollback 보일러플레이트가 반복되어 실수 위험이 커지면**, 그때 `transaction()` 컨텍스트 매니저 헬퍼로 추출한다. 능력 차이는 없고 DRY·실수 방지 목적이며, 그 경우에도 트랜잭션 소유는 repository 층에 유지한다.
- 마음에 들지 않으면 이후 리팩터링한다. 이 문서는 그 시점의 출발 규약이다.

## 요약

1. 커넥션은 repository가 얻는다. 단건은 한 함수에서 execute→commit.
2. 원자적 다중 쓰기는 한 트랜잭션 안에 모은다. 커밋하는 함수를 연쇄 호출하지 않는다.
3. Cross-repo(특히 감사)는 도메인 repository가 커넥션을 소유하고 참여형 `insert_x(conn)`에 주입, 마지막에 도메인 repository가 한 번 commit.
4. 감사는 성공한 쓰기와 같은 트랜잭션, 실패·읽기·보안 이벤트는 별도 트랜잭션.
5. sqlite3는 repository 층에만. service는 plain 데이터만 주고받는다.
