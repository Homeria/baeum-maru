import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { pickBackupFile } from "../../../api/appCommands";
import type { BackupResponse } from "../../../api/generated/BackupResponse";
import type { RestorePreviewResponse } from "../../../api/generated/RestorePreviewResponse";
import { pickFolder } from "../../../api/pickFolder";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { StatusPill } from "../../../components/ui/StatusPill";
import { HandoverDialog } from "../../gate/handover/HandoverDialog";
import {
  createBackup,
  listBackups,
  previewRestore,
  setSecondLocation,
  type RestoreSource,
} from "./api";
import {
  BACKUP_KIND_LABELS,
  formatSize,
  formatTakenAt,
  kindText,
  restoreOutcomeText,
  secondLocationWarning,
} from "./backupData";
import styles from "./BackupPanel.module.css";
import { RestoreDialog } from "./RestoreDialog";

const QUERY_KEY = ["backups"] as const;

/**
 * 백업 탭(ADR 0011). 관리자에게만 보인다.
 *
 * 백업 목록·지금 백업·복원, 두 번째 위치 카드, 분리된 위험 영역(호스트 넘기기, BR-H1)으로 나눈다.
 */
export function BackupPanel() {
  const queryClient = useQueryClient();
  const [notice, setNotice] = useState<string | null>(null);
  const [restoring, setRestoring] = useState<{
    source: RestoreSource;
    preview: RestorePreviewResponse;
  } | null>(null);
  const [handingOver, setHandingOver] = useState(false);
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

  const preview = useMutation({
    mutationFn: async (source: RestoreSource | "pick") => {
      if (source !== "pick") {
        return { source, preview: await previewRestore(source) };
      }
      const path = await pickBackupFile();
      // 파일 고르기 창을 닫았으면 하지 않는다
      if (path === null) {
        return null;
      }
      const picked: RestoreSource = { path };
      return { source: picked, preview: await previewRestore(picked) };
    },
    onSuccess: (found) => {
      if (found !== null) {
        setRestoring(found);
      }
    },
  });

  if (isPending) {
    return <NoticeBanner message="백업 목록을 불러오는 중입니다…" />;
  }
  if (isError) {
    return (
      <NoticeBanner
        tone="error"
        message={error.message}
        actionLabel="다시 해 보기"
        onAction={() => void refetch()}
      />
    );
  }

  const second = data.secondLocation;
  const secondWarning = secondLocationWarning(second, new Date());
  // 두 번째 위치를 바꿀 수 있는 PC가 호스트다. 복원도 호스트에서만 한다
  const onHost = second.changeable;
  const busy =
    backupNow.isPending || changeSecond.isPending || preview.isPending;
  const failure = backupNow.error ?? changeSecond.error ?? preview.error;

  const columns: DataTableColumn<BackupResponse>[] = [
    {
      key: "takenAt",
      header: "시각",
      isRowHeader: true,
      render: (item) => formatTakenAt(item.takenAt),
    },
    {
      key: "kind",
      header: "종류",
      render: (item) => kindText(item),
    },
    {
      key: "size",
      header: "크기",
      width: "120px",
      align: "right",
      render: (item) => formatSize(item.size),
    },
  ];

  if (onHost) {
    columns.push({
      key: "actions",
      header: "",
      width: "96px",
      align: "right",
      isAction: true,
      render: (item) => (
        <Button
          size="sm"
          variant="secondary"
          disabled={busy}
          aria-label={`${formatTakenAt(item.takenAt)} ${kindText(item)} 백업으로 복원`}
          onClick={() => {
            setNotice(null);
            preview.mutate({ fileName: item.fileName });
          }}
        >
          복원
        </Button>
      ),
    });
  }

  return (
    <div className={styles.panel}>
      {data.lastRestore !== null && (
        <NoticeBanner
          tone={data.lastRestore.restored ? "neutral" : "error"}
          message={restoreOutcomeText(data.lastRestore)}
        />
      )}
      {data.lastFailure !== null && (
        <NoticeBanner
          tone="error"
          message={`${formatTakenAt(data.lastFailure.at)} ${BACKUP_KIND_LABELS[data.lastFailure.kind]} 백업에 실패했습니다. 디스크 공간이 남아 있는지 확인해 주세요.`}
        />
      )}
      {secondWarning !== null && (
        <NoticeBanner tone="error" message={secondWarning} />
      )}
      {failure !== null && (
        <NoticeBanner tone="error" message={failure.message} />
      )}
      {failure === null && notice !== null && <NoticeBanner message={notice} />}

      <Card padding="none">
        <div className={styles.toolbar}>
          <div className={styles.headingGroup}>
            <h2 className={styles.title}>백업 목록</h2>
            <p className={styles.count}>모두 {data.items.length}개</p>
            {!onHost && (
              <p className={styles.muted}>
                복원과 호스트 넘기기는 호스트 PC에서만 할 수 있습니다.
              </p>
            )}
          </div>
          <div className={styles.actions}>
            <Button
              size="sm"
              variant="secondary"
              disabled={busy}
              onClick={() => void refetch()}
            >
              새로 보기
            </Button>
            {onHost && (
              <Button
                size="sm"
                variant="secondary"
                disabled={busy}
                onClick={() => {
                  setNotice(null);
                  preview.mutate("pick");
                }}
              >
                다른 곳의 백업 파일로 복원
              </Button>
            )}
            <Button
              variant="primary"
              busy={backupNow.isPending}
              busyLabel="백업하는 중…"
              disabled={changeSecond.isPending || preview.isPending}
              onClick={() => {
                setNotice(null);
                backupNow.mutate();
              }}
            >
              지금 백업
            </Button>
          </div>
        </div>

        <DataTable
          caption="백업 목록"
          columns={columns}
          rows={data.items}
          getRowKey={(item) => item.fileName}
          emptyMessage="저장된 백업이 없습니다."
        />
      </Card>

      <Card
        title="두 번째 위치"
        subtitle="하루 한 번 백업을 다른 디스크나 USB의 폴더에도 한 부 더 둡니다."
        actions={
          second.changeable ? (
            <div className={styles.actions}>
              <Button
                size="sm"
                variant="secondary"
                disabled={busy}
                onClick={() => changeSecond.mutate(true)}
              >
                {second.path === null ? "폴더 고르기" : "다른 폴더 고르기"}
              </Button>
              {second.path !== null && (
                <Button
                  size="sm"
                  variant="secondary"
                  disabled={busy}
                  onClick={() => changeSecond.mutate(false)}
                >
                  끄기
                </Button>
              )}
            </div>
          ) : undefined
        }
      >
        <div className={styles.secondBody}>
          <div className={styles.secondStatus}>
            <StatusPill
              tone={
                second.path === null
                  ? "neutral"
                  : second.reachable
                    ? "ok"
                    : "bad"
              }
            >
              {second.path === null
                ? "꺼져 있음"
                : second.reachable
                  ? "연결됨"
                  : "폴더를 찾을 수 없음"}
            </StatusPill>
            {second.path !== null && (
              <p className={styles.path}>{second.path}</p>
            )}
          </div>
          {second.path !== null && (
            <p className={styles.muted}>
              마지막 복사:{" "}
              {second.lastCopiedAt === null
                ? "없음"
                : formatTakenAt(second.lastCopiedAt)}
            </p>
          )}
          {!second.changeable && (
            <p className={styles.muted}>호스트 PC에서만 바꿀 수 있습니다.</p>
          )}
        </div>
      </Card>

      {onHost && (
        <Card className={styles.dangerCard}>
          <div className={styles.dangerRow}>
            <div className={styles.dangerText}>
              <h2 className={styles.dangerTitle}>호스트 넘기기</h2>
              <p className={styles.muted}>
                PC를 바꾸거나 다른 PC를 호스트로 쓸 때만 사용합니다. 넘기는 동안
                다른 PC에서는 배움마루를 쓸 수 없습니다.
              </p>
            </div>
            <Button
              variant="danger"
              disabled={busy}
              onClick={() => {
                setNotice(null);
                setHandingOver(true);
              }}
            >
              호스트 넘기기
            </Button>
          </div>
        </Card>
      )}

      {handingOver && <HandoverDialog onCancel={() => setHandingOver(false)} />}
      {restoring !== null && (
        <RestoreDialog
          source={restoring.source}
          preview={restoring.preview}
          onCancel={() => setRestoring(null)}
        />
      )}
    </div>
  );
}
