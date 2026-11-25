//! 호스트 넘기기의 앱 쪽(BR-H1, BR-H2). 서버를 띄우지 않은 상태에서 일한다.
//!
//! 1. 관리자가 호스트 화면에서 PIN을 넣으면 서버가 실행 모드를 "넘기는 중"으로 바꾸고(`/api/handover`)
//!    화면이 앱을 다시 켠다.
//! 2. 넘기는 중이면 켤 때 서버 없이 DB만 열어 "데이터 넘기기 직전" 백업을 만든다([`prepare`]).
//! 3. 관리자가 저장 창에서 고른 곳에 그 백업을 넘기기 파일로 복사하면 "넘김"이 된다
//!    ([`save_handover_file`]). 고르지 않고 돌아가면 다시 호스트로 켠다([`abort_handover`]).
//! 4. 넘긴 PC는 관리자 PIN으로 넘기기를 취소하고 다시 호스트로 쓸 수 있다([`cancel_handover`]).
//!    세대 번호는 올리지 않는다(ADR 0002 구현 메모).

use std::path::{Path, PathBuf};

use baeum_domain::Role;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use time::OffsetDateTime;
use ts_rs::TS;

use crate::backup::inspect::inspect;
use crate::backup_file::name::BackupKind;
use crate::backup_file::{self, store};
use crate::clock;
use crate::db::access_logs::{self, AccessEvent};
use crate::db::{self, staff};
use crate::pin_check::check_pin;
use crate::run_mode::{self, RunMode, RunModeState};
use crate::startup;

/// 넘기는 중일 때 켜면서 만든 넘기기 파일. 앱에 맡겨 두고 명령이 꺼내 쓴다.
#[derive(Debug)]
pub struct HandingOver {
    /// 백업 폴더의 "데이터 넘기기 직전" 백업. 만들지 못했으면 화면에 보여 줄 문장.
    file: Result<PathBuf, String>,
}

/// 넘기는 중이면 켤 때 부른다. 서버를 띄우기 전이라 그사이 저장되는 것이 없다.
pub async fn prepare(data_dir: &Path, now: OffsetDateTime) -> HandingOver {
    HandingOver {
        file: make_file(data_dir, now).await,
    }
}

async fn make_file(data_dir: &Path, now: OffsetDateTime) -> Result<PathBuf, String> {
    let pool = startup::open_checked(data_dir).await.map_err(|problem| {
        tracing::error!(?problem, "넘기려는 DB를 열지 못했습니다");
        "DB를 열지 못해 넘기기 파일을 만들지 못했습니다. 넘기지 않고 돌아가 주세요".to_owned()
    })?;
    let dir = backup_file::dir(data_dir);
    let made = store::create(
        &pool,
        &dir,
        BackupKind::BeforeHandover,
        clock::local_datetime(now),
    )
    .await;
    pool.close().await;
    let name = made.map_err(|error| {
        tracing::error!(%error, "데이터 넘기기 직전 백업을 만들지 못했습니다");
        "넘기기 파일을 만들지 못했습니다. 넘기지 않고 돌아가 주세요".to_owned()
    })?;
    tracing::info!(%name, "데이터 넘기기 직전 백업을 만들었습니다");
    Ok(dir.join(name.to_string()))
}

/// 넘기기 파일 확인 화면에 보여 줄 것. 회원 이름 같은 개인정보는 담지 않는다.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HandoverFileResponse {
    pub term_names: Vec<String>,
    #[ts(type = "number")]
    pub member_count: i64,
    #[ts(type = "number")]
    pub application_count: i64,
}

/// 켤 때 만든 넘기기 파일의 미리보기.
///
/// # Errors
///
/// 넘기는 중이 아니거나 파일을 만들지 못했으면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn handover_file(
    state: State<'_, Option<HandingOver>>,
) -> Result<HandoverFileResponse, String> {
    let file = handing_over_file(&state)?;
    let preview = inspect(file).await.map_err(|error| error.to_string())?;
    Ok(HandoverFileResponse {
        term_names: preview.term_names,
        member_count: preview.member_count,
        application_count: preview.application_count,
    })
}

