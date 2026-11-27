import { useEffect, useState, type ReactNode } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { SplitAuthLayout } from "../../../components/shell/SplitAuthLayout";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import authStyles from "../auth/auth.module.css";
import { TakeOverPage } from "../handover/TakeOverPage";
import { RunModeSwitch } from "../run-mode/RunModeSwitch";
import { AddressForm } from "./AddressForm";
import {
  clearDeviceRequest,
  forgetHost,
  rememberApproved,
  remoteSettings,
} from "./api";
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
  // BR-H3: 이 PC가 넘겨받아 새 호스트가 된다
  const [takingOver, setTakingOver] = useState(false);
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
    <SplitAuthLayout
      panelTitle="호스트 PC 접속"
      panelDescription="호스트 PC의 주소를 입력하고 관리자 승인을 받으면 같은 데이터를 함께 쓸 수 있습니다."
      cardTitle="다른 PC에 접속"
      cardFooter={
        <div className={authStyles.stack}>
          {leftoverData && (
            <p className={authStyles.hint}>
              이 PC에 이전 데이터가 남아 있습니다. 지우지 않았고, 접속
              모드에서는 쓰지 않습니다.
            </p>
          )}
          <div className={authStyles.actionsBetween}>
            {/* 승인받기 전이면 아직 쓴 데이터가 없어 PIN 없이 되돌린다 */}
            <RunModeSwitch current="remote" needsPin={approvedOnce} />
            <Button
              size="sm"
              variant="secondary"
              onClick={() => setTakingOver(true)}
            >
              넘겨받은 파일로 호스트 시작
            </Button>
          </div>
        </div>
      }
    >
      <div className={authStyles.stack}>
        {notice && <NoticeBanner tone="error" message={notice} />}
        {body}
      </div>
    </SplitAuthLayout>
  );

  if (takingOver) {
    return <TakeOverPage onBack={() => setTakingOver(false)} />;
  }
  if (settings.isPending) {
    return page(<NoticeBanner message="앱을 준비하는 중…" />);
  }
  if (settings.isError) {
    return page(<NoticeBanner tone="error" message={settings.error.message} />);
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
    return page(<NoticeBanner message="호스트에 연결하는 중…" />);
  }

  const retry = (
    <div className={authStyles.actions}>
      <Button variant="secondary" onClick={() => setEditingAddress(true)}>
        주소 다시 입력
      </Button>
      <Button variant="primary" onClick={() => void connection.refetch()}>
        다시 시도
      </Button>
    </div>
  );

  // BR-H5: 붙지 않을 호스트에서는 다시 시도해도 같다
  const addressOnly = (
    <div className={authStyles.actions}>
      <Button variant="primary" onClick={() => setEditingAddress(true)}>
        주소 다시 입력
      </Button>
    </div>
  );

  if (connection.isError) {
    return page(
      <>
        <NoticeBanner tone="error" message={connection.error.message} />
        {retry}
      </>,
    );
  }
  switch (connection.data.kind) {
    case "unreachable":
      return page(
        <>
          <NoticeBanner
            tone="error"
            message={`호스트 PC(${address})에 연결하지 못했습니다. 호스트 PC가 꺼져 있거나 주소가 바뀌었을 수 있습니다.`}
          />
          {retry}
        </>,
      );
    case "incompatible":
      return page(
        <>
          <NoticeBanner
            tone="error"
            message={`호스트 PC의 배움마루(${connection.data.hostVersion})와 이 PC의 버전이 맞지 않습니다. 두 PC를 같은 버전으로 맞춰 주세요.`}
          />
          {retry}
        </>,
      );
    case "olderHost":
      return page(
        <>
          <NoticeBanner
            tone="error"
            message={`이 주소의 호스트 PC(${address})는 다른 PC로 호스트를 넘기기 전의 옛 데이터를 갖고 있습니다. 여기에 접수하면 데이터가 둘로 갈라집니다. 새 호스트 PC의 주소를 넣어 주세요.`}
          />
          {addressOnly}
        </>,
      );
    case "otherHost":
      return page(
        <>
          <NoticeBanner
            tone="error"
            message={`이 주소의 호스트 PC(${address})는 이 PC가 쓰던 것과 다른 데이터를 갖고 있습니다. 쓰던 호스트 PC의 주소를 넣어 주세요. 호스트 PC를 새로 설치했다면 이 호스트에 새로 승인을 요청해 주세요.`}
          />
          <div className={authStyles.actions}>
            <Button variant="secondary" onClick={() => setEditingAddress(true)}>
              주소 다시 입력
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                setNotice(null);
                void forgetHost()
                  .then(reloadSettings)
                  .catch((error: unknown) =>
                    setNotice(
                      error instanceof Error
                        ? error.message
                        : "처리하지 못했습니다",
                    ),
                  );
              }}
            >
              이 호스트에 새로 승인 요청
            </Button>
          </div>
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
          <NoticeBanner
            tone="error"
            message={
              connection.data.reason === "rejected"
                ? "호스트 PC에서 승인을 거절했습니다."
                : "5분 안에 승인되지 않았습니다."
            }
          />
          <div className={authStyles.actions}>
            <Button variant="primary" onClick={() => void reloadSettings()}>
              다시 요청
            </Button>
          </div>
        </>,
      );
    case "ready":
      return children;
  }
}
