import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { hostAddress, listDevices, revokeDevice } from "./api";
import type { DeviceResponse } from "../../../api/generated/DeviceResponse";
import { ConfirmDialog } from "../../../components/dialog/ConfirmDialog";
import tableStyles from "../../../components/table/table.module.css";
import { formatOccurredAt } from "../access-logs/accessLogData";
import { useIsHostPc } from "../../gate/run-mode/useRunMode";
import { DeviceRequestList } from "./DeviceRequestList";
import styles from "./devices.module.css";
import { useOpenRequestCount } from "./useOpenRequestCount";

/**
 * 기기 탭(BR-K15, BR-K16). 관리자에게만 보인다.
 *
 * 호스트 PC에서는 접속 주소와 기다리는 승인 요청도 보여 준다. 승인한 기기 목록과 승인 취소는
 * 어느 PC에서나 한다.
 */
export function DevicePanel() {
  const host = useIsHostPc();
  const openCount = useOpenRequestCount(host);
  const [notice, setNotice] = useState<string | null>(null);

  return (
    <section className={styles.panel} aria-label="기기">
      {notice && (
        <p className={styles.notice} role="status">
          {notice}
        </p>
      )}
      {host ? (
        <>
          <HostAddress />
          <div className={styles.section}>
            <h3 className={styles.heading}>승인을 기다리는 기기</h3>
            <DeviceRequestList openCount={openCount} onDone={setNotice} />
          </div>
        </>
      ) : (
        <p className={styles.muted}>새 기기 승인은 호스트 PC에서 합니다.</p>
      )}
      <ApprovedDevices onDone={setNotice} />
    </section>
  );
}

function HostAddress() {
  const { data, isPending } = useQuery({
    queryKey: ["hostAddress"],
    queryFn: hostAddress,
  });

  return (
    <div className={styles.section}>
      <h3 className={styles.heading}>이 PC의 접속 주소</h3>
      {isPending ? (
        <p className={styles.muted}>찾는 중입니다…</p>
      ) : data ? (
        <>
          <p className={styles.address}>{data}</p>
          <p className={styles.muted}>
            다른 PC에서 배움마루를 처음 켜고 "다른 PC에 접속"을 고른 뒤 이
            주소를 넣어 주세요.
          </p>
        </>
      ) : (
        <p className={styles.muted}>
          주소를 찾지 못했습니다. 이 PC가 공유기에 연결되어 있는지 확인해
          주세요.
        </p>
      )}
    </div>
  );
}

function ApprovedDevices({ onDone }: { onDone: (message: string) => void }) {
  const queryClient = useQueryClient();
  const [revoking, setRevoking] = useState<DeviceResponse | null>(null);
  const { data, isPending, isError, error } = useQuery({
    queryKey: ["devices"],
    queryFn: listDevices,
  });
  const revoke = useMutation({
    mutationFn: (device: DeviceResponse) => revokeDevice(device.id),
    onSuccess: async (_, device) => {
      setRevoking(null);
      onDone(`${device.name}의 승인을 취소했습니다`);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["devices"] }),
        queryClient.invalidateQueries({ queryKey: ["accessLogs"] }),
      ]);
    },
  });

  return (
    <div className={styles.section}>
      <h3 className={styles.heading}>승인한 기기</h3>
      {isPending ? (
        <p className={styles.muted}>불러오는 중입니다…</p>
      ) : isError ? (
        <p className={styles.error} role="alert">
          {error.message}
        </p>
      ) : data.items.length === 0 ? (
        <p className={styles.muted}>
          승인한 기기가 없습니다. 지금은 이 PC에서만 씁니다.
        </p>
      ) : (
        <table className={tableStyles.table} aria-label="승인한 기기 목록">
          <thead>
            <tr>
              <th scope="col">기기</th>
              <th scope="col">허락한 때</th>
              <th scope="col">허락한 사람</th>
              <th scope="col">할 일</th>
            </tr>
          </thead>
          <tbody>
            {data.items.map((device) => (
              <tr key={device.id}>
                <td>{device.name}</td>
                <td>{formatOccurredAt(device.approvedAt)}</td>
                <td>{device.approvedBy}</td>
                <td>
                  <button type="button" onClick={() => setRevoking(device)}>
                    승인 취소
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {revoking && (
        <ConfirmDialog
          title={`${revoking.name}의 승인을 취소할까요?`}
          warning="그 기기는 바로 배움마루를 쓸 수 없게 됩니다. 다시 쓰려면 승인을 새로 받아야 합니다."
          confirmLabel="승인 취소"
          busyLabel="취소하는 중…"
          busy={revoke.isPending}
          onConfirm={() => revoke.mutate(revoking)}
          onCancel={() => setRevoking(null)}
        >
          {revoke.isError && (
            <p className={styles.error} role="alert">
              {revoke.error.message}
            </p>
          )}
        </ConfirmDialog>
      )}
    </div>
  );
}
