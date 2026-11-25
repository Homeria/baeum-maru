import { useEffect, useRef, useState } from "react";
import formStyles from "../../components/form/form.module.css";
import { DeviceRequestList } from "./DeviceRequestList";
import styles from "./devices.module.css";
import { useOpenRequestCount } from "./useOpenRequestCount";

/** {@link DeviceRequestNotice}의 속성. */
export interface DeviceRequestNoticeProps {
  /** 관리자가 호스트 PC에서 로그인해 있고 화면이 잠기지 않았다. */
  active: boolean;
}

/**
 * 호스트 화면 위쪽의 알림 줄(BR-K15). 새 기기가 승인을 기다리면 어느 탭에서든 보인다.
 *
 * "확인하기"를 누르면 요청 목록 창을 띄워 그 자리에서 허락하거나 거절한다.
 */
export function DeviceRequestNotice({ active }: DeviceRequestNoticeProps) {
  const openCount = useOpenRequestCount(active);
  const [open, setOpen] = useState(false);

  if (!active || (openCount === 0 && !open)) {
    return null;
  }
  return (
    <>
      {openCount > 0 && (
        <div className={styles.bar} role="status">
          <span>승인을 기다리는 기기가 {openCount}대 있습니다</span>
          <button type="button" onClick={() => setOpen(true)}>
            확인하기
          </button>
        </div>
      )}
      {open && (
        <RequestsDialog openCount={openCount} onClose={() => setOpen(false)} />
      )}
    </>
  );
}

function RequestsDialog({
  openCount,
  onClose,
}: {
  openCount: number;
  onClose: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    dialog.current?.showModal();
  }, []);

  return (
    <dialog
      ref={dialog}
      className={styles.dialog}
      aria-labelledby="device-requests-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
    >
      <div className={styles.dialogHeader}>
        <h2 id="device-requests-title" className={styles.heading}>
          승인을 기다리는 기기
        </h2>
        <button
          type="button"
          className={formStyles.secondary}
          onClick={onClose}
        >
          닫기
        </button>
      </div>
      {notice && (
        <p className={styles.notice} role="status">
          {notice}
        </p>
      )}
      <DeviceRequestList openCount={openCount} onDone={setNotice} />
    </dialog>
  );
}
