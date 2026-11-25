//! 넘겨받기(BR-H3): 옛 호스트가 만든 넘기기 파일로 이 PC를 새 호스트로 시작한다.
//!
//! 이 PC는 아직 호스트가 아니라 서버도 DB도 열려 있지 않다. 그래서 확인은 파일 안의 관리자 이름과
//! PIN으로 한다. 직원의 실수를 막는 장치이고, 파일을 가진 사람까지 막지는 않는다.
//!
//! 1. 파일을 살펴본다(백업 복원과 같은 검사).
//! 2. 파일 안 관리자의 PIN을 확인한다.
//! 3. 이 PC에 예전 DB가 남아 있으면 "넘겨받기 직전" 백업으로 남긴다. 지우지 않는다(ADR 0021).
//! 4. 파일을 이 PC의 DB로 들이고, 세대 번호를 하나 올리고(BR-H4), 접속 기록에 남긴다.
//! 5. 실행 모드를 호스트로 바꾼다. 다시 켜기는 `commands::take_over`가 한다.

use std::path::Path;

use serde::Serialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, SqliteConnection};
use time::OffsetDateTime;
use ts_rs::TS;

use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::features::backups::backup::inspect::inspect;
use crate::features::backups::backup::{self, restore};
use crate::infra::backup_file::name::BackupKind;
use crate::infra::backup_file::{self, store};
use crate::infra::clock;
use crate::infra::secret;
use crate::infra::startup;
use crate::infra::{data_bundle, db};
use crate::run_mode::{self, RunMode, RunModeState};

/// 예전 DB를 백업으로 남기지 못했을 때 옮겨 두는 이름. 하나만 둔다.
const KEPT_DB: &str = "baeum-maru.before-take-over.db";

/// 넘겨받기 확인 화면에 보여 줄 것. 관리자 이름 말고는 개인정보를 담지 않는다.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TakeOverPreviewResponse {
    pub term_names: Vec<String>,
    #[ts(type = "number")]
    pub member_count: i64,
    #[ts(type = "number")]
    pub application_count: i64,
    /// 확인에 쓸 수 있는 파일 안의 관리자들.
    pub admins: Vec<TakeOverAdmin>,
}

/// 파일 안의 관리자 한 명.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TakeOverAdmin {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
}

