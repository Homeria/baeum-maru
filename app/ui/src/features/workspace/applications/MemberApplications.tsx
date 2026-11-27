import { useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { HostError } from "../../../api/client";
import type { ApplicationResponse } from "../../../api/generated/ApplicationResponse";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";
import type { SelectionStageCode } from "../../../api/generated/SelectionStageCode";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { Card } from "../../../components/surface/Card";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { StatusPill } from "../../../components/ui/StatusPill";
import { useCan } from "../../gate/auth/useCan";
import { courseName, scheduleSummary } from "../courses/courseFormData";
import { getMember } from "../members/api";
import { deleteApplication, listApplications, setPriorityReasons } from "./api";
import { RULE_LABELS, toggleId } from "./applicationsData";
import styles from "./ApplicationsPage.module.css";
import { DeleteApplicationDialog } from "./DeleteApplicationDialog";
import { PriorityChoices } from "./PriorityChoices";

/** {@link MemberApplications}의 속성. */
export interface MemberApplicationsProps {
  termId: number;
  memberId: number;
  /** 이 학기의 강좌. 신청한 강좌의 이름과 시간을 찾는다. */
  courses: readonly CourseResponse[];
  /** 우선 배정 사유. 없으면 우선 배정 칸을 보여 주지 않는다. */
  reasons: readonly PriorityReasonResponse[];
  /** 강좌 id로 찾는 선발 단계. 마감한 강좌는 우선 배정을 바꿀 수 없다(BR-S13). */
  stages: ReadonlyMap<number, SelectionStageCode>;
}

/** 바꾸기 전에 묻는 중인 우선 배정. 추첨한 강좌라 바꾸면 추첨이 취소된다. */
interface PendingPriority {
  application: ApplicationResponse;
  reasonIds: number[];
}

/**
 * 고른 회원이 이 학기에 낸 신청 카드. 접수한 순서이고, 규칙 예외를 받은 신청은 표시한다.
 *
 * 신청을 받을 수 있는 역할(BR-K3)은 신청마다 지우기 버튼이 있다(BR-D1~D3).
 * 우선 배정을 할 수 있는 역할은 신청마다 사유 체크박스로 붙이고 뗀다(BR-S12, BR-S13).
 * 추첨한 강좌면 바꾸기 전에 추첨이 취소된다고 한 번 묻는다.
 */
export function MemberApplications({
  termId,
  memberId,
  courses,
  reasons,
  stages,
}: MemberApplicationsProps) {
  const member = useQuery({
    queryKey: ["members", "detail", memberId],
    queryFn: () => getMember(memberId),
  });
  const applications = useQuery({
    queryKey: ["applications", termId, memberId],
    queryFn: () => listApplications(termId, memberId),
  });
  const queryClient = useQueryClient();
  const canDelete = useCan("takeApplications");
  const canAssign = useCan("assignPriority");
  const [pendingPriority, setPendingPriority] =
    useState<PendingPriority | null>(null);
  const [deleting, setDeleting] = useState<ApplicationResponse | null>(null);
  const [reasonError, setReasonError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const cancelPriorityRef = useRef<HTMLButtonElement>(null);

  const nameOf = (courseId: number) => {
    const course = courses.find((candidate) => candidate.id === courseId);
    return course ? courseName(course) : "지워진 강좌";
  };

  const remove = useMutation({
    mutationFn: ({ id, reason }: { id: number; reason: string | null }) =>
      deleteApplication(id, { reason }),
    onSuccess: () => {
      setNotice(`지웠습니다: ${deleting ? nameOf(deleting.courseId) : ""}`);
      setDeleting(null);
      setReasonError(null);
      void queryClient.invalidateQueries({
        queryKey: ["applications", termId, memberId],
      });
      void queryClient.invalidateQueries({
        queryKey: ["applicationCounts", termId],
      });
    },
    onError: (failure) => {
      if (failure instanceof HostError && failure.fields.length > 0) {
        // 사유가 받아들여지지 않음 (예: 너무 김). 창을 열어 둔다
        setReasonError(failure.fields[0].message);
        return;
      }
      setDeleting(null);
      setError(failure.message);
      // 그 사이 지워졌거나 선발됐을 수 있어 새로 받음
      void queryClient.invalidateQueries({
        queryKey: ["applications", termId, memberId],
      });
    },
  });

  const changePriority = useMutation({
    mutationFn: ({ application, reasonIds }: PendingPriority) =>
      setPriorityReasons(application.id, reasonIds),
    onSuccess: (result, { application }) => {
      const name = nameOf(application.courseId);
      setNotice(
        result.lotteryCancelled
          ? `${name}: 우선 배정을 바꿔 추첨이 취소되었습니다. 추첨 탭에서 다시 추첨해 주세요`
          : `${name}: 우선 배정을 바꿨습니다`,
      );
      setPendingPriority(null);
      void queryClient.invalidateQueries({
        queryKey: ["applications", termId, memberId],
      });
      if (result.lotteryCancelled) {
        void queryClient.invalidateQueries({
          queryKey: ["selections", termId],
        });
        void queryClient.invalidateQueries({
          queryKey: ["lottery", application.courseId],
        });
      }
    },
    onError: (failure) => {
      setPendingPriority(null);
      setError(failure.message);
      // 그 사이 마감했거나 사유가 지워졌을 수 있어 새로 받음
      void queryClient.invalidateQueries({
        queryKey: ["applications", termId, memberId],
      });
      void queryClient.invalidateQueries({ queryKey: ["selections", termId] });
    },
  });

  const togglePriority = (
    application: ApplicationResponse,
    reasonId: number,
  ) => {
    setNotice(null);
    setError(null);
    const next = {
      application,
      reasonIds: [...toggleId(application.priorityReasonIds, reasonId)],
    };
    if (stages.get(application.courseId) === "drawn") {
      setPendingPriority(next);
    } else {
      changePriority.mutate(next);
    }
  };

  const reasonNames = (ids: readonly number[]) =>
    reasons
      .filter((reason) => ids.includes(reason.id))
      .map((reason) => reason.name)
      .join(", ");

  const title = member.data
    ? `${member.data.name}(${member.data.memberNo})의 신청`
    : "고른 회원의 신청";

  if (applications.isPending) {
    return (
      <Card title={title}>
        <NoticeBanner message="신청을 불러오는 중입니다…" />
      </Card>
    );
  }
  if (applications.isError) {
    return (
      <Card title={title}>
        <NoticeBanner
          tone="error"
          message={applications.error.message}
          actionLabel="다시 해 보기"
          onAction={() => void applications.refetch()}
        />
      </Card>
    );
  }

  const items = applications.data.items;

  return (
    <Card title={`${title} · ${items.length}건`}>
      <div className={styles.applyDock}>
        {notice && <NoticeBanner message={notice} />}
        {error && <NoticeBanner tone="error" message={error} />}

        {items.length === 0 ? (
          <NoticeBanner message="이 학기에 낸 신청이 없습니다." />
        ) : (
          <ul className={styles.applicationList}>
            {items.map((application) => {
              const course = courses.find(
                (candidate) => candidate.id === application.courseId,
              );
              const name = nameOf(application.courseId);
              return (
                <li key={application.id} className={styles.application}>
                  <div className={styles.applicationHead}>
                    <div className={styles.applicationMeta}>
                      <span className={styles.courseName}>{name}</span>
                      {course && (
                        <span className={styles.muted}>
                          {scheduleSummary(course.schedules, false).times}
                        </span>
                      )}
                      {application.overriddenRules.length > 0 && (
                        <StatusPill tone="neutral">
                          예외:{" "}
                          {application.overriddenRules
                            .map((rule) => RULE_LABELS[rule])
                            .join(", ")}
                        </StatusPill>
                      )}
                    </div>
                    {canDelete && (
                      <Button
                        size="sm"
                        variant="danger"
                        onClick={() => {
                          setNotice(null);
                          setError(null);
                          setReasonError(null);
                          setDeleting(application);
                        }}
                      >
                        지우기
                      </Button>
                    )}
                  </div>

                  {canAssign && reasons.length > 0 ? (
                    <PriorityChoices
                      reasons={reasons}
                      checked={application.priorityReasonIds}
                      courseName={name}
                      disabled={
                        changePriority.isPending ||
                        stages.get(application.courseId) === "closed"
                      }
                      onToggle={(reasonId) =>
                        togglePriority(application, reasonId)
                      }
                    />
                  ) : (
                    application.priorityReasonIds.length > 0 && (
                      <div>
                        <StatusPill tone="primary">
                          우선 배정:{" "}
                          {reasonNames(application.priorityReasonIds)}
                        </StatusPill>
                      </div>
                    )
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>

      <ModalDialog
        open={pendingPriority !== null}
        size="sm"
        title={`${pendingPriority ? nameOf(pendingPriority.application.courseId) : ""}의 우선 배정을 바꿀까요?`}
        busy={changePriority.isPending}
        initialFocusRef={cancelPriorityRef}
        onClose={() => setPendingPriority(null)}
        footer={
          <>
            <Button
              ref={cancelPriorityRef}
              variant="secondary"
              disabled={changePriority.isPending}
              onClick={() => setPendingPriority(null)}
            >
              그만두기
            </Button>
            <Button
              variant="dangerSolid"
              busy={changePriority.isPending}
              busyLabel="바꾸는 중…"
              onClick={() => {
                if (pendingPriority) {
                  changePriority.mutate(pendingPriority);
                }
              }}
            >
              바꾸기
            </Button>
          </>
        }
      >
        <NoticeBanner
          tone="error"
          message="추첨한 강좌입니다. 바꾸면 이 강좌의 추첨이 취소되고 추첨 전으로 돌아갑니다."
        />
      </ModalDialog>

      {deleting && (
        <DeleteApplicationDialog
          courseName={nameOf(deleting.courseId)}
          deleting={remove.isPending}
          reasonError={reasonError}
          onConfirm={(reason) => remove.mutate({ id: deleting.id, reason })}
          onCancel={() => setDeleting(null)}
        />
      )}
    </Card>
  );
}
