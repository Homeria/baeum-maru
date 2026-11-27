import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";
import styles from "./ApplicationsPage.module.css";

/** {@link PriorityChoices}의 속성. */
export interface PriorityChoicesProps {
  /** 기관이 만든 우선 배정 사유. 이름 순서. */
  reasons: readonly PriorityReasonResponse[];
  /** 붙인 사유 id. */
  checked: readonly number[];
  /** 화면 읽기 프로그램에 읽어 줄 강좌 이름. 예: "요가 초급" */
  courseName: string;
  disabled: boolean;
  onToggle: (reasonId: number) => void;
}

/**
 * 신청 하나에 붙일 우선 배정 사유 체크박스(BR-S12). 여러 개를 고를 수 있다.
 *
 * 아무것도 고르지 않으면 일반 추첨이다.
 */
export function PriorityChoices({
  reasons,
  checked,
  courseName,
  disabled,
  onToggle,
}: PriorityChoicesProps) {
  return (
    <span
      className={styles.priorityChoices}
      role="group"
      aria-label={`${courseName} 우선 배정`}
    >
      <span className={styles.priorityLabel}>우선 배정</span>
      {reasons.map((reason) => (
        <label key={reason.id} className={styles.priorityChoice}>
          <input
            type="checkbox"
            checked={checked.includes(reason.id)}
            disabled={disabled}
            onChange={() => onToggle(reason.id)}
          />
          {reason.name}
        </label>
      ))}
    </span>
  );
}
