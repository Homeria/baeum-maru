import { useState } from "react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import type { AccessLogResponse } from "../../../api/generated/AccessLogResponse";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { Button } from "../../../components/ui/Button";
import { StatusPill } from "../../../components/ui/StatusPill";
import {
  ACCESS_EVENT_LABELS,
  formatOccurredAt,
  isFailure,
} from "./accessLogData";
import { listAccessLogs } from "./api";
import styles from "./AccessLogList.module.css";

const PAGE_SIZE = 50;

/** 접속 기록(BR-K13). 최근 것부터 한 쪽에 50줄. 고치거나 지울 수 없다. */
export function AccessLogList() {
  const [page, setPage] = useState(1);
  const { data, isPending, isError, error, isPlaceholderData, refetch } =
    useQuery({
      queryKey: ["accessLogs", page],
      queryFn: () => listAccessLogs(page, PAGE_SIZE),
      placeholderData: keepPreviousData,
    });

  const lastPage = data ? Math.max(1, Math.ceil(data.total / PAGE_SIZE)) : 1;

  const columns: DataTableColumn<AccessLogResponse>[] = [
    {
      key: "occurredAt",
      header: "시각",
      width: "200px",
      isRowHeader: true,
      render: (item) => formatOccurredAt(item.occurredAt),
    },
    {
      key: "staffName",
      header: "직원",
      width: "180px",
      render: (item) => item.staffName ?? "(고른 직원 없음)",
    },
    {
      key: "deviceName",
      header: "기기",
      render: (item) => item.deviceName ?? "호스트 PC",
    },
    {
      key: "event",
      header: "일",
      width: "180px",
      render: (item) => (
        <StatusPill tone={isFailure(item.event) ? "bad" : "neutral"}>
          {ACCESS_EVENT_LABELS[item.event]}
        </StatusPill>
      ),
    },
  ];

  return (
    <Card padding="none">
      <div className={styles.toolbar}>
        <div className={styles.headingGroup}>
          <h2 className={styles.title}>접속 기록</h2>
          {data && <p className={styles.count}>모두 {data.total}줄</p>}
        </div>
        <Button size="sm" variant="secondary" onClick={() => void refetch()}>
          새로 보기
        </Button>
      </div>

      <div className={styles.tableWrap} aria-busy={isPlaceholderData}>
        <DataTable
          caption="접속 기록 목록"
          columns={columns}
          rows={data?.items ?? []}
          getRowKey={(item) => item.id}
          loading={isPending}
          loadingText="접속 기록을 불러오는 중입니다…"
          error={isError ? error.message : null}
          onRetry={() => void refetch()}
          emptyMessage="접속 기록이 없습니다."
        />
      </div>

      {data && data.total > 0 && (
        <div className={styles.pager}>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => setPage(page - 1)}
            disabled={page <= 1}
          >
            이전
          </Button>
          <span className={styles.pageInfo}>
            {page} / {lastPage}쪽 · 모두 {data.total}줄
          </span>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => setPage(page + 1)}
            disabled={page >= lastPage}
          >
            다음
          </Button>
        </div>
      )}
    </Card>
  );
}
