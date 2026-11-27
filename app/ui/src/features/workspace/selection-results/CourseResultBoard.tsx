import { useId, useState, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ResultMemberResponse } from "../../../api/generated/ResultMemberResponse";
import {
  correctWithdrawal,
  getCourseResult,
  recordContact,
  withdraw,
} from "./api";
import formStyles from "../../../components/form/form.module.css";
import {
  NO_FORM_ERRORS,
  formErrorsFrom,
  type FormErrors,
} from "../../../components/form/formErrors";
import {
  ChangeDialog,
  type ChangeChoice,
  type ChangeRequest,
} from "./ChangeDialog";
import {
  CONTACT_LABELS,
  WITHDRAWN_FROM_LABELS,
  matchesQuery,
  memberLabel,
  openSeatsLabel,
  restoreLabel,
} from "./resultsData";
import styles from "./SelectionResultsPage.module.css";

/** {@link CourseResultBoard}의 속성. */
export interface CourseResultBoardProps {
  courseId: number;
  /** 화면 위에 보여 줄 강좌 이름. 예: 요가 초급 */
  courseName: string;
}

/**
 * 마감한 강좌 하나의 선발 결과. 당첨자, 대기자, 포기 세 칸이다.
 *
 * 대기자 칸은 대기 순번 순서이고, 빈자리가 있으면 연락할 차례인 대기자를 옅은 바탕으로
 * 강조한다(BR-W1). 연락처는 연락할 수 있는 역할에게만 온다(BR-W2).
 * 이름 검색 칸은 전화 온 회원을 찾아 그 줄에 테두리를 두른다.
 *
 * 관리자·직원은 줄마다 포기와 연락 결과 버튼이, 관리자는 포기 칸에 바로잡기 버튼이 있다.
 */
export function CourseResultBoard({
  courseId,
  courseName,
}: CourseResultBoardProps) {
  const result = useQuery({
    queryKey: ["selection-result", courseId],
    queryFn: () => getCourseResult(courseId),
  });
  const [query, setQuery] = useState("");
  const searchId = useId();
  const queryClient = useQueryClient();
  const [asking, setAsking] = useState<ChangeRequest | null>(null);
  const [errors, setErrors] = useState<FormErrors<"reason">>(NO_FORM_ERRORS);
  const [notice, setNotice] = useState<string | null>(null);

  const open = (request: ChangeRequest) => {
    setErrors(NO_FORM_ERRORS);
    setAsking(request);
  };
  const close = () => {
    setErrors(NO_FORM_ERRORS);
    setAsking(null);
  };

  const change = useMutation({
    mutationFn: ({ member, choice }: Change) =>
      send(member.applicationId, choice),
    onSuccess: (_, { member, choice }) => {
      setNotice(`${member.memberName}${DONE[doneKey(choice)]}`);
      close();
    },
    onError: (failure) => {
      setErrors(formErrorsFrom(failure, ["reason"] as const));
    },
    onSettled: () => {
      // 다른 직원이 먼저 바꿨을 수도 있어 실패해도 새로 받는다
      void queryClient.invalidateQueries({
        queryKey: ["selection-result", courseId],
      });
      void queryClient.invalidateQueries({ queryKey: ["closed-courses"] });
      // 신청 탭의 신청 상태와 인원, 추첨 탭의 결과가 바뀐다
      void queryClient.invalidateQueries({ queryKey: ["applications"] });
      void queryClient.invalidateQueries({ queryKey: ["applicationCounts"] });
      void queryClient.invalidateQueries({ queryKey: ["lottery", courseId] });
    },
  });

  if (result.isPending) {
    return <p className={formStyles.loading}>결과를 불러오는 중입니다…</p>;
  }
  if (result.isError) {
    return (
      <p className={formStyles.formError} role="alert">
        {result.error.message}
      </p>
    );
  }

  const data = result.data;
  const matchClass = (matched: boolean) => (matched ? styles.match : "");
  const button = (label: string, request: ChangeRequest) => (
    <button
      type="button"
      className={styles.entryButton}
      onClick={() => open(request)}
    >
      {label}
    </button>
  );

  return (
    <section className={styles.board} aria-label={`${courseName} 선발 결과`}>
      <header className={styles.boardHeader}>
        <h2 className={styles.courseTitle}>{courseName}</h2>
        <span className={styles.meta}>
          정원 {data.capacity}명 · {openSeatsLabel(data.openSeats)}
        </span>
        <label htmlFor={searchId} className={formStyles.label}>
          회원 찾기
        </label>
        <input
          id={searchId}
          className={`${formStyles.input} ${styles.search}`}
          type="search"
          autoComplete="off"
          placeholder="이름이나 회원번호"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
      </header>
      {notice && (
        <p className={`${formStyles.notice} ${styles.notice}`} role="status">
          {notice}
        </p>
      )}

      <div className={styles.columns}>
        <Column title="당첨자" count={data.selected.length}>
          {data.selected.map((entry) => (
            <li
              key={entry.member.applicationId}
              className={`${styles.entry} ${matchClass(matchesQuery(entry.member, query))}`}
            >
              <span className={styles.name}>{memberLabel(entry.member)}</span>
              {entry.guaranteed && <span className={styles.tag}>우선</span>}
              {entry.waitlistNumber !== null && (
                <span className={styles.small}>
                  대기 {entry.waitlistNumber}번에서 승격
                </span>
              )}
              {data.canRecord && (
                <span className={styles.entryActions}>
                  {button("포기", {
                    kind: "withdraw",
                    member: entry.member,
                    from: "당첨",
                  })}
                </span>
              )}
            </li>
          ))}
        </Column>

        <Column title="대기자" count={data.waitlisted.length}>
          {data.waitlisted.map((entry) => (
            <li
              key={entry.member.applicationId}
              className={`${styles.entry} ${entry.contactTurn ? styles.turn : ""} ${matchClass(matchesQuery(entry.member, query))}`}
            >
              <span className={styles.small}>{entry.waitlistNumber}번</span>
              <span className={styles.name}>{memberLabel(entry.member)}</span>
              {entry.phoneNumber !== null && <span>{entry.phoneNumber}</span>}
              {entry.contactTurn && (
                <span className={styles.tag}>연락할 차례</span>
              )}
              {entry.lastContact !== null && (
                <span className={styles.small}>
                  {CONTACT_LABELS[entry.lastContact]}
                </span>
              )}
              {data.canRecord && (
                <span className={styles.entryActions}>
                  {button("연락 결과", {
                    kind: "contact",
                    member: entry.member,
                    phoneNumber: entry.phoneNumber,
                    waitlistNumber: entry.waitlistNumber,
                  })}
                  {button("포기", {
                    kind: "withdraw",
                    member: entry.member,
                    from: "대기",
                  })}
                </span>
              )}
            </li>
          ))}
        </Column>

        <Column title="포기" count={data.withdrawn.length}>
          {data.withdrawn.map((entry) => (
            <li
              key={entry.member.applicationId}
              className={`${styles.entry} ${matchClass(matchesQuery(entry.member, query))}`}
            >
              <span className={styles.name}>{memberLabel(entry.member)}</span>
              <span className={styles.small}>
                {WITHDRAWN_FROM_LABELS[entry.withdrawnFrom]}
                {entry.waitlistNumber !== null &&
                  ` (대기 ${entry.waitlistNumber}번)`}
              </span>
              {data.canCorrect && (
                <span className={styles.entryActions}>
                  {button("바로잡기", {
                    kind: "correct",
                    member: entry.member,
                    restoreTo: restoreLabel(entry),
                  })}
                </span>
              )}
            </li>
          ))}
        </Column>
      </div>

      {asking && (
        <ChangeDialog
          request={asking}
          openSeats={data.openSeats}
          busy={change.isPending}
          reasonError={errors.fields.reason ?? null}
          formError={errors.form}
          onChoose={(choice) => {
            setNotice(null);
            change.mutate({ member: asking.member, choice });
          }}
          onCancel={close}
        />
      )}
    </section>
  );
}

