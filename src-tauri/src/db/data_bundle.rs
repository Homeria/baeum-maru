//! 데이터 묶음 ID와 세대 번호(BR-H4). `data_bundle` 표의 한 줄이다.

use std::path::Path;

use baeum_domain::{DataBundleId, HostIdentity};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, SqliteExecutor};

/// 이 DB의 묶음 ID와 세대 번호.
///
/// # Errors
///
/// 읽지 못하거나, 표에 적힌 값이 형식에 맞지 않으면 오류.
pub async fn read(executor: impl SqliteExecutor<'_>) -> sqlx::Result<HostIdentity> {
    let (bundle_id, generation): (String, i64) =
        sqlx::query_as("SELECT bundle_id, generation FROM data_bundle WHERE id = 1")
            .fetch_one(executor)
            .await?;
    Ok(HostIdentity {
        bundle_id: DataBundleId::parse(&bundle_id)
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
        generation: u32::try_from(generation)
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
    })
}

/// 묶음 ID와 세대 번호를 `identity`로 바꾼다. 백업으로 되돌린 뒤 되돌리기 전 값을 지킬 때 쓴다.
///
/// # Errors
///
/// 쓰지 못하면 오류.
pub async fn write(executor: impl SqliteExecutor<'_>, identity: HostIdentity) -> sqlx::Result<()> {
    sqlx::query("UPDATE data_bundle SET bundle_id = ?, generation = ? WHERE id = 1")
        .bind(identity.bundle_id.to_string())
        .bind(i64::from(identity.generation))
        .execute(executor)
        .await?;
    Ok(())
}

/// 세대 번호를 하나 올리고 올린 값을 돌려준다(BR-H4). 넘겨받을 때 쓴다.
///
/// # Errors
///
/// 쓰지 못하면 오류.
pub async fn raise_generation(executor: impl SqliteExecutor<'_>) -> sqlx::Result<u32> {
    let generation: i64 = sqlx::query_scalar(
        "UPDATE data_bundle SET generation = generation + 1 WHERE id = 1 RETURNING generation",
    )
    .fetch_one(executor)
    .await?;
    u32::try_from(generation).map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

/// 열려 있지 않은 DB 파일에서 읽는다. 깨진 DB도 이 표만은 읽힐 수 있어 한번 해 본다.
/// 못 읽으면 `None`.
pub async fn read_file(file: &Path) -> Option<HostIdentity> {
    // 고치지 않고, 잠금이나 보조 파일을 만들지 않는다 (backup::inspect와 같다)
    let mut conn = SqliteConnectOptions::new()
        .filename(file)
        .read_only(true)
        .immutable(true)
        .connect()
        .await
        .ok()?;
    let identity = read(&mut conn).await;
    let _ = conn.close().await;
    identity
        .inspect_err(|error| tracing::warn!(%error, "데이터 묶음 정보를 읽지 못했습니다"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_temp;

    #[tokio::test]
    async fn br_h4_new_db_starts_at_generation_one() {
        let (_dir, pool) = open_temp().await;

        let identity = read(&pool).await.unwrap();

        assert_eq!(identity.generation, 1);
    }

    #[tokio::test]
    async fn br_h4_each_db_gets_its_own_bundle_id() {
        let (_a, one) = open_temp().await;
        let (_b, other) = open_temp().await;

        assert_ne!(
            read(&one).await.unwrap().bundle_id,
            read(&other).await.unwrap().bundle_id
        );
    }

    #[tokio::test]
    async fn written_identity_reads_back() {
        let (_dir, pool) = open_temp().await;
        let identity = HostIdentity {
            bundle_id: DataBundleId::from_bytes([7; 16]),
            generation: 4,
        };

        write(&pool, identity).await.unwrap();

        assert_eq!(read(&pool).await.unwrap(), identity);
    }

    #[tokio::test]
    async fn br_h4_raising_generation_keeps_bundle_id() {
        let (_dir, pool) = open_temp().await;
        let before = read(&pool).await.unwrap();

        assert_eq!(raise_generation(&pool).await.unwrap(), 2);

        let after = read(&pool).await.unwrap();
        assert_eq!(after.bundle_id, before.bundle_id);
        assert_eq!(after.generation, 2);
    }

    #[tokio::test]
    async fn row_cannot_be_deleted() {
        let (_dir, pool) = open_temp().await;

        let deleted = sqlx::query("DELETE FROM data_bundle").execute(&pool).await;

        assert!(deleted.is_err());
        assert!(read(&pool).await.is_ok());
    }

    #[tokio::test]
    async fn reads_closed_file() {
        let (dir, pool) = open_temp().await;
        let identity = read(&pool).await.unwrap();
        pool.close().await;

        assert_eq!(
            read_file(&crate::db::file(dir.path())).await,
            Some(identity)
        );
    }
}