fn handing_over_file(state: &Option<HandingOver>) -> Result<&Path, String> {
    let Some(handing_over) = state else {
        return Err("호스트를 넘기는 중이 아닙니다".to_owned());
    };
    handing_over
        .file
        .as_deref()
        .map_err(|message| message.clone())
}

/// 저장 창에서 고른 곳에 넘기기 파일을 저장하고, 이 PC를 "넘김"으로 바꿔 다시 켠다(BR-H1).
/// 저장 창을 닫으면 `false`를 돌려주고 아무것도 바꾸지 않는다.
///
/// # Errors
///
/// 넘기는 중이 아니거나, 저장하지 못했거나, 저장한 파일을 읽을 수 없으면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn save_handover_file(
    app: AppHandle,
    state: State<'_, Option<HandingOver>>,
    run_mode: State<'_, RunModeState>,
) -> Result<bool, String> {
    let source = handing_over_file(&state)?;
    let (picked, chosen) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(file_name(OffsetDateTime::now_utc()))
        .add_filter("배움마루 데이터", &["db"])
        .save_file(move |path| {
            let _ = picked.send(path);
        });
    let Some(path) = chosen
        .await
        .map_err(|_| "저장 창이 닫혔습니다".to_owned())?
    else {
        return Ok(false);
    };
    let target = path
        .into_path()
        .map_err(|_| "저장할 곳을 알 수 없습니다".to_owned())?;
    copy_checked(source, &target).await?;
    run_mode::save(run_mode.data_dir(), RunMode::HandedOver).map_err(|error| {
        tracing::error!(%error, "실행 모드를 기억하지 못했습니다");
        "넘기기 파일은 저장했지만 이 PC를 넘김 상태로 바꾸지 못했습니다. 데이터 폴더를 쓸 수 있는지 확인해 주세요".to_owned()
    })?;
    tracing::info!("호스트를 넘겼습니다");
    app.restart();
}

/// 저장 창에 미리 채울 이름. 예: `배움마루 넘기기 2026-11-25.db`
fn file_name(now: OffsetDateTime) -> String {
    format!("배움마루 넘기기 {}.db", clock::local_date(now))
}

/// `source`를 `target`에 복사하고 다시 읽어 본다. USB가 빠지거나 가득 차도 알아채게 한다.
async fn copy_checked(source: &Path, target: &Path) -> Result<(), String> {
    let failed = |kind: std::io::ErrorKind| {
        // 경로에는 직원 이름 같은 폴더 이름이 있을 수 있어 로그에 남기지 않는다
        tracing::warn!(?kind, "넘기기 파일을 저장하지 못했습니다");
        "넘기기 파일을 저장하지 못했습니다. 다른 곳을 골라 주세요".to_owned()
    };
    tokio::fs::copy(source, target)
        .await
        .map_err(|error| failed(error.kind()))?;
    inspect(target).await.map_err(|error| {
        tracing::warn!(%error, "저장한 넘기기 파일을 읽지 못했습니다");
        "저장한 넘기기 파일을 읽지 못했습니다. 다른 곳을 골라 주세요".to_owned()
    })?;
    Ok(())
}

/// 넘기지 않고 다시 호스트로 켠다. 넘기는 중일 때만. 아직 아무 데도 넘기지 않아 PIN을 묻지 않는다.
///
/// # Errors
///
/// 넘기는 중이 아니거나 기억하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn abort_handover(
    app: AppHandle,
    run_mode: State<'_, RunModeState>,
) -> Result<(), String> {
    if run_mode.mode() != Some(RunMode::HandingOver) {
        return Err("호스트를 넘기는 중이 아닙니다".to_owned());
    }
    back_to_host(run_mode.data_dir(), None, OffsetDateTime::now_utc()).await?;
    tracing::info!("넘기지 않고 호스트로 돌아갑니다");
    app.restart();
}

/// 넘김 화면의 관리자 목록 한 줄.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HandedOverAdmin {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
}

