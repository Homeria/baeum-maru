import { useMutation } from "@tanstack/react-query";
import { restartApp } from "../../api/appCommands";
import { prepareRestore, type RestoreSource } from "../../api/backups";
import type { RestorePreviewResponse } from "../../api/generated/RestorePreviewResponse";
import { ConfirmDialog } from "../../components/dialog/ConfirmDialog";
import { previewLines } from "./backupData";
import styles from "./BackupPanel.module.css";

/** {@link RestoreDialog}의 속성. */
export interface RestoreDialogProps {
  source: RestoreSource;
  preview: RestorePreviewResponse;
  onCancel: () => void;
}

/**
 * 복원 확인 창(ADR 0011). 미리보기를 보여 주고, 확인하면 복원을 준비한 뒤 앱을 다시 켠다.
 *
 * 다시 켜지 못해도 준비는 남아서, 앱을 직접 껐다 켜면 복원된다.
 */
export function RestoreDialog({
  source,
  preview,
  onCancel,
}: RestoreDialogProps) {
  const restore = useMutation({
    mutationFn: async () => {
      await prepareRestore(source);
      await restartApp();
    },
  });

  return (
    <ConfirmDialog
      title="이 백업으로 되돌릴까요?"
      warning="지금 데이터는 복원 직전 백업으로 남습니다. 앱이 다시 켜지고, 모두 다시 로그인합니다."
      confirmLabel="되돌리기"
      busyLabel="되돌리는 중…"
      busy={restore.isPending}
      onConfirm={() => restore.mutate()}
      onCancel={onCancel}
    >
      <ul className={styles.preview}>
        {previewLines(preview).map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
      {restore.isError && (
        <p className={styles.warning} role="alert">
          {restore.error.message}
        </p>
      )}
    </ConfirmDialog>
  );
}