/** 보낼 바뀜 하나. 알림에 이름을 쓰려고 회원을 함께 둔다. */
interface Change {
  member: ResultMemberResponse;
  choice: ChangeChoice;
}

/** 창에서 고른 것을 호스트에 보낸다. */
function send(applicationId: number, choice: ChangeChoice): Promise<void> {
  switch (choice.kind) {
    case "withdraw":
      return withdraw(applicationId, choice.reason);
    case "contact":
      return recordContact(applicationId, choice.result);
    case "correct":
      return correctWithdrawal(applicationId, choice.reason);
  }
}

type DoneKey = "withdraw" | "accepted" | "declined" | "unreachable" | "correct";

function doneKey(choice: ChangeChoice): DoneKey {
  return choice.kind === "contact" ? choice.result : choice.kind;
}

/** 저장한 뒤 알림. 이름 뒤에 붙인다. */
const DONE: Record<DoneKey, string> = {
  withdraw: " 님을 포기로 바꿨습니다.",
  accepted: " 님이 수락해 당첨으로 바꿨습니다.",
  declined: " 님이 거절해 포기로 바꿨습니다.",
  unreachable: " 님은 연락 안 됨으로 남겼습니다. 대기 그대로입니다.",
  correct: " 님의 포기를 바로잡았습니다.",
};

/** {@link Column}의 속성. */
interface ColumnProps {
  title: string;
  count: number;
  children: ReactNode;
}

/** 세 칸 가운데 하나. 제목에 인원을 붙인다. */
function Column({ title, count, children }: ColumnProps) {
  return (
    <section className={styles.column} aria-label={title}>
      <h3 className={styles.columnTitle}>
        {title} ({count}명)
      </h3>
      <ul className={styles.list}>{children}</ul>
    </section>
  );
}