/// 넘기기를 취소할 수 있는 관리자들. 넘긴 PC에서만.
///
/// # Errors
///
/// 넘긴 PC가 아니거나 DB를 열지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn handed_over_admins(
    run_mode: State<'_, RunModeState>,
) -> Result<Vec<HandedOverAdmin>, String> {
    let pool = open_handed_over(&run_mode).await?;
    let listed = staff::list_for_login(&pool, clock::local_today()).await;
    pool.close().await;
    let choices = listed.map_err(|error| {
        tracing::error!(%error, "관리자 목록을 읽지 못했습니다");
        "관리자 목록을 읽지 못했습니다".to_owned()
    })?;
    Ok(choices
        .into_iter()
        .filter(|choice| choice.role == Role::Admin)
        .map(|choice| HandedOverAdmin {
            id: choice.id,
            name: choice.name,
        })
        .collect())
}

/// 관리자 PIN을 확인하고 넘기기를 취소해 다시 호스트로 켠다(BR-H2). 세대 번호는 그대로다.
///
/// 틀린 PIN은 로그인과 같이 BR-K7의 횟수에 들어간다.
///
/// # Errors
///
/// 넘긴 PC가 아니거나, 관리자가 아니거나, PIN이 틀렸거나, 기억하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn cancel_handover(
    app: AppHandle,
    run_mode: State<'_, RunModeState>,
    staff_id: i64,
    pin: String,
) -> Result<(), String> {
    let pool = open_handed_over(&run_mode).await?;
    let now = OffsetDateTime::now_utc();
    let checked = check_admin_pin(&pool, staff_id, pin, now).await;
    pool.close().await;
    let staff_id = checked?;
    back_to_host(run_mode.data_dir(), Some(staff_id), now).await?;
    tracing::info!(staff_id, "넘기기를 취소하고 호스트로 돌아갑니다");
    app.restart();
}

/// 넘긴 PC의 DB를 연다. 서버는 띄우지 않는다.
async fn open_handed_over(run_mode: &RunModeState) -> Result<SqlitePool, String> {
    if run_mode.mode() != Some(RunMode::HandedOver) {
        return Err("호스트를 넘긴 PC가 아닙니다".to_owned());
    }
    startup::open_checked(run_mode.data_dir())
        .await
        .map_err(|problem| {
            tracing::error!(?problem, "넘긴 PC의 DB를 열지 못했습니다");
            "이 PC의 DB를 열지 못했습니다".to_owned()
        })
}

/// 관리자 PIN이 맞으면 그 관리자 id.
async fn check_admin_pin(
    pool: &SqlitePool,
    staff_id: i64,
    pin: String,
    now: OffsetDateTime,
) -> Result<i64, String> {
    let account = staff::get_login_account(pool, staff_id)
        .await
        .map_err(|error| {
            tracing::error!(%error, "관리자 계정을 읽지 못했습니다");
            "관리자 계정을 읽지 못했습니다".to_owned()
        })?
        .ok_or_else(|| "관리자를 찾지 못했습니다".to_owned())?;
    if account.role != Role::Admin {
        return Err("관리자만 넘기기를 취소할 수 있습니다".to_owned());
    }
    // 넘긴 PC 자신에서 하므로 기기는 늘 호스트 자신이다
    let account = check_pin(pool, account, pin, None, AccessEvent::LoginFailed, now)
        .await
        .map_err(|refusal| refusal.to_string())?;
    Ok(account.id)
}

