import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  SplitAuthLayout,
  type SplitAuthStep,
} from "../../../components/shell/SplitAuthLayout";
import { Card } from "../../../components/surface/Card";
import { Button } from "../../../components/ui/Button";
import { listPriorityReasons } from "../../workspace/priority-reasons/api";
import { listRooms } from "../../workspace/rooms/api";
import { listSubjects } from "../../workspace/subjects/api";
import { RoomsStep } from "./RoomsStep";
import { SubjectsAndReasonsStep } from "./SubjectsAndReasonsStep";
import styles from "./SetupBasicsPage.module.css";

type BasicsStep = "rooms" | "subjects";

/** {@link SetupBasicsPage}의 속성. */
export interface SetupBasicsPageProps {
  /** 기초 정보 미리 넣기를 끝내거나 나중에 하기를 눌렀다. */
  onDone: () => void;
}

/**
 * 호스트 첫 실행 3단계: 기초 정보 미리 넣기(강의실 → 과목·우선 배정 사유).
 *
 * 기존 `/api/rooms`, `/api/subjects`, `/api/priority-reasons` API로 즉시 저장·삭제한다.
 * 선택 단계라서 어느 화면에서든 "나중에 하기"로 메인 화면에 바로 들어갈 수 있다.
 */
export function SetupBasicsPage({ onDone }: SetupBasicsPageProps) {
  const [step, setStep] = useState<BasicsStep>("rooms");
  const rooms = useQuery({ queryKey: ["rooms"], queryFn: listRooms });
  const subjects = useQuery({ queryKey: ["subjects"], queryFn: listSubjects });
  const reasons = useQuery({
    queryKey: ["priority-reasons"],
    queryFn: listPriorityReasons,
  });

  const roomItems = rooms.data?.items ?? [];
  const subjectItems = subjects.data?.items ?? [];
  const reasonItems = reasons.data?.items ?? [];

  const steps: readonly SplitAuthStep[] = [
    {
      label: "관리자 계정 만들기",
      description: "관리자 계정을 만들었습니다.",
      status: "done",
    },
    {
      label: "복구 코드 보관",
      description: "복구 코드를 확인했습니다.",
      status: "done",
    },
    {
      label: "기초 정보 미리 넣기",
      description:
        step === "rooms"
          ? "강의실(건물·층) → 과목·우선 배정 사유 순으로 넣습니다."
          : `강의실 ${roomItems.length}개 등록 · 과목과 우선 배정 사유를 넣습니다.`,
      status: "current",
    },
  ];

  const tip =
    step === "rooms" ? (
      <div className={styles.panelTip}>
        <p className={styles.panelTipTitle}>지금 다 넣지 않아도 됩니다</p>
        <p className={styles.panelTipText}>
          건너뛰거나 일부만 넣고, 나중에 [기초 정보] 탭에서 언제든 더하고 고칠
          수 있습니다.
        </p>
      </div>
    ) : (
      <div className={styles.panelTip}>
        <p className={styles.panelTipTitle}>강좌는 시작한 뒤 엽니다</p>
        <p className={styles.panelTipText}>
          학기마다 바뀌는 정원·수업 시간은 시작 후 [강좌] 탭에서 편하게
          등록합니다.
        </p>
      </div>
    );

  return (
    <SplitAuthLayout
      panelBadge="선택 단계"
      panelTitle={
        step === "rooms"
          ? "기관의 기초 정보를 미리 넣어 둡니다"
          : "과목과 우선 배정 사유를 정합니다"
      }
      panelDescription={
        step === "rooms"
          ? "강의실과 과목처럼 학기가 바뀌어도 그대로 쓰는 정보를 먼저 등록하면 바로 강좌를 열 수 있습니다."
          : "자주 여는 과목과 우선 배정 사유(예: 지난 학기 반장)를 넣어 두면 첫 학기 강좌를 바로 만들 수 있습니다."
      }
      steps={steps}
      panelFooter={tip}
      frameless
    >
      <div className={styles.subStepBar}>
        <nav aria-label="기초 정보 단계" className={styles.subStepNav}>
          <button
            type="button"
            aria-current={step === "rooms" ? "step" : undefined}
            className={[
              styles.subStepPill,
              step === "rooms"
                ? styles.subStepPillActive
                : styles.subStepPillDone,
            ].join(" ")}
            onClick={() => setStep("rooms")}
          >
            1. 강의실 · {roomItems.length}개
          </button>
          <span className={styles.subStepArrow} aria-hidden="true">
            →
          </span>
          <button
            type="button"
            aria-current={step === "subjects" ? "step" : undefined}
            className={[
              styles.subStepPill,
              step === "subjects" ? styles.subStepPillActive : "",
            ]
              .filter(Boolean)
              .join(" ")}
            onClick={() => setStep("subjects")}
          >
            2. 과목 · 우선 배정 사유
          </button>
        </nav>
        <p className={styles.subStepHint}>
          {step === "rooms"
            ? "건물과 층은 마지막 값이 유지되어 연속 입력하기 쉽습니다"
            : "언제든 [기초 정보] 탭에서 바꿀 수 있습니다"}
        </p>
      </div>

      {step === "rooms" ? (
        <RoomsStep
          rooms={roomItems}
          loading={rooms.isPending}
          error={rooms.error?.message ?? null}
          onRetry={() => void rooms.refetch()}
        />
      ) : (
        <SubjectsAndReasonsStep
          subjects={subjectItems}
          subjectsLoading={subjects.isPending}
          subjectsError={subjects.error?.message ?? null}
          onRetrySubjects={() => void subjects.refetch()}
          reasons={reasonItems}
          reasonsLoading={reasons.isPending}
          reasonsError={reasons.error?.message ?? null}
          onRetryReasons={() => void reasons.refetch()}
        />
      )}

      <Card>
        <div className={styles.footerBar}>
          {step === "rooms" ? (
            <>
              <Button variant="secondary" onClick={onDone}>
                나중에 하기
              </Button>
              <Button variant="primary" onClick={() => setStep("subjects")}>
                다음: 과목 · 우선 배정
              </Button>
            </>
          ) : (
            <>
              <Button variant="secondary" onClick={() => setStep("rooms")}>
                이전: 강의실
              </Button>
              <div className={styles.footerRight}>
                <Button variant="secondary" onClick={onDone}>
                  나중에 하기
                </Button>
                <Button variant="primary" onClick={onDone}>
                  설정 마치고 시작하기
                </Button>
              </div>
            </>
          )}
        </div>
      </Card>
    </SplitAuthLayout>
  );
}
