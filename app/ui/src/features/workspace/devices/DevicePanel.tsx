import { useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { DeviceResponse } from "../../../api/generated/DeviceResponse";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { useIsHostPc } from "../../gate/run-mode/useRunMode";
import { formatOccurredAt } from "../access-logs/accessLogData";
import { hostAddress, listDevices, revokeDevice } from "./api";
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
    <div className={styles.panel}>
      {notice && <NoticeBanner message={notice} />}
      {host ? (
        <>
          <HostAddress />
          <Card title="승인을 기다리는 기기">
            <DeviceRequestList openCount={openCount} onDone={setNotice} />
          </Card>
        </>
      ) : (
        <NoticeBanner message="새 기기 승인은 호스트 PC에서 합니다." />
      )}
      <ApprovedDevices onDone={setNotice} />
    </div>
  );
}

function HostAddress() {
  const { data, isPending } = useQuery({
    queryKey: ["hostAddress"],
    queryFn: hostAddress,
  });

  return (
    <Card title="이 PC의 접속 주소">
      {isPending ? (
        <p className={styles.muted}>찾는 중입니다…</p>
      ) : data ? (
        <div className={styles.addressBlock}>
          <p className={styles.address}>{data}</p>
          <p className={styles.muted}>
            다른 PC에서 배움마루를 처음 켜고 &ldquo;다른 PC에 접속&rdquo;을 고른
            뒤 이 주소를 넣어 주세요.
          </p>
        </div>
      ) : (
        <p className={styles.muted}>
          주소를 찾지 못했습니다. 이 PC가 공유기에 연결되어 있는지 확인해
          주세요.
        </p>
      )}
    </Card>
  );
}

function ApprovedDevices({ onDone }: { onDone: (message: string) => void }) {
  const queryClient = useQueryClient();
  const [revoking, setRevoking] = useState<DeviceResponse | null>(null);
  const cancelRevokeRef = useRef<HTMLButtonElement>(null);
  const { data, isPending, isError, error, refetch } = useQuery({
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

  const columns: DataTableColumn<DeviceResponse>[] = [
    {
      key: "name",
      header: "기기",
      isRowHeader: true,
      render: (device) => device.name,
    },
    {
      key: "approvedAt",
      header: "허락한 때",
      width: "200px",
      render: (device) => formatOccurredAt(device.approvedAt),
    },
    {
      key: "approvedBy",
      header: "허락한 사람",
      width: "160px",
      render: (device) => device.approvedBy,
    },
    {
      key: "actions",
      header: "",
      width: "120px",
      align: "right",
      isAction: true,
      render: (device) => (
        <Button size="sm" variant="danger" onClick={() => setRevoking(device)}>
          승인 취소
        </Button>
      ),
    },
  ];

  return (
    <>
      <Card
        title="승인한 기기"
        subtitle={data ? `모두 ${data.items.length}대` : undefined}
        padding="none"
      >
        <DataTable
          caption="승인한 기기 목록"
          columns={columns}
          rows={data?.items ?? []}
          getRowKey={(device) => device.id}
          loading={isPending}
          loadingText="승인한 기기를 불러오는 중입니다…"
          error={isError ? error.message : null}
          onRetry={() => void refetch()}
          emptyMessage="승인한 기기가 없습니다. 지금은 이 PC에서만 씁니다."
        />
      </Card>

      <ModalDialog
        open={revoking !== null}
        size="sm"
        title={`${revoking?.name ?? ""}의 승인을 취소할까요?`}
        busy={revoke.isPending}
        initialFocusRef={cancelRevokeRef}
        onClose={() => setRevoking(null)}
        footer={
          <>
            <Button
              ref={cancelRevokeRef}
              variant="secondary"
              disabled={revoke.isPending}
              onClick={() => setRevoking(null)}
            >
              그만두기
            </Button>
            <Button
              variant="dangerSolid"
              busy={revoke.isPending}
              busyLabel="취소하는 중…"
              onClick={() => {
                if (revoking) {
                  revoke.mutate(revoking);
                }
              }}
            >
              승인 취소
            </Button>
          </>
        }
      >
        <div className={styles.dialogBody}>
          <NoticeBanner
            tone="error"
            message="그 기기는 바로 배움마루를 쓸 수 없게 됩니다. 다시 쓰려면 승인을 새로 받아야 합니다."
          />
          {revoke.isError && (
            <NoticeBanner tone="error" message={revoke.error.message} />
          )}
        </div>
      </ModalDialog>
    </>
  );
}
