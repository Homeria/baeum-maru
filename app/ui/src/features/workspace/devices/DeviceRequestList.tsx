import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { DeviceRequestResponse } from "../../../api/generated/DeviceRequestResponse";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import { approveDevice, listDeviceRequests, rejectDevice } from "./api";
import styles from "./devices.module.css";

/** {@link DeviceRequestList}의 속성. */
export interface DeviceRequestListProps {
  /** 앱이 알려 준 기다리는 요청 수. 바뀌면 목록을 다시 받는다. */
  openCount: number;
  /** 허락하거나 거절했다. 예: "접수 노트북을 허락했습니다" */
  onDone: (message: string) => void;
}

/**
 * 기다리는 승인 요청(BR-K15). 관리자가 호스트 PC에서 본다.
 *
 * 요청마다 승인 번호를 크게 보여 준다. 관리자가 요청한 PC의 번호와 같은지 보고 PIN을 넣어 허락한다.
 */
export function DeviceRequestList({
  openCount,
  onDone,
}: DeviceRequestListProps) {
  const { data, isPending, isError, error, refetch } = useQuery({
    // 앱이 알려 준 수가 바뀌면 새로 받는다
    queryKey: ["deviceRequests", openCount],
    queryFn: listDeviceRequests,
  });

  if (isPending) {
    return <NoticeBanner message="승인 요청을 불러오는 중입니다…" />;
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
  if (data.items.length === 0) {
    return <p className={styles.muted}>승인을 기다리는 기기가 없습니다.</p>;
  }
  return (
    <ul className={styles.requests} aria-label="승인을 기다리는 기기">
      {data.items.map((request) => (
        <DeviceRequestItem key={request.id} request={request} onDone={onDone} />
      ))}
    </ul>
  );
}

function DeviceRequestItem({
  request,
  onDone,
}: {
  request: DeviceRequestResponse;
  onDone: (message: string) => void;
}) {
  const queryClient = useQueryClient();
  const [pin, setPin] = useState("");

  const refresh = async () => {
    await Promise.all(
      ["deviceRequests", "openDeviceRequestCount", "devices", "accessLogs"].map(
        (key) => queryClient.invalidateQueries({ queryKey: [key] }),
      ),
    );
  };

  const approve = useMutation({
    mutationFn: () => approveDevice(request.id, pin),
    onSuccess: async () => {
      onDone(`${request.name}을(를) 허락했습니다`);
      await refresh();
    },
    // 틀린 PIN은 지우고 다시 넣게
    onError: () => setPin(""),
  });
  const reject = useMutation({
    mutationFn: () => rejectDevice(request.id),
    onSuccess: async () => {
      onDone(`${request.name}을(를) 거절했습니다`);
      await refresh();
    },
  });
  const busy = approve.isPending || reject.isPending;
  const failure = approve.error ?? reject.error;

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!busy && pin !== "") {
      approve.mutate();
    }
  };

  return (
    <li className={styles.request}>
      <div className={styles.requestMeta}>
        <p className={styles.requestName}>{request.name}</p>
        <p className={styles.muted}>그 PC 화면의 번호가 같은지 보세요</p>
      </div>
      <p className={styles.code} aria-label="승인 번호">
        {request.code}
      </p>
      <form className={styles.approve} onSubmit={onSubmit} noValidate>
        <TextField
          label="관리자 PIN"
          type="password"
          inputMode="numeric"
          autoComplete="off"
          maxLength={8}
          value={pin}
          onChange={(event) => setPin(event.target.value)}
          className={styles.pinField}
        />
        <Button
          type="submit"
          variant="primary"
          busy={approve.isPending}
          busyLabel="허락하는 중…"
          disabled={reject.isPending || pin === ""}
        >
          허락
        </Button>
        <Button
          variant="secondary"
          disabled={busy}
          onClick={() => reject.mutate()}
        >
          거절
        </Button>
      </form>
      {failure && (
        <div className={styles.requestError}>
          <NoticeBanner tone="error" message={failure.message} />
        </div>
      )}
    </li>
  );
}
