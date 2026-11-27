import { useState, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ResultMemberResponse } from "../../../api/generated/ResultMemberResponse";
import {
  NO_FORM_ERRORS,
  formErrorsFrom,
  type FormErrors,
} from "../../../components/form/formErrors";
import { Card } from "../../../components/surface/Card";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { SearchInput } from "../../../components/ui/SearchInput";
import { StatusPill } from "../../../components/ui/StatusPill";
import {
  correctWithdrawal,
  getCourseResult,
  recordContact,
  withdraw,
} from "./api";
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
  /** 화면 위에 보여 줄 강좌 이름. 예: 서예 초급 1반 */
  courseName: string;
}

/**
 * 마감한 강좌 하나의 선발 결과 보드. 당첨자(파란 머리줄), 대기자(가장 넓음), 포기 세 칸이다.
 *
 * 대기자 칸은 대기 순번 순서이고, 빈자리가 있으면 연락할 차례인 대기자를 노란 바탕과
 * "연락할 차례" 꼬리표로 강조한다(BR-W1). 연락처는 연락할 수 있는 역할에게만 온다(BR-W2).
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
    return <NoticeBanner message="결과를 불러오는 중입니다…" />;
  }
  if (result.isError) {
    return (
      <NoticeBanner
        tone="error"
        message={result.error.message}
        actionLabel="다시 해 보기"
        onAction={() => void result.refetch()}
      />
    );
  }

  const data = result.data;
  const matchClass = (matched: boolean) => (matched ? styles.match : "");

  return (
    <section className={styles.board} aria-label={`${courseName} 선발 결과`}>
      <Card>
        <div className={styles.boardHeader}>
          <div className={styles.boardTitleGroup}>
            <h2 className={styles.courseTitle}>{courseName}</h2>
            <span className={styles.meta}>정원 {data.capacity}명</span>
            <StatusPill tone={data.openSeats > 0 ? "primary" : "neutral"}>
              {openSeatsLabel(data.openSeats)}
            </StatusPill>
          </div>
          <SearchInput
            label="회원 찾기"
            hideLabel
            placeholder="이름이나 회원번호로 찾기"
            value={query}
            onChange={setQuery}
            className={styles.searchInput}
          />
        </div>
      </Card>

      {notice && <NoticeBanner message={notice} />}

      <div className={styles.columns}>
        <Column
          title="당첨자"
          count={data.selected.length}
          highlightHeader
          emptyText="당첨자가 없습니다."
        >
          {data.selected.map((entry) => (
            <li
              key={entry.member.applicationId}
              className={[
                styles.entry,
                matchClass(matchesQuery(entry.member, query)),
              ]
                .filter(Boolean)
                .join(" ")}
            >
              <div className={styles.entryMain}>
                <span className={styles.name}>{memberLabel(entry.member)}</span>
                {entry.guaranteed && (
                  <StatusPill tone="primary">우선</StatusPill>
                )}
                {entry.waitlistNumber !== null && (
                  <StatusPill tone="ok">
                    대기 {entry.waitlistNumber}번에서 승격
                  </StatusPill>
                )}
              </div>
              {data.canRecord && (
                <div className={styles.entryActionsEnd}>
                  <Button
                    size="sm"
                    variant="danger"
                    onClick={() =>
                      open({
                        kind: "withdraw",
                        member: entry.member,
                        from: "당첨",
                      })
                    }
                  >
                    포기
                  </Button>
                </div>
              )}
            </li>
          ))}
        </Column>

        <Column
          title="대기자"
          count={data.waitlisted.length}
          emptyText="대기자가 없습니다."
        >
          {data.waitlisted.map((entry) => (
            <li
              key={entry.member.applicationId}
              className={[
                styles.entry,
                entry.contactTurn ? styles.turn : "",
                matchClass(matchesQuery(entry.member, query)),
              ]
                .filter(Boolean)
                .join(" ")}
            >
              <div className={styles.entryMain}>
                <span className={styles.waitNumber}>
                  {entry.waitlistNumber}번
                </span>
                <span className={styles.name}>{memberLabel(entry.member)}</span>
                {entry.phoneNumber !== null && (
                  <span className={styles.phone}>{entry.phoneNumber}</span>
                )}
                {entry.contactTurn && (
                  <StatusPill tone="turn">연락할 차례</StatusPill>
                )}
                {entry.lastContact !== null && (
                  <StatusPill tone="neutral">
                    {CONTACT_LABELS[entry.lastContact]}
                  </StatusPill>
                )}
              </div>
              {data.canRecord && (
                <div className={styles.entryActions}>
                  <Button
                    size="sm"
                    variant={entry.contactTurn ? "primary" : "secondary"}
                    onClick={() =>
                      open({
                        kind: "contact",
                        member: entry.member,
                        phoneNumber: entry.phoneNumber,
                        waitlistNumber: entry.waitlistNumber,
                      })
                    }
                  >
                    연락 결과
                  </Button>
                  <Button
                    size="sm"
                    variant="danger"
                    onClick={() =>
                      open({
                        kind: "withdraw",
                        member: entry.member,
                        from: "대기",
                      })
                    }
                  >
                    포기
                  </Button>
                </div>
              )}
            </li>
          ))}
        </Column>

        <Column
          title="포기"
          count={data.withdrawn.length}
          emptyText="포기한 회원이 없습니다."
        >
          {data.withdrawn.map((entry) => (
            <li
              key={entry.member.applicationId}
              className={[
                styles.entry,
                matchClass(matchesQuery(entry.member, query)),
              ]
                .filter(Boolean)
                .join(" ")}
            >
              <div className={styles.entryMain}>
                <span className={styles.name}>{memberLabel(entry.member)}</span>
                <span className={styles.small}>
                  {WITHDRAWN_FROM_LABELS[entry.withdrawnFrom]}
                  {entry.waitlistNumber !== null &&
                    ` (대기 ${entry.waitlistNumber}번)`}
                </span>
              </div>
              {data.canCorrect && (
                <div className={styles.entryActionsEnd}>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() =>
                      open({
                        kind: "correct",
                        member: entry.member,
                        restoreTo: restoreLabel(entry),
                      })
                    }
                  >
                    바로잡기
                  </Button>
                </div>
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
  highlightHeader?: boolean;
  emptyText: string;
  children: ReactNode;
}

/** 세 칸 가운데 하나. 제목과 인원을 보여 준다. */
function Column({
  title,
  count,
  highlightHeader = false,
  emptyText,
  children,
}: ColumnProps) {
  return (
    <section className={styles.column} aria-label={title}>
      <div
        className={[
          styles.columnHeader,
          highlightHeader ? styles.columnWinnerHeader : "",
        ]
          .filter(Boolean)
          .join(" ")}
      >
        <h3 className={styles.columnTitle}>{title}</h3>
        <span className={styles.columnCount}>{count}명</span>
      </div>
      {count === 0 ? (
        <p className={styles.emptyColumn}>{emptyText}</p>
      ) : (
        <ul className={styles.list}>{children}</ul>
      )}
    </section>
  );
}
