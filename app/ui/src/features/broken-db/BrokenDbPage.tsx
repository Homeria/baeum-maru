import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { pickBackupFile } from "../../api/appCommands";
import type { RestoreSource } from "../../api/backups";
import type { RestorePreviewResponse } from "../../api/generated/RestorePreviewResponse";
import type { StartupStateResponse } from "../../api/generated/StartupStateResponse";
import { previewBrokenDbBackup, restoreBrokenDb } from "../../api/startup";
import styles from "../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";
import {
  BACKUP_KIND_LABELS,
  formatTakenAt,
  previewLines,
} from "../backups/backupData";

/** {@link BrokenDbPage}의 속성. */
export interface BrokenDbPageProps {
  /** `state`가 `broken`인 응답. */
  startup: StartupStateResponse;
}

/** 되돌릴 백업과 그 미리보기. */
interface Chosen {
  source: RestoreSource;
  preview: RestorePreviewResponse;
  /** 앱이 권한 가장 최근 정상 백업인지. */
  suggested: boolean;
}

/**
 * DB가 깨졌을 때 로그인 없이 복원하는 화면(ADR 0011). 이 PC의 창에서만 뜬다.
 *
 * 가장 최근 정상 백업을 권하고, 다른 백업이나 다른 곳의 파일도 고를 수 있다. 데이터는 보여 주지
 * 않고 개수와 학기 이름만 보여 준다. 되돌리면 앱이 다시 켜지고 평소처럼 로그인한다.
 */
export function BrokenDbPage({ startup }: BrokenDbPageProps) {
  const [chosen, setChosen] = useState<Chosen | null>(
    startup.suggestion === null
      ? null
      : {
          source: { fileName: startup.suggestion.fileName },
          preview: startup.suggestion.preview,
          suggested: true,
        },
  );

  const choose = useMutation({
    mutationFn: async (source: RestoreSource | "pick") => {
      if (source !== "pick") {
        return {
          source,
          preview: await previewBrokenDbBackup(source),
          suggested: false,
        };
      }
      const path = await pickBackupFile();
      if (path === null) {
        return null;
      }
      const picked: RestoreSource = { path };
      return {
        source: picked,
        preview: await previewBrokenDbBackup(picked),
        suggested: false,
      };
    },
    onSuccess: (found) => {
      if (found !== null) {
        setChosen(found);
      }
    },
  });

  const restore = useMutation({
    mutationFn: (source: RestoreSource) => restoreBrokenDb(source),
  });

  const busy = choose.isPending || restore.isPending;
  const failure = restore.error ?? choose.error;

  return (
    <div className={authStyles.page}>
      <section className={`${styles.form} ${authStyles.card}`}>
        <h2 className={styles.title}>데이터 파일이 손상되었습니다</h2>
        <p className={authStyles.intro}>
          정전이나 디스크 문제로 데이터 파일을 읽을 수 없습니다. 백업으로
          되돌리면 다시 쓸 수 있습니다. 손상된 파일은 데이터 폴더에 남겨 둡니다.
        </p>

        {chosen === null ? (
          <p className={authStyles.warning} role="alert">
            쓸 수 있는 백업을 찾지 못했습니다. 두 번째 위치나 USB에 있는 백업
            파일을 골라 주세요.
          </p>
        ) : (
          <div className={authStyles.stack}>
            <p className={authStyles.intro}>
              {chosen.suggested
                ? "가장 최근 정상 백업으로 되돌릴까요?"
                : "이 백업으로 되돌릴까요?"}
            </p>
            <ul className={authStyles.steps}>
              {previewLines(chosen.preview).map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
            <div className={styles.actions}>
              <button
                type="button"
                className={styles.submit}
                disabled={busy}
                onClick={() => restore.mutate(chosen.source)}
              >
                {restore.isPending ? "되돌리는 중…" : "되돌리기"}
              </button>
            </div>
          </div>
        )}

        {failure !== null && (
          <p className={styles.formError} role="alert">
            {failure.message}
          </p>
        )}

        {startup.backups.length > 0 && (
          <div className={styles.field}>
            <label htmlFor="broken-db-backup" className={styles.label}>
              다른 백업 고르기
            </label>
            <select
              id="broken-db-backup"
              className={styles.input}
              disabled={busy}
              value=""
              onChange={(event) => {
                if (event.target.value !== "") {
                  choose.mutate({ fileName: event.target.value });
                }
              }}
            >
              <option value="">백업 목록에서 고르기</option>
              {startup.backups.map((backup) => (
                <option key={backup.fileName} value={backup.fileName}>
                  {formatTakenAt(backup.takenAt)} ·{" "}
                  {BACKUP_KIND_LABELS[backup.kind]}
                </option>
              ))}
            </select>
          </div>
        )}
        <div className={styles.actions}>
          <button
            type="button"
            className={styles.secondary}
            disabled={busy}
            onClick={() => choose.mutate("pick")}
          >
            다른 곳의 백업 파일 고르기
          </button>
        </div>
      </section>
    </div>
  );
}
