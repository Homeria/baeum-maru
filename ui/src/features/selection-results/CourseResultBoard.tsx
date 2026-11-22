import { useId, useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { getCourseResult } from "../../api/selectionResults";
import formStyles from "../../components/form/form.module.css";
import {
  CONTACT_LABELS,
  WITHDRAWN_FROM_LABELS,
  matchesQuery,
  memberLabel,
  openSeatsLabel,
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
            </li>
          ))}
        </Column>
      </div>
    </section>
  );
}

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
