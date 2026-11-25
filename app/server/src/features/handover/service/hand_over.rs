//! 호스트 넘기기의 앱 쪽(BR-H1, BR-H2). 서버를 띄우지 않은 상태에서 일한다. Tauri를 모르고,
//! 저장 창과 다시 켜기는 `commands::handover`가 한다.
//!
//! 1. 관리자가 호스트 화면에서 PIN을 넣으면 서버가 실행 모드를 "넘기는 중"으로 바꾸고(`/api/handover`)
//!    화면이 앱을 다시 켠다.
//! 2. 넘기는 중이면 켤 때 서버 없이 DB만 열어 "데이터 넘기기 직전" 백업을 만든다([`prepare`]).
//! 3. 관리자가 저장 창에서 고른 곳에 그 백업을 넘기기 파일로 복사하면 "넘김"이 된다
//!    ([`save_file`]). 고르지 않고 돌아가면 다시 호스트로 켠다([`abort`]).
//! 4. 넘긴 PC는 관리자 PIN으로 넘기기를 취소하고 다시 호스트로 쓸 수 있다([`cancel`]).
//!    세대 번호는 올리지 않는다(ADR 0002 구현 메모).

use std::path::{Path, PathBuf};

use baeum_domain::Role;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::run_mode::{self, RunMode, RunModeState};
use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::features::auth::repository as auth;
use crate::features::auth::service::check_pin;
use crate::features::backups::service::inspect::inspect;
/// 화면이 받는 모양. launcher가 이 모듈 이름으로 찾는다.
pub use crate::features::handover::dto::{HandedOverAdmin, HandoverFileResponse};
use crate::infra::backup_file::name::BackupKind;
use crate::infra::backup_file::{self, store};
use crate::infra::clock;
use crate::infra::db;
use crate::infra::startup;

/// 넘기는 중일 때 켜면서 만든 넘기기 파일. 앱에 맡겨 두고 명령이 꺼내 쓴다.
#[derive(Debug)]
pub struct HandingOver {
    /// 백업 폴더의 "데이터 넘기기 직전" 백업. 만들지 못했으면 화면에 보여 줄 문장.
    pub(in crate::features::handover) file: Result<PathBuf, String>,
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

/// 켤 때 만든 넘기기 파일의 미리보기.
///
/// # Errors
///
/// 넘기는 중이 아니거나 파일을 만들지 못했으면 화면에 보여 줄 문장.
pub async fn preview(state: &Option<HandingOver>) -> Result<HandoverFileResponse, String> {
    let file = handing_over_file(state)?;
    let preview = inspect(file).await.map_err(|error| error.to_string())?;
    Ok(HandoverFileResponse {
        term_names: preview.term_names,
        member_count: preview.member_count,
        application_count: preview.application_count,
    })
}

/// 켤 때 만든 넘기기 파일. 넘기는 중이 아니거나 만들지 못했으면 화면에 보여 줄 문장.
///
/// # Errors
///
/// 넘기는 중이 아니거나 파일을 만들지 못했으면 그 문장.
pub fn handing_over_file(state: &Option<HandingOver>) -> Result<&Path, String> {
    let Some(handing_over) = state else {
        return Err("호스트를 넘기는 중이 아닙니다".to_owned());
    };
    handing_over
        .file
        .as_deref()
        .map_err(|message| message.clone())
}

/// 넘기기 파일을 `target`에 저장하고 이 PC를 "넘김"으로 기억한다(BR-H1). 다시 켜기는 부르는 쪽이 한다.
///
/// # Errors
///
/// 넘기는 중이 아니거나, 저장하지 못했거나, 저장한 파일을 읽을 수 없으면 화면에 보여 줄 문장.
pub async fn save_file(
    state: &Option<HandingOver>,
    run_mode: &RunModeState,
    target: &Path,
) -> Result<(), String> {
    let source = handing_over_file(state)?;
    copy_checked(source, target).await?;
    run_mode::save(run_mode.data_dir(), RunMode::HandedOver).map_err(|error| {
        tracing::error!(%error, "실행 모드를 기억하지 못했습니다");
        "넘기기 파일은 저장했지만 이 PC를 넘김 상태로 바꾸지 못했습니다. 데이터 폴더를 쓸 수 있는지 확인해 주세요".to_owned()
    })?;
    tracing::info!("호스트를 넘겼습니다");
    Ok(())
}

/// 저장 창에 미리 채울 이름. 예: `배움마루 넘기기 2026-11-25.db`
pub fn file_name(now: OffsetDateTime) -> String {
    format!("배움마루 넘기기 {}.db", clock::local_date(now))
}

/// `source`를 `target`에 복사하고 다시 읽어 본다. USB가 빠지거나 가득 차도 알아채게 한다.
pub(in crate::features::handover) async fn copy_checked(
    source: &Path,
    target: &Path,
) -> Result<(), String> {
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

/// 넘기지 않고 다시 호스트로 기억한다. 넘기는 중일 때만. 아직 아무 데도 넘기지 않아 PIN을 묻지 않는다.
///
/// # Errors
///
/// 넘기는 중이 아니거나 기억하지 못하면 화면에 보여 줄 문장.
pub async fn abort(run_mode: &RunModeState) -> Result<(), String> {
    if run_mode.mode() != Some(RunMode::HandingOver) {
        return Err("호스트를 넘기는 중이 아닙니다".to_owned());
    }
    back_to_host(run_mode.data_dir(), None, OffsetDateTime::now_utc()).await?;
    tracing::info!("넘기지 않고 호스트로 돌아갑니다");
    Ok(())
}

/// 넘기기를 취소할 수 있는 관리자들. 넘긴 PC에서만.
///
/// # Errors
///
/// 넘긴 PC가 아니거나 DB를 열지 못하면 화면에 보여 줄 문장.
pub async fn admins(run_mode: &RunModeState) -> Result<Vec<HandedOverAdmin>, String> {
    let pool = open_handed_over(run_mode).await?;
    let listed = auth::list_for_login(&pool, clock::local_today()).await;
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

/// 관리자 PIN을 확인하고 넘기기를 취소해 다시 호스트로 기억한다(BR-H2). 세대 번호는 그대로다.
/// 확인한 관리자 id를 돌려준다. 다시 켜기는 부르는 쪽이 한다.
///
/// 틀린 PIN은 로그인과 같이 BR-K7의 횟수에 들어간다.
///
/// # Errors
///
/// 넘긴 PC가 아니거나, 관리자가 아니거나, PIN이 틀렸거나, 기억하지 못하면 화면에 보여 줄 문장.
pub async fn cancel(run_mode: &RunModeState, staff_id: i64, pin: String) -> Result<i64, String> {
    let pool = open_handed_over(run_mode).await?;
    let now = OffsetDateTime::now_utc();
    let checked = check_admin_pin(&pool, staff_id, pin, now).await;
    pool.close().await;
    let staff_id = checked?;
    back_to_host(run_mode.data_dir(), Some(staff_id), now).await?;
    tracing::info!(staff_id, "넘기기를 취소하고 호스트로 돌아갑니다");
    Ok(staff_id)
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
pub(in crate::features::handover) async fn check_admin_pin(
    pool: &SqlitePool,
    staff_id: i64,
    pin: String,
    now: OffsetDateTime,
) -> Result<i64, String> {
    let account = auth::get_login_account(pool, staff_id)
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
pub(in crate::features::handover) async fn back_to_host(
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
