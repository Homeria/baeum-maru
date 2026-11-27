import { useState } from "react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { listAccessLogs } from "./api";
import tableStyles from "../../../components/table/table.module.css";
import {
  ACCESS_EVENT_LABELS,
  formatOccurredAt,
  isFailure,
} from "./accessLogData";
import styles from "./AccessLogList.module.css";

const PAGE_SIZE = 50;

/** 접속 기록 (BR-K13). 최근 것부터 한 쪽에 50줄. 고치거나 지울 수 없다. */
export function AccessLogList() {
  const [page, setPage] = useState(1);
  const { data, isPending, isError, error, isPlaceholderData, refetch } =
    useQuery({
      queryKey: ["accessLogs", page],
      queryFn: () => listAccessLogs(page, PAGE_SIZE),
      placeholderData: keepPreviousData,
    });

  if (isPending) {
    return <p className={tableStyles.message}>불러오는 중입니다…</p>;
  }
  if (isError) {
    return (
      <p className={tableStyles.error} role="alert">
        {error.message}
      </p>
    );
  }

  const lastPage = Math.max(1, Math.ceil(data.total / PAGE_SIZE));

  return (
    <section className={styles.list} aria-label="접속 기록">
      <div className={tableStyles.tableWrap} aria-busy={isPlaceholderData}>
        <table className={tableStyles.table} aria-label="접속 기록 목록">
          <thead>
            <tr>
              <th scope="col">시각</th>
              <th scope="col">직원</th>
              <th scope="col">기기</th>
              <th scope="col">일</th>
            </tr>
          </thead>
          <tbody>
            {data.items.map((item) => (
              <tr key={item.id}>
                <td>{formatOccurredAt(item.occurredAt)}</td>
                <td>{item.staffName ?? "(고른 직원 없음)"}</td>
                <td>{item.deviceName ?? "호스트 PC"}</td>
                <td
                  className={isFailure(item.event) ? styles.failure : undefined}
                >
                  {ACCESS_EVENT_LABELS[item.event]}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {data.items.length === 0 && (
          <p className={tableStyles.message}>접속 기록이 없습니다</p>
        )}
      </div>

      <div className={styles.pager}>
        <button
          type="button"
          onClick={() => setPage(page - 1)}
          disabled={page <= 1}
        >
          이전
        </button>
        <span>
          {page} / {lastPage}쪽 · 모두 {data.total}줄
        </span>
        <button
          type="button"
          onClick={() => setPage(page + 1)}
          disabled={page >= lastPage}
        >
          다음
        </button>
        <button type="button" onClick={() => void refetch()}>
          새로 보기
        </button>
      </div>
    </section>
  );
}
