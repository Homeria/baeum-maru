import { useEffect, useId, useRef, useState } from "react";
import type { ContactResultCode } from "../../api/generated/ContactResultCode";
import type { ResultMemberResponse } from "../../api/generated/ResultMemberResponse";
import dialogStyles from "../../components/dialog/ConfirmDialog.module.css";
import formStyles from "../../components/form/form.module.css";
import styles from "./SelectionResultsPage.module.css";

/** 선발 결과에서 할 일. 고른 회원과 함께 창을 연다. */
export type ChangeRequest =
  | { kind: "withdraw"; member: ResultMemberResponse; from: "당첨" | "대기" }
  | {
      kind: "contact";
      member: ResultMemberResponse;
      phoneNumber: string | null;
      waitlistNumber: number;
    }
  | {
      kind: "correct";
      member: ResultMemberResponse;
      /** 되돌아갈 곳. 예: "대기 2번" */
      restoreTo: string;
    };

/** 창에서 고른 것. 호스트에 보낼 내용이다. */
export type ChangeChoice =
  | { kind: "withdraw"; reason: string | null }
  | { kind: "contact"; result: ContactResultCode }
  | { kind: "correct"; reason: string };

/** {@link ChangeDialog}의 속성. */
export interface ChangeDialogProps {
  request: ChangeRequest;
  /** 지금 빈자리. 0이면 수락을 막는다(BR-W3). */
  openSeats: number;
  busy: boolean;
  /** 사유 칸에 붙일 호스트 오류. 예: 너무 김 */
  reasonError: string | null;
  /** 사유 칸에 붙일 수 없는 호스트 오류. 예: 빈자리 없음 */
  formError: string | null;
  onChoose: (choice: ChangeChoice) => void;
  onCancel: () => void;
}

/**
 * 포기, 대기자 연락 결과, 잘못 처리 바로잡기 창.
 *
 * 전화를 받으며 쓰는 창이라 누구의 일인지 이름과 회원번호를 크게 보여 준다.
 * Esc나 "그만두기"로 닫는다. 처음 커서는 "그만두기"에 둔다.
 */
export function ChangeDialog({
  request,
  openSeats,
  busy,
  reasonError,
  formError,
  onChoose,
  onCancel,
}: ChangeDialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const reasonId = useId();
  const [reason, setReason] = useState("");
  const [blankReason, setBlankReason] = useState(false);

  useEffect(() => {
    dialog.current?.showModal();
  }, []);

  const shownReasonError =
    reasonError ?? (blankReason ? "바로잡는 사유를 적어 주세요" : null);
  const reasonField = (required: boolean) => (
    <div className={styles.dialogField}>
      <label htmlFor={reasonId} className={formStyles.label}>
        {required ? "사유 (필수)" : "사유 (선택)"}
      </label>
      <input
        id={reasonId}
        className={formStyles.input}
        value={reason}
        onChange={(event) => {
          setReason(event.target.value);
          setBlankReason(false);
        }}
        placeholder={
          required ? "예: 다른 회원을 잘못 누름" : "예: 본인 전화, 건강 문제"
        }
        maxLength={100}
        aria-invalid={shownReasonError ? true : undefined}
      />
      {shownReasonError && (
        <p className={formStyles.fieldError} role="alert">
          {shownReasonError}
        </p>
      )}
    </div>
  );

  return (
    <dialog
      ref={dialog}
      className={dialogStyles.dialog}
      aria-labelledby={titleId}
      onCancel={(event) => {
        // Esc: 브라우저가 닫기 전에 상태로 닫음
        event.preventDefault();
        if (!busy) {
          onCancel();
        }
      }}
    >
      <h2 id={titleId} className={dialogStyles.title}>
        {TITLES[request.kind]}
      </h2>
      <p className={styles.dialogMember}>
        {request.member.memberName}
        <span className={styles.dialogMemberNo}>{request.member.memberNo}</span>
      </p>

      {request.kind === "withdraw" && (
        <>
          <p className={dialogStyles.body}>
            {request.from}에서 포기로 바꿉니다.
            {request.from === "당첨" && " 빈자리가 하나 생깁니다."}
          </p>
          {reasonField(false)}
        </>
      )}
      {request.kind === "contact" && (
        <p className={dialogStyles.body}>
          대기 {request.waitlistNumber}번
          {request.phoneNumber !== null && ` · ${request.phoneNumber}`}
          <br />
          {openSeats === 0
            ? "지금은 빈자리가 없어 수락을 남길 수 없습니다."
            : `빈자리 ${openSeats}. 수락하면 당첨이 됩니다.`}
        </p>
      )}
      {request.kind === "correct" && (
        <>
          <p className={dialogStyles.body}>
            포기를 잘못 처리했을 때만 씁니다. {request.restoreTo}(으)로 되돌리고
            사유와 함께 기록합니다.
          </p>
          {reasonField(true)}
        </>
      )}

      {formError && (
        <p className={formStyles.formError} role="alert">
          {formError}
        </p>
      )}

      <div className={dialogStyles.actions}>
        <button
          type="button"
          className={dialogStyles.cancel}
          onClick={onCancel}
          disabled={busy}
          autoFocus
        >
          그만두기
        </button>
        {request.kind === "withdraw" && (
          <button
            type="button"
            className={dialogStyles.confirm}
            disabled={busy}
            onClick={() =>
              onChoose({
                kind: "withdraw",
                reason: reason.trim() === "" ? null : reason,
              })
            }
          >
            포기로 바꾸기
          </button>
        )}
        {request.kind === "contact" &&
          CONTACT_BUTTONS.map(({ result, label }) => (
            <button
              key={result}
              type="button"
              className={
                result === "accepted" ? formStyles.submit : formStyles.secondary
              }
              disabled={busy || (result === "accepted" && openSeats === 0)}
              onClick={() => onChoose({ kind: "contact", result })}
            >
              {label}
            </button>
          ))}
        {request.kind === "correct" && (
          <button
            type="button"
            className={formStyles.submit}
            disabled={busy}
            onClick={() => {
              if (reason.trim() === "") {
                setBlankReason(true);
                return;
              }
              onChoose({ kind: "correct", reason });
            }}
          >
            바로잡기
          </button>
        )}
      </div>
    </dialog>
  );
}

const TITLES: Record<ChangeRequest["kind"], string> = {
  withdraw: "포기로 바꿀까요?",
  contact: "연락한 결과를 남깁니다",
  correct: "잘못 처리한 포기를 바로잡을까요?",
};

/** 연락 결과 버튼. 거절은 포기가 되고(BR-W4), 연락 안 됨은 대기 그대로다(BR-W5). */
const CONTACT_BUTTONS: readonly { result: ContactResultCode; label: string }[] =
  [
    { result: "unreachable", label: "연락 안 됨" },
    { result: "declined", label: "거절 (포기)" },
    { result: "accepted", label: "수락 (당첨)" },
  ];
