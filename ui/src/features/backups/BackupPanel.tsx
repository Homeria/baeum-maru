import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createBackup,
  listBackups,
  setSecondLocation,
} from "../../api/backups";
import { pickFolder } from "../../api/pickFolder";
import tableStyles from "../../components/table/table.module.css";
import {
  BACKUP_KIND_LABELS,
  formatSize,
  formatTakenAt,
  kindText,
  secondLocationWarning,
} from "./backupData";
import styles from "./BackupPanel.module.css";

const QUERY_KEY = ["backups"] as const;

/**
 * 백업 탭(ADR 0011). 관리자에게만 보인다.
 *
 * 백업 목록과 지금 백업, 두 번째 위치를 보여 준다. 두 번째 위치는 호스트 PC에서만 바꾼다.
 */
export function BackupPanel() {
  const queryClient = useQueryClient();
  const [notice, setNotice] = useState<string | null>(null);
  const { data, isPending, isError, error, refetch } = useQuery({
    queryKey: QUERY_KEY,
    queryFn: listBackups,
  });

  const refresh = () => queryClient.invalidateQueries({ queryKey: QUERY_KEY });

  const backupNow = useMutation({
    mutationFn: createBackup,
    onSuccess: async (created) => {
      setNotice(`백업했습니다 (${formatTakenAt(created.takenAt)})`);
      await refresh();
    },
  });

  const changeSecond = useMutation({
    mutationFn: async (turnOn: boolean) => {
      if (!turnOn) {
        return setSecondLocation(null);
      }
      const path = await pickFolder();
      // 폴더 고르기 창을 닫았으면 바꾸지 않는다
      return path === null ? null : setSecondLocation(path);
    },
    onSuccess: async (changed) => {
      if (changed === null) {
        return;
      }
      setNotice(
        changed.path === null
          ? "두 번째 위치를 껐습니다"
          : "두 번째 위치를 바꿨습니다",
      );
      await refresh();
    },
  });

  if (isPending) {
    return <p className={tableStyles.message}>불러오는 중입니다…</p>;
  }
  if (isError) {
    return (
      <p className={tableStyles.error} role="alert">
        {error.message}
      </p>
    );
  }

  const second = data.secondLocation;
  const secondWarning = secondLocationWarning(second, new Date());
  const busy = backupNow.isPending || changeSecond.isPending;
  const failure = backupNow.error ?? changeSecond.error;

  return (
    <section className={styles.panel} aria-label="백업">
      {data.lastFailure !== null && (
        <p className={styles.warning} role="alert">
          {formatTakenAt(data.lastFailure.at)}{" "}
          {BACKUP_KIND_LABELS[data.lastFailure.kind]} 백업에 실패했습니다.
          디스크 공간이 남아 있는지 확인해 주세요
        </p>
      )}
      {secondWarning !== null && (
        <p className={styles.warning} role="alert">
          {secondWarning}
        </p>
      )}

      <div className={styles.second}>
        <h2 className={styles.heading}>두 번째 위치</h2>
        <p className={styles.muted}>
          하루 한 번 백업을 다른 디스크나 USB의 폴더에도 한 부 더 둡니다.
        </p>
        {second.path === null ? (
          <p className={styles.path}>꺼져 있음</p>
        ) : (
          <>
            <p className={styles.path}>{second.path}</p>
            <p className={styles.muted}>
              마지막 복사:{" "}
              {second.lastCopiedAt === null
                ? "없음"
                : formatTakenAt(second.lastCopiedAt)}
            </p>
          </>
        )}
        {second.changeable ? (
          <div className={styles.actions}>
            <button
              type="button"
              disabled={busy}
              onClick={() => changeSecond.mutate(true)}
            >
              {second.path === null ? "폴더 고르기" : "다른 폴더 고르기"}
            </button>
            {second.path !== null && (
              <button
                type="button"
                disabled={busy}
                onClick={() => changeSecond.mutate(false)}
              >
                끄기
              </button>
            )}
          </div>
        ) : (
          <p className={styles.muted}>호스트 PC에서만 바꿀 수 있습니다</p>
        )}
      </div>

      <div className={styles.actions}>
        <button
          type="button"
          disabled={busy}
          onClick={() => {
            setNotice(null);
            backupNow.mutate();
          }}
        >
          {backupNow.isPending ? "백업하는 중…" : "지금 백업"}
        </button>
        <button type="button" onClick={() => void refetch()}>
          새로 보기
        </button>
        {failure !== null ? (
          <p className={styles.warning} role="alert">
            {failure.message}
          </p>
        ) : (
          notice !== null && (
            <p className={styles.notice} role="status">
              {notice}
            </p>
          )
        )}
      </div>

      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table} aria-label="백업 목록">
          <thead>
            <tr>
              <th scope="col">시각</th>
              <th scope="col">종류</th>
              <th scope="col">크기</th>
            </tr>
          </thead>
          <tbody>
            {data.items.map((item) => (
              <tr key={item.fileName}>
                <td>{formatTakenAt(item.takenAt)}</td>
                <td>{kindText(item)}</td>
                <td className={styles.size}>{formatSize(item.size)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {data.items.length === 0 && (
          <p className={tableStyles.message}>백업이 없습니다</p>
        )}
      </div>
    </section>
  );
}
