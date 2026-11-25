import type { AdditionalOutcomeResponse } from "../../api/generated/AdditionalOutcomeResponse";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import type { SelectionStageCode } from "../../api/generated/SelectionStageCode";
import listStyles from "../../components/layout/listPage.module.css";
import tableStyles from "../../components/table/table.module.css";
import { courseName, scheduleSummary } from "../courses/courseFormData";
import { STAGE_LABELS } from "../selection/selectionData";
import { additionalLabel, countLabel } from "./applicationsData";
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

/** 이 학기의 강좌와 신청 인원/정원. 맨 앞의 칸으로 신청할 강좌를 고른다. */
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
  return (
    <section className={listStyles.list} aria-label="강좌">
      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table} aria-label="신청할 강좌">
          <thead>
            <tr>
              <th scope="col">
                <span className={styles.visuallyHidden}>고르기</span>
              </th>
              <th scope="col">과목</th>
              <th scope="col">수준</th>
              <th scope="col">반</th>
              <th scope="col">수업 시간</th>
              <th scope="col">강의실</th>
              <th scope="col">신청/정원</th>
            </tr>
          </thead>
          <tbody>
            {courses.map((course) => {
              const summary = scheduleSummary(course.schedules, showBuilding);
              const applied = appliedIds.has(course.id);
              const stage = stages.get(course.id) ?? "not_drawn";
              // 마감한 강좌는 추가 신청으로 받는다. 미리 보기를 아직 못 받았으면 기다린다
              const preview =
                stage === "closed" && !termEnded
                  ? additional.get(course.id)
                  : undefined;
              const open = stage === "not_drawn" || preview !== undefined;
              // 신청했거나, 추첨하고 마감 전이거나, 학기가 끝난 마감 강좌는 고를 수 없다
              const locked = applied || !open;
              const previewId = `additional-preview-${course.id}`;
              const checked = chosen.includes(course.id);
              return (
                <tr
                  key={course.id}
                  className={
                    selectable && !locked ? tableStyles.row : undefined
                  }
                  aria-current={checked ? "true" : undefined}
                  onClick={() => {
                    if (selectable && !locked) {
                      onToggle(course.id);
                    }
                  }}
                >
                  <td>
                    {applied ? (
                      <span className={styles.muted}>신청함</span>
                    ) : locked ? (
                      <span className={styles.muted}>
                        {stage === "closed" && termEnded
                          ? "학기 끝남"
                          : STAGE_LABELS[stage]}
                      </span>
                    ) : (
                      <input
                        type="checkbox"
                        checked={checked}
                        disabled={!selectable}
                        aria-label={`${courseName(course)} 고르기`}
                        aria-describedby={preview ? previewId : undefined}
                        // 줄을 누르면 고르므로 칸 자체의 누름은 줄에 맡김
                        onClick={(event) => event.stopPropagation()}
                        onChange={() => onToggle(course.id)}
                      />
                    )}
                  </td>
                  <td>{course.subjectName}</td>
                  <td>{course.level ?? ""}</td>
                  <td>{course.section ?? ""}</td>
                  <td>{summary.times}</td>
                  <td>{summary.room}</td>
                  <td>
                    {countLabel(counts.get(course.id) ?? 0, course.capacity)}
                    {preview && !applied && (
                      <span id={previewId} className={styles.additionalPreview}>
                        추가 신청: {additionalLabel(preview)}
                      </span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {courses.length === 0 && (
          <p className={tableStyles.message}>
            이 학기에 등록된 강좌가 없습니다. 강좌 탭에서 먼저 등록해 주세요
          </p>
        )}
      </div>
    </section>
  );
}