/// 넘기기 취소를 접속 기록에 남기고 실행 모드를 호스트로 바꾼다.
///
/// 넘기지 않고 돌아갈 때(`staff_id`가 `None`)는 기록하지 못해도 돌아간다. 아무 데도 넘기지 않았다.
async fn back_to_host(
    data_dir: &Path,
    staff_id: Option<i64>,
    now: OffsetDateTime,
) -> Result<(), String> {
    match db::open(data_dir).await {
        Ok(pool) => {
            let recorded =
                access_logs::record(&pool, staff_id, AccessEvent::HandoverCanceled, now).await;
            pool.close().await;
            if let Err(error) = recorded {
                tracing::error!(%error, "넘기기 취소를 접속 기록에 남기지 못했습니다");
                if staff_id.is_some() {
                    return Err("넘기기 취소를 접속 기록에 남기지 못했습니다".to_owned());
                }
            }
        }
        Err(error) => tracing::error!(%error, "넘기기 취소를 남길 DB를 열지 못했습니다"),
    }
    run_mode::save(data_dir, RunMode::Host).map_err(|error| {
        tracing::error!(%error, "실행 모드를 기억하지 못했습니다");
        "실행 모드를 기억하지 못했습니다. 데이터 폴더를 쓸 수 있는지 확인해 주세요".to_owned()
    })
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::db::access_logs::list_for_test;
    use crate::db::data_bundle;
    use crate::db::test_support::insert_staff;

    const NOW: OffsetDateTime = datetime!(2026-11-25 01:00 UTC);

    #[tokio::test]
    async fn br_h1_prepare_makes_before_handover_backup() {
        let dir = tempfile::tempdir().unwrap();
        db::open(dir.path()).await.unwrap().close().await;

        let handing_over = prepare(dir.path(), NOW).await;

        let file = handing_over.file.unwrap();
        assert!(file.starts_with(backup_file::dir(dir.path())));
        assert!(inspect(&file).await.is_ok());
        let listed = store::list(&backup_file::dir(dir.path())).await.unwrap();
        assert_eq!(listed[0].name.kind, BackupKind::BeforeHandover);
    }

    #[tokio::test]
    async fn saved_file_is_checked() {
        let dir = tempfile::tempdir().unwrap();
        db::open(dir.path()).await.unwrap().close().await;
        let source = prepare(dir.path(), NOW).await.file.unwrap();
        let target = dir.path().join("넘기기.db");

        copy_checked(&source, &target).await.unwrap();

        assert!(inspect(&target).await.is_ok());
        assert!(
            copy_checked(&source, &dir.path().join("없는 폴더/넘기기.db"))
                .await
                .is_err()
        );
    }

    /// 넘긴 PC의 DB에 PIN 123456인 관리자를 만든다.
    async fn handed_over_with_admin(data_dir: &Path) -> i64 {
        let pool = db::open(data_dir).await.unwrap();
        let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
        sqlx::query("UPDATE staff SET pin_hash = ?")
            .bind(crate::secret::hash("123456").unwrap())
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
        admin
    }

    #[tokio::test]
    async fn br_h2_admin_pin_cancels_without_raising_generation() {
        let dir = tempfile::tempdir().unwrap();
        let admin = handed_over_with_admin(dir.path()).await;
        let pool = db::open(dir.path()).await.unwrap();

        let checked = check_admin_pin(&pool, admin, "123456".to_owned(), NOW).await;
        pool.close().await;
        back_to_host(dir.path(), checked.ok(), NOW).await.unwrap();

        assert_eq!(run_mode::resolve(dir.path()), Some(RunMode::Host));
        let pool = db::open(dir.path()).await.unwrap();
        assert_eq!(data_bundle::read(&pool).await.unwrap().generation, 1);
        assert_eq!(
            list_for_test(&pool).await.last().unwrap(),
            &(Some(admin), "handover_canceled".to_owned())
        );
    }

    #[tokio::test]
    async fn br_h2_wrong_pin_or_non_admin_does_not_cancel() {
        let dir = tempfile::tempdir().unwrap();
        let admin = handed_over_with_admin(dir.path()).await;
        let pool = db::open(dir.path()).await.unwrap();
        let worker = insert_staff(&pool, "직원1", "staff", None).await.unwrap();

        let wrong = check_admin_pin(&pool, admin, "000000".to_owned(), NOW).await;
        let not_admin = check_admin_pin(&pool, worker, "123456".to_owned(), NOW).await;

        assert_eq!(
            wrong,
            Err("PIN이 맞지 않습니다. 5번 연속 틀리면 5분 동안 잠깁니다".to_owned())
        );
        assert!(not_admin.is_err());
        assert_eq!(
            list_for_test(&pool).await.last().unwrap(),
            &(Some(admin), "login_failed".to_owned())
        );
    }

    #[test]
    fn suggested_file_name_has_local_date() {
        assert!(file_name(NOW).starts_with("배움마루 넘기기 2026-11-2"));
        assert!(file_name(NOW).ends_with(".db"));
    }
}
