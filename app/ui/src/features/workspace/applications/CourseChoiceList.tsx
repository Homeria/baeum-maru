import type { AdditionalOutcomeResponse } from "../../../api/generated/AdditionalOutcomeResponse";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { SelectionStageCode } from "../../../api/generated/SelectionStageCode";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { StatusPill } from "../../../components/ui/StatusPill";
import { courseName, scheduleSummary } from "../courses/courseFormData";
import { STAGE_LABELS, countLabel } from "../courses/courseStatusData";
import { additionalLabel } from "./applicationsData";
import styles from "./ApplicationsPage.module.css";

/** {@link CourseChoiceList}의 속성. */
export interface CourseChoiceListProps {
  courses: readonly CourseResponse[];
  /** 강좌 id로 찾는 신청 인원. 없으면 0명. */
  counts: ReadonlyMap<number, number>;
  /** 강의실 이름에 건물을 붙일지. 건물이 여럿일 때만. */
  showBuilding: boolean;
  /** 강좌를 고를 수 있는지. 회원을 고르고 신청을 받을 수 있는 역할일 때. */
  selectable: boolean;
  /** 고른 강좌. */
  chosen: readonly number[];
  /** 고른 회원이 이미 신청한 강좌. 고를 수 없다. */
  appliedIds: ReadonlySet<number>;
  /**
   * 강좌 id로 찾는 선발 단계. 추첨했지만 마감 전인 강좌는 새 신청을 받지 않아 고를 수 없다(BR-L3).
   * 마감한 강좌는 추가 신청으로 고른다(BR-X1).
   */
  stages: ReadonlyMap<number, SelectionStageCode>;
  /** 마감한 강좌에 지금 추가 신청을 받으면 어떻게 되는지(BR-X2). 강좌 id로 찾는다. */
  additional: ReadonlyMap<number, AdditionalOutcomeResponse>;
  /** 학기 종료일이 지나 추가 신청을 받지 않는다(BR-X1). */
  termEnded: boolean;
  onToggle: (id: number) => void;
}

/**
 * 이 학기의 강좌와 신청 인원/정원 표.
 *
 * 여러 요일·시간대 수업 시간은 셀 안에서 줄바꿈하고, 추가 신청 가능 강좌는 결과를 꼬리표로 보여 준다.
 */
export function CourseChoiceList({
  courses,
  counts,
  showBuilding,
  selectable,
  chosen,
  appliedIds,
  stages,
  additional,
  termEnded,
  onToggle,
}: CourseChoiceListProps) {
  const rowState = (course: CourseResponse) => {
    const applied = appliedIds.has(course.id);
    const stage = stages.get(course.id) ?? "not_drawn";
    // 마감한 강좌는 추가 신청으로 받는다. 미리 보기를 아직 못 받았으면 기다린다
    const preview =
      stage === "closed" && !termEnded ? additional.get(course.id) : undefined;
    const open = stage === "not_drawn" || preview !== undefined;
    const locked = applied || !open;
    return { applied, stage, preview, locked };
  };

  const columns: DataTableColumn<CourseResponse>[] = [
    {
      key: "select",
      header: "",
      width: "92px",
      align: "center",
      isAction: true,
      render: (course) => {
        const { applied, stage, preview, locked } = rowState(course);
        const checked = chosen.includes(course.id);
        const previewId = `additional-preview-${course.id}`;
        if (applied) {
          return <StatusPill tone="primary">신청함</StatusPill>;
        }
        if (locked) {
          return (
            <StatusPill tone="neutral">
              {stage === "closed" && termEnded
                ? "학기 끝남"
                : STAGE_LABELS[stage]}
            </StatusPill>
          );
        }
        return (
          <input
            type="checkbox"
            className={styles.courseCheck}
            checked={checked}
            disabled={!selectable}
            aria-label={`${courseName(course)} 고르기`}
            aria-describedby={preview ? previewId : undefined}
            onChange={() => onToggle(course.id)}
          />
        );
      },
    },
    {
      key: "course",
      header: "강좌",
      isRowHeader: true,
      render: (course) => courseName(course),
    },
    {
      key: "schedules",
      header: "수업 시간",
      render: (course) => {
        const summary = scheduleSummary(course.schedules, showBuilding);
        const lines = summary.times
          .split(", ")
          .filter((line) => line.length > 0);
        return (
          <div className={styles.scheduleLines}>
            {lines.map((line) => (
              <span key={line} className={styles.scheduleLine}>
                {line}
              </span>
            ))}
          </div>
        );
      },
    },
    {
      key: "room",
      header: "강의실",
      width: "120px",
      render: (course) => scheduleSummary(course.schedules, showBuilding).room,
    },
    {
      key: "capacity",
      header: "신청/정원",
      width: "160px",
      align: "right",
      render: (course) => {
        const { applied, preview } = rowState(course);
        const previewId = `additional-preview-${course.id}`;
        return (
          <div className={styles.capacityCell}>
            <span>
              {countLabel(counts.get(course.id) ?? 0, course.capacity)}
            </span>
            {preview && !applied && (
              <span id={previewId}>
                <StatusPill
                  tone={preview.kind === "selected" ? "ok" : "neutral"}
                >
                  추가 신청: {additionalLabel(preview)}
                </StatusPill>
              </span>
            )}
          </div>
        );
      },
    },
  ];

  return (
    <Card
      title="신청할 강좌"
      subtitle={`모두 ${courses.length}개`}
      padding="none"
    >
      <DataTable
        caption="신청할 강좌"
        columns={columns}
        rows={courses}
        getRowKey={(course) => course.id}
        onRowClick={
          selectable
            ? (course) => {
                if (!rowState(course).locked) {
                  onToggle(course.id);
                }
              }
            : undefined
        }
        emptyMessage="이 학기에 등록된 강좌가 없습니다. 강좌 탭에서 먼저 등록해 주세요."
      />
    </Card>
  );
}
