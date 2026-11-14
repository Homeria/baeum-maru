//! 버린 코드: SQLite 동시 저장 실험 (3단계 기술 실험 01)
//!
//! 직원 여러 명이 동시에 접수하는 상황을 흉내 내서,
//! 1. "DB가 잠겨 있음" 오류가 생기는지
//! 2. "한 회원 최대 4개"(BR-A3) 규칙이 깨지는지
//! 3. 신청 한 건 저장에 얼마나 걸리는지
//! 를 확인한다. 회원·강좌는 모두 번호뿐인 가짜 데이터다.

use std::time::{Duration, Instant};

use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use sqlx::SqliteConnection;

const DB_FILE: &str = "spike.db";
const STAFF: i64 = 8; // 동시에 접수하는 직원 수
const ATTEMPTS_PER_STAFF: i64 = 500; // 직원 한 명이 넣는 신청 수
const MEMBERS: i64 = 300;
const COURSES: i64 = 30;
const MAX_APPLICATIONS: i64 = 4; // BR-A3 최대 신청 개수

/// 신청 한 건을 넣어 본 결과.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Outcome {
    Inserted,     // 저장됨
    LimitReached, // 최대 개수를 넘어서 거절
    Duplicate,    // 같은 강좌에 이미 신청 있음 (BR-A4)
}

/// 확인과 저장을 어떻게 묶을지.
#[derive(Debug, Clone, Copy)]
enum Mode {
    /// 올바른 방식: `BEGIN IMMEDIATE` 트랜잭션 하나 안에서 확인하고 저장한다.
    Safe,
    /// 잘못된 방식: 확인과 저장을 따로 한다. 그 사이에 다른 직원이 끼어들 수 있다.
    Unsafe,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 인자 없이 실행하면 두 방식을 모두 돌린다.
    let arg = std::env::args().nth(1);
    let modes = match arg.as_deref() {
        Some("safe") => vec![Mode::Safe],
        Some("unsafe") => vec![Mode::Unsafe],
        _ => vec![Mode::Safe, Mode::Unsafe],
    };

    for mode in modes {
        run(mode).await?;
    }
    Ok(())
}

async fn run(mode: Mode) -> Result<(), sqlx::Error> {
    let pool = fresh_database().await?;

    let started = Instant::now();
    let mut handles = Vec::new();
    for staff in 0..STAFF {
        // pool은 내부적으로 공유되는 핸들이라 clone해도 DB 연결이 복제되지 않는다.
        let pool = pool.clone();
        handles.push(tokio::spawn(async move { staff_work(&pool, staff, mode).await }));
    }

    // 모든 직원의 결과를 모은다.
    let mut results = Vec::new();
    for handle in handles {
        results.extend(handle.await.expect("직원 작업이 패닉으로 끝남"));
    }
    let elapsed = started.elapsed();

    report(mode, &pool, &results, elapsed).await?;
    pool.close().await;
    Ok(())
}

/// 실험마다 빈 DB를 새로 만든다.
async fn fresh_database() -> Result<SqlitePool, sqlx::Error> {
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{DB_FILE}{suffix}"));
    }

    let options = SqliteConnectOptions::new()
        .filename(DB_FILE)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal) // 읽기와 쓰기가 서로 덜 막히는 모드
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5)); // 잠겨 있으면 바로 실패하지 말고 최대 5초 기다림

    let pool = SqlitePoolOptions::new()
        .max_connections(STAFF as u32)
        .connect_with(options)
        .await?;

    sqlx::query(
        "CREATE TABLE members (id INTEGER PRIMARY KEY);
         CREATE TABLE courses (id INTEGER PRIMARY KEY);
         CREATE TABLE applications (
             id        INTEGER PRIMARY KEY,
             member_id INTEGER NOT NULL REFERENCES members(id),
             course_id INTEGER NOT NULL REFERENCES courses(id),
             UNIQUE (member_id, course_id)
         );
         CREATE INDEX applications_member ON applications(member_id);",
    )
    .execute(&pool)
    .await?;

    for id in 0..MEMBERS {
        sqlx::query("INSERT INTO members (id) VALUES (?)").bind(id).execute(&pool).await?;
    }
    for id in 0..COURSES {
        sqlx::query("INSERT INTO courses (id) VALUES (?)").bind(id).execute(&pool).await?;
    }
    Ok(pool)
}

/// 직원 한 명의 접수 작업.
///
/// 모든 직원이 같은 순서로 회원을 돌아가며 접수한다. 그래서 같은 회원을
/// 여러 직원이 거의 동시에 접수하는 상황이 계속 생긴다 (일부러 만든 최악의 경우).
async fn staff_work(
    pool: &SqlitePool,
    staff: i64,
    mode: Mode,
) -> Vec<(Result<Outcome, sqlx::Error>, Duration)> {
    let mut results = Vec::with_capacity(ATTEMPTS_PER_STAFF as usize);
    for k in 0..ATTEMPTS_PER_STAFF {
        let member = k % MEMBERS;
        // 직원마다 다른 강좌를 고르게 해서 "최대 개수" 경쟁이 생기게 한다.
        let course = (k * 7 + staff * 13) % COURSES;

        let t = Instant::now();
        let result = match mode {
            Mode::Safe => apply_safe(pool, member, course).await,
            Mode::Unsafe => apply_unsafe(pool, member, course).await,
        };
        results.push((result, t.elapsed()));
    }
    results
}

