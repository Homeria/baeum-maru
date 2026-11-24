import { useEffect, useState, type ReactNode } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  clearDeviceRequest,
  rememberApproved,
  remoteSettings,
} from "../../api/remote";
import formStyles from "../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";
import { RunModeSwitch } from "../run-mode/RunModeSwitch";
import { AddressForm } from "./AddressForm";
import { checkConnection } from "./connection";
import { RequestApproval } from "./RequestApproval";
import { WaitingApproval } from "./WaitingApproval";

/** {@link RemoteGate}의 속성. */
export interface RemoteGateProps {
  /** 이 PC에 예전 DB가 남아 있다(ADR 0021). */
  leftoverData: boolean;
  /** 승인받아 호스트에 이어진 뒤에 보여 줄 것(로그인 등). */
  children: ReactNode;
}

/**
 * 접속 모드(ADR 0021)의 첫 관문: 호스트 주소를 넣고(ADR 0004), 이 PC의 승인을 받는다(BR-K15).
 *
 * 승인받아 호스트에 이어지면 `children`을 보여 준다. 쓰는 중에 승인이 취소되면 다시 요청 화면으로
 * 돌아온다(BR-K16).
 */
export function RemoteGate({ leftoverData, children }: RemoteGateProps) {
  const queryClient = useQueryClient();
  const [editingAddress, setEditingAddress] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const settings = useQuery({
    queryKey: ["remoteSettings"],
    queryFn: remoteSettings,
    staleTime: Infinity,
  });
  const address = settings.data?.hostAddress ?? null;
  const token = settings.data?.deviceToken ?? null;

  const reloadSettings = () =>
    queryClient.invalidateQueries({ queryKey: ["remoteSettings"] });

  const onRejected = () => {
    setNotice("이 PC의 승인이 취소되었습니다. 다시 요청해 주세요.");
    void clearDeviceRequest().then(reloadSettings);
  };

  const connection = useQuery({
    queryKey: ["remoteConnection", address, token],
    queryFn: () => checkConnection(address ?? "", token, onRejected),
    enabled: address !== null && !editingAddress,
    retry: false,
    staleTime: Infinity,
  });

  const ready = connection.data?.kind === "ready";
  const approvedOnce = settings.data?.approvedOnce ?? true;
  useEffect(() => {
    // BR-K17: 한 번 승인받은 뒤로는 호스트로 바꿀 때 관리자 PIN이 필요하다
    if (ready && !approvedOnce) {
      void rememberApproved().then(() =>
        queryClient.invalidateQueries({ queryKey: ["remoteSettings"] }),
      );
    }
  }, [ready, approvedOnce, queryClient]);

  const page = (body: ReactNode) => (
    <div className={authStyles.page}>
      <section className={`${formStyles.form} ${authStyles.card}`}>
        <h2 className={formStyles.title}>다른 PC에 접속</h2>
        {notice && (
          <p className={formStyles.formError} role="alert">
            {notice}
          </p>
        )}
        {body}
        {leftoverData && (
          <p className={authStyles.hint}>
            이 PC에 이전 데이터가 남아 있습니다. 지우지 않았고, 접속 모드에서는
            쓰지 않습니다.
          </p>
        )}
      </section>
      {/* 승인받기 전이면 아직 쓴 데이터가 없어 PIN 없이 되돌린다 */}
      <RunModeSwitch current="remote" needsPin={approvedOnce} />
    </div>
  );

  if (settings.isPending) {
    return <p className={formStyles.loading}>앱을 준비하는 중…</p>;
  }
  if (settings.isError) {
    return page(
      <p className={formStyles.formError} role="alert">
        {settings.error.message}
      </p>,
    );
  }
  if (address === null || editingAddress) {
    return page(
      <AddressForm
        current={address}
        onSaved={async () => {
          setEditingAddress(false);
          setNotice(null);
          await reloadSettings();
        }}
        onCancel={address === null ? null : () => setEditingAddress(false)}
      />,
    );
  }
  if (connection.isPending) {
    return <p className={formStyles.loading}>호스트에 연결하는 중…</p>;
  }

  const retry = (
    <div className={formStyles.actions}>
      <button
        type="button"
        className={formStyles.secondary}
        onClick={() => setEditingAddress(true)}
      >
        주소 다시 입력
      </button>
      <button
        type="button"
        className={formStyles.submit}
        onClick={() => void connection.refetch()}
      >
        다시 시도
      </button>
    </div>
  );

  if (connection.isError) {
    return page(
      <>
        <p className={formStyles.formError} role="alert">
          {connection.error.message}
        </p>
        {retry}
      </>,
    );
  }
  switch (connection.data.kind) {
    case "unreachable":
      return page(
        <>
          <p className={authStyles.intro} role="alert">
            호스트 PC({address})에 연결하지 못했습니다. 호스트 PC가 꺼져 있거나
            주소가 바뀌었을 수 있습니다.
          </p>
          {retry}
        </>,
      );
    case "incompatible":
      return page(
        <>
          <p className={authStyles.intro} role="alert">
            호스트 PC의 배움마루({connection.data.hostVersion})와 이 PC의 버전이
            맞지 않습니다. 두 PC를 같은 버전으로 맞춰 주세요.
          </p>
          {retry}
        </>,
      );
    case "request":
      return page(
        <RequestApproval
          computerName={settings.data.computerName}
          onRequested={async () => {
            setNotice(null);
            await reloadSettings();
          }}
        />,
      );
    case "waiting":
      return page(
        <WaitingApproval
          code={settings.data.approvalCode}
          onChanged={() => void connection.refetch()}
        />,
      );
    case "refused":
      return page(
        <>
          <p className={authStyles.intro} role="alert">
            {connection.data.reason === "rejected"
              ? "호스트 PC에서 승인을 거절했습니다."
              : "5분 안에 승인되지 않았습니다."}
          </p>
          <div className={formStyles.actions}>
            <button
              type="button"
              className={formStyles.submit}
              onClick={() => void reloadSettings()}
            >
              다시 요청
            </button>
          </div>
        </>,
      );
    case "ready":
      return children;
  }
}
