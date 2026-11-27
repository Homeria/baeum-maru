import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { approvalStatus } from "./approvalApi";
import authStyles from "../auth/auth.module.css";

/** 승인 상태를 다시 묻는 간격. */
const CHECK_EVERY_MS = 3 * 1000;

/** {@link WaitingApproval}의 속성. */
export interface WaitingApprovalProps {
  /** 이 PC가 받은 승인 번호. 앱이 기억하지 못했으면 `null`. */
  code: string | null;
  /** 허락·거절되었거나 5분이 지났다. */
  onChanged: () => void;
}

/** 승인 번호를 보여 주고 관리자를 기다린다(BR-K15). 결과가 나오면 알린다. */
export function WaitingApproval({ code, onChanged }: WaitingApprovalProps) {
  const status = useQuery({
    queryKey: ["approvalStatus"],
    queryFn: approvalStatus,
    refetchInterval: CHECK_EVERY_MS,
    gcTime: 0,
  });
  const current = status.data?.status;

  useEffect(() => {
    if (current !== undefined && current !== "pending") {
      onChanged();
    }
  }, [current, onChanged]);

  return (
    <div className={authStyles.stack}>
      <p className={authStyles.intro}>
        호스트 PC 화면에 같은 번호가 떴는지 관리자에게 확인을 부탁해 주세요. 5분
        안에 승인해야 합니다.
      </p>
      {code && (
        <p className={authStyles.code} aria-label="승인 번호">
          {code}
        </p>
      )}
      <p className={authStyles.hint} role="status">
        승인을 기다리는 중…
      </p>
    </div>
  );
}