/// 넘겨받지 못했다. 이 PC는 그대로다.
#[derive(Debug, thiserror::Error)]
pub enum TakeOverError {
    #[error(transparent)]
    Inspect(#[from] backup::inspect::InspectError),
    #[error("이 PC는 이미 호스트이거나 호스트를 넘기는 중입니다")]
    NotAllowedHere,
    #[error("파일에서 그 관리자를 찾지 못했습니다")]
    AdminNotFound,
    #[error("PIN이 맞지 않습니다")]
    WrongPin,
    #[error("이 PC에 남은 예전 데이터를 보관하지 못해 넘겨받지 않았습니다")]
    KeepLeftover(#[source] std::io::Error),
    #[error("파일을 이 PC로 옮기지 못했습니다")]
    Install(#[source] std::io::Error),
    #[error("넘겨받은 DB를 열지 못했습니다")]
    Open(#[source] db::DbError),
    #[error("넘겨받은 DB에 세대 번호를 쓰지 못했습니다")]
    Record(#[source] sqlx::Error),
}

/// 넘겨받기는 아직 호스트가 아닌 PC에서만 한다. 처음 켠 PC와 접속 모드 PC.
fn can_take_over(mode: Option<RunMode>) -> bool {
    matches!(mode, None | Some(RunMode::Remote))
}

/// 고른 넘기기 파일을 살펴보고 미리보기와 관리자 목록을 돌려준다.
///
/// # Errors
///
/// 넘겨받을 수 없는 PC거나 파일이면 화면에 보여 줄 문장.
pub async fn preview_for(
    run_mode: &RunModeState,
    file: &Path,
) -> Result<TakeOverPreviewResponse, String> {
    if !can_take_over(run_mode.mode()) {
        return Err(TakeOverError::NotAllowedHere.to_string());
    }
    preview(file).await.map_err(|error| error.to_string())
}

async fn preview(file: &Path) -> Result<TakeOverPreviewResponse, TakeOverError> {
    let found = inspect(file).await?;
    let mut conn = open_read_only(file).await?;
    let admins = admins_in(&mut conn).await;
    let _ = conn.close().await;
    let admins = admins.map_err(|error| read_failed(&error))?;
    Ok(TakeOverPreviewResponse {
        term_names: found.term_names,
        member_count: found.member_count,
        application_count: found.application_count,
        admins,
    })
}

/// 파일 안의 쓸 수 있는 관리자들. 이름 순서.
async fn admins_in(conn: &mut SqliteConnection) -> sqlx::Result<Vec<TakeOverAdmin>> {
    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, name FROM staff WHERE role = 'admin' AND disabled = 0 ORDER BY name, id",
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name)| TakeOverAdmin { id, name })
        .collect())
}

/// 파일을 고치지 않는 연결을 연다. 잠금이나 보조 파일도 만들지 않는다(backup::inspect와 같다).
async fn open_read_only(file: &Path) -> Result<SqliteConnection, TakeOverError> {
    SqliteConnectOptions::new()
        .filename(file)
        .read_only(true)
        .immutable(true)
        .connect()
        .await
        .map_err(|_| not_ours())
}

/// 파일을 읽지 못했다. 살펴보기는 통과했으니 배움마루 파일이 아니라고 본다.
fn not_ours() -> TakeOverError {
    TakeOverError::Inspect(backup::inspect::InspectError::NotBaeumMaru)
}

/// 읽은 결과를 넘기기 파일 오류로.
fn read_failed(error: &sqlx::Error) -> TakeOverError {
    tracing::info!(%error, "넘기기 파일을 읽지 못했습니다");
    not_ours()
}

/// 파일 안 관리자의 PIN 해시. 쓸 수 있는 관리자가 아니면 `None`.
async fn admin_pin_hash(
    conn: &mut SqliteConnection,
    staff_id: i64,
) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar(
        "SELECT pin_hash FROM staff WHERE id = ? AND role = 'admin' AND disabled = 0",
    )
    .bind(staff_id)
    .fetch_optional(conn)
    .await
}

/// 파일 안 관리자의 PIN으로 확인하고 이 PC를 새 호스트로 기억한다(BR-H3). 다시 켜기는 부르는 쪽이 한다.
///
/// # Errors
///
/// 넘겨받을 수 없는 PC나 파일이거나, PIN이 틀렸거나, 옮기지 못하면 화면에 보여 줄 문장.
/// 이 PC는 그대로다.
pub async fn take_over(
    run_mode: &RunModeState,
    file: &Path,
    staff_id: i64,
    pin: String,
) -> Result<(), String> {
    if !can_take_over(run_mode.mode()) {
        return Err(TakeOverError::NotAllowedHere.to_string());
    }
    let generation = take_over_file(
        run_mode.data_dir(),
        file,
        staff_id,
        pin,
        OffsetDateTime::now_utc(),
    )
    .await
    .map_err(|error| {
        match &error {
            TakeOverError::WrongPin | TakeOverError::AdminNotFound => {}
            // 경로는 남기지 않는다
            other => tracing::error!(error = %other, "넘겨받지 못했습니다"),
        }
        error.to_string()
    })?;
    run_mode::save(run_mode.data_dir(), RunMode::Host).map_err(|error| {
        tracing::error!(%error, "실행 모드를 기억하지 못했습니다");
        "데이터는 옮겼지만 실행 모드를 바꾸지 못했습니다. 데이터 폴더를 쓸 수 있는지 확인해 주세요"
            .to_owned()
    })?;
    tracing::info!(staff_id, generation, "넘겨받아 새 호스트로 시작합니다");
    Ok(())
}

/// 넘겨받기의 본체. 바뀐 세대 번호를 돌려준다. 실행 모드는 부르는 쪽이 바꾼다.
async fn take_over_file(
    data_dir: &Path,
    source: &Path,
    staff_id: i64,
    pin: String,
    now: OffsetDateTime,
) -> Result<u32, TakeOverError> {
    inspect(source).await?;
    let mut conn = open_read_only(source).await?;
    let hash = admin_pin_hash(&mut conn, staff_id).await;
    let _ = conn.close().await;
    let hash = hash
        .map_err(|error| read_failed(&error))?
        .ok_or(TakeOverError::AdminNotFound)?;
    // 해시 확인은 일부러 느려서 따로 돌린다
    let matches = tokio::task::spawn_blocking(move || secret::verify(&pin, &hash))
        .await
        .unwrap_or(false);
    if !matches {
        return Err(TakeOverError::WrongPin);
    }

    keep_leftover(data_dir, now)
        .await
        .map_err(TakeOverError::KeepLeftover)?;
    tokio::fs::create_dir_all(data_dir)
        .await
        .map_err(TakeOverError::Install)?;
    restore::install_copy(data_dir, source)
        .await
        .map_err(TakeOverError::Install)?;

    let pool = db::open(data_dir).await.map_err(TakeOverError::Open)?;
    let recorded = async {
        let generation = data_bundle::raise_generation(&pool).await?;
        access_logs::record(&pool, Some(staff_id), AccessEvent::TakeOver, now).await?;
        Ok::<_, sqlx::Error>(generation)
    }
    .await;
    pool.close().await;
    recorded.map_err(TakeOverError::Record)
}

/// 이 PC에 예전 DB가 있으면 남긴다. 열리면 백업 폴더에 "넘겨받기 직전" 백업을 만들고,
/// 열리지 않으면 데이터 폴더에 [`KEPT_DB`]로 옮겨 둔다. 없으면 할 일이 없다.
async fn keep_leftover(data_dir: &Path, now: OffsetDateTime) -> std::io::Result<()> {
    let db_file = db::file(data_dir);
    if !tokio::fs::try_exists(&db_file).await? {
        return Ok(());
    }
    if let Ok(pool) = startup::open_checked(data_dir).await {
        let made = store::create(
            &pool,
            &backup_file::dir(data_dir),
            BackupKind::BeforeTakeOver,
            clock::local_datetime(now),
        )
        .await;
        pool.close().await;
        match made {
            Ok(name) => {
                tracing::info!(%name, "이 PC의 예전 데이터를 백업으로 남겼습니다");
                return Ok(());
            }
            Err(error) => tracing::warn!(%error, "예전 데이터를 백업하지 못해 옮겨 둡니다"),
        }
    }
    // 깨졌거나 더 새 버전의 DB다. 그대로 옮겨 둔다. 지난번에 옮겨 둔 것은 덮는다
    tokio::fs::copy(&db_file, data_dir.join(KEPT_DB)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use std::path::PathBuf;

    use super::*;
    use crate::db::test_support::{insert_member, insert_staff};
    use crate::features::access_logs::repository::list_for_test;

    const NOW: OffsetDateTime = datetime!(2026-11-25 01:00 UTC);

    /// 옛 호스트의 넘기기 파일을 만든다. PIN 123456인 관리자와 회원 한 명. 관리자 id를 함께 준다.
    async fn handover_file(dir: &Path) -> (PathBuf, i64) {
        let old_host = dir.join("옛 호스트");
        let pool = db::open(&old_host).await.unwrap();
        let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
        insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        sqlx::query("UPDATE staff SET pin_hash = ?")
            .bind(secret::hash("123456").unwrap())
            .execute(&pool)
            .await
            .unwrap();
        insert_member(&pool, "00001").await.unwrap();
        let name = store::create(
            &pool,
            &backup_file::dir(&old_host),
            BackupKind::BeforeHandover,
            datetime!(2026-11-25 09:00),
        )
        .await
        .unwrap();
        pool.close().await;
        (backup_file::dir(&old_host).join(name.to_string()), admin)
    }

    #[tokio::test]
    async fn preview_lists_only_admins() {
        let dir = tempfile::tempdir().unwrap();
        let (file, admin) = handover_file(dir.path()).await;

        let preview = preview(&file).await.unwrap();

        assert_eq!(preview.member_count, 1);
        assert_eq!(
            preview.admins,
            [TakeOverAdmin {
                id: admin,
                name: "관리자".to_owned()
            }]
        );
    }

    #[tokio::test]
    async fn br_h3_take_over_raises_generation_and_keeps_bundle_id() {
        let dir = tempfile::tempdir().unwrap();
        let (file, admin) = handover_file(dir.path()).await;
        let source = data_bundle::read_file(&file).await.unwrap();
        let new_pc = dir.path().join("새 PC");

        let generation = take_over_file(&new_pc, &file, admin, "123456".to_owned(), NOW)
            .await
            .unwrap();

        assert_eq!(generation, 2);
        let pool = db::open(&new_pc).await.unwrap();
        let now = data_bundle::read(&pool).await.unwrap();
        assert_eq!(now.bundle_id, source.bundle_id);
        assert_eq!(now.generation, 2);
        assert_eq!(
            list_for_test(&pool).await.last().unwrap(),
            &(Some(admin), "take_over".to_owned())
        );
        // 넘기기 파일은 그대로다
        assert_eq!(data_bundle::read_file(&file).await.unwrap(), source);
    }

    #[tokio::test]
    async fn br_h3_wrong_pin_or_non_admin_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let (file, admin) = handover_file(dir.path()).await;
        let new_pc = dir.path().join("새 PC");

        let wrong = take_over_file(&new_pc, &file, admin, "000000".to_owned(), NOW).await;
        let worker = take_over_file(&new_pc, &file, admin + 1, "123456".to_owned(), NOW).await;

        assert!(matches!(wrong, Err(TakeOverError::WrongPin)));
        assert!(matches!(worker, Err(TakeOverError::AdminNotFound)));
        assert!(!db::file(&new_pc).exists());
    }

    #[tokio::test]
    async fn br_h3_leftover_data_is_kept_as_backup() {
        let dir = tempfile::tempdir().unwrap();
        let (file, admin) = handover_file(dir.path()).await;
        let new_pc = dir.path().join("새 PC");
        let leftover = db::open(&new_pc).await.unwrap();
        insert_member(&leftover, "00001").await.unwrap();
        insert_member(&leftover, "00002").await.unwrap();
        leftover.close().await;

        take_over_file(&new_pc, &file, admin, "123456".to_owned(), NOW)
            .await
            .unwrap();

        let backups = store::list(&backup_file::dir(&new_pc)).await.unwrap();
        assert_eq!(backups[0].name.kind, BackupKind::BeforeTakeOver);
        let kept = inspect(&backup_file::dir(&new_pc).join(backups[0].name.to_string()))
            .await
            .unwrap();
        assert_eq!(kept.member_count, 2);
    }

    #[tokio::test]
    async fn broken_leftover_is_moved_aside() {
        let dir = tempfile::tempdir().unwrap();
        let (file, admin) = handover_file(dir.path()).await;
        let new_pc = dir.path().join("새 PC");
        tokio::fs::create_dir_all(&new_pc).await.unwrap();
        tokio::fs::write(db::file(&new_pc), "망가진 파일 ".repeat(1000))
            .await
            .unwrap();

        take_over_file(&new_pc, &file, admin, "123456".to_owned(), NOW)
            .await
            .unwrap();

        assert!(new_pc.join(KEPT_DB).exists());
        let pool = db::open(&new_pc).await.unwrap();
        assert_eq!(data_bundle::read(&pool).await.unwrap().generation, 2);
    }

    #[test]
    fn only_pc_that_is_not_yet_host_takes_over() {
        assert!(can_take_over(None));
        assert!(can_take_over(Some(RunMode::Remote)));
        assert!(!can_take_over(Some(RunMode::Host)));
        assert!(!can_take_over(Some(RunMode::HandingOver)));
        assert!(!can_take_over(Some(RunMode::HandedOver)));
    }
}