/// 올바른 방식: 트랜잭션 하나 안에서 확인하고 저장한다.
///
/// `BEGIN IMMEDIATE`는 시작하자마자 쓰기 잠금을 잡는다. 그래서 다른 직원의
/// 접수는 이 트랜잭션이 끝날 때까지 기다리고, 그동안 개수가 바뀌지 않는다.
async fn apply_safe(pool: &SqlitePool, member: i64, course: i64) -> Result<Outcome, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;

    let result = check_and_insert(&mut conn, member, course).await;

    // 저장했으면 확정(COMMIT), 거절이나 오류면 되돌린다(ROLLBACK).
    let finish = if matches!(result, Ok(Outcome::Inserted)) { "COMMIT" } else { "ROLLBACK" };
    sqlx::query(finish).execute(&mut *conn).await?;
    result
}

async fn check_and_insert(
    conn: &mut SqliteConnection,
    member: i64,
    course: i64,
) -> Result<Outcome, sqlx::Error> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM applications WHERE member_id = ? AND course_id = ?",
    )
    .bind(member)
    .bind(course)
    .fetch_one(&mut *conn)
    .await?;
    if exists > 0 {
        return Ok(Outcome::Duplicate);
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM applications WHERE member_id = ?")
        .bind(member)
        .fetch_one(&mut *conn)
        .await?;
    if count >= MAX_APPLICATIONS {
        return Ok(Outcome::LimitReached);
    }

    sqlx::query("INSERT INTO applications (member_id, course_id) VALUES (?, ?)")
        .bind(member)
        .bind(course)
        .execute(&mut *conn)
        .await?;
    Ok(Outcome::Inserted)
}

/// 잘못된 방식: 개수를 확인한 뒤 따로 저장한다.
///
/// 확인과 저장 사이에 다른 직원이 같은 회원을 저장하면, 둘 다 "아직 3개"를
/// 보고 저장해서 5개가 될 수 있다.
async fn apply_unsafe(pool: &SqlitePool, member: i64, course: i64) -> Result<Outcome, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM applications WHERE member_id = ?")
        .bind(member)
        .fetch_one(pool)
        .await?;
    if count >= MAX_APPLICATIONS {
        return Ok(Outcome::LimitReached);
    }

    // 실제 서버에서도 확인과 저장 사이에는 다른 일이 끼어들 수 있다. 그 틈을 흉내 낸다.
    tokio::task::yield_now().await;

    let inserted = sqlx::query("INSERT INTO applications (member_id, course_id) VALUES (?, ?)")
        .bind(member)
        .bind(course)
        .execute(pool)
        .await;
    match inserted {
        Ok(_) => Ok(Outcome::Inserted),
        // 같은 강좌 중복은 DB의 UNIQUE 제약이 막아준다.
        Err(e) if e.as_database_error().is_some_and(|d| d.is_unique_violation()) => {
            Ok(Outcome::Duplicate)
        }
        Err(e) => Err(e),
    }
}

async fn report(
    mode: Mode,
    pool: &SqlitePool,
    results: &[(Result<Outcome, sqlx::Error>, Duration)],
    elapsed: Duration,
) -> Result<(), sqlx::Error> {
    let count = |want: Outcome| results.iter().filter(|(r, _)| matches!(r, Ok(o) if *o == want)).count();
    let errors: Vec<&sqlx::Error> = results.iter().filter_map(|(r, _)| r.as_ref().err()).collect();
    let locked = errors
        .iter()
        .filter(|e| {
            let msg = e.to_string();
            msg.contains("locked") || msg.contains("busy")
        })
        .count();

    // BR-A3 위반: 신청이 최대 개수보다 많은 회원 수
    let violations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM (
             SELECT member_id FROM applications GROUP BY member_id HAVING COUNT(*) > ?
         )",
    )
    .bind(MAX_APPLICATIONS)
    .fetch_one(pool)
    .await?;
    let worst: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(c), 0) FROM (SELECT COUNT(*) AS c FROM applications GROUP BY member_id)",
    )
    .fetch_one(pool)
    .await?;

    let mut latencies: Vec<Duration> = results.iter().map(|(_, d)| *d).collect();
    latencies.sort();
    let pct = |p: f64| latencies[((latencies.len() as f64 - 1.0) * p).round() as usize];

    println!("== {mode:?} ==");
    println!("시도             {}건 (직원 {STAFF}명 x {ATTEMPTS_PER_STAFF}건)", results.len());
    println!("저장됨           {}", count(Outcome::Inserted));
    println!("최대 개수 거절   {}", count(Outcome::LimitReached));
    println!("중복 거절        {}", count(Outcome::Duplicate));
    println!("오류             {} (그중 잠금 오류 {locked})", errors.len());
    if let Some(first) = errors.first() {
        println!("  첫 오류: {first}");
    }
    println!("BR-A3 위반 회원  {violations}명 (가장 많은 회원의 신청 수: {worst})");
    println!(
        "저장 시간        중간 {:.2}ms / 느린 쪽 1% {:.2}ms / 최대 {:.2}ms",
        ms(pct(0.50)),
        ms(pct(0.99)),
        ms(*latencies.last().unwrap_or(&Duration::ZERO))
    );
    println!("전체 걸린 시간   {:.2}s", elapsed.as_secs_f64());
    println!();
    Ok(())
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}
