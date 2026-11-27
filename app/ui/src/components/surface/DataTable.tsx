import { useId, type MouseEvent, type ReactNode } from "react";
import { NoticeBanner } from "./NoticeBanner";
import styles from "./DataTable.module.css";

/** {@link DataTable}의 열 정의. */
export interface DataTableColumn<T> {
  /** 열 고유 키. */
  key: string;
  /** 머리줄에 보여 줄 글자. 동작 열은 빈 글자 `""`로 둔다. */
  header: string;
  /** 셀 내용을 그리는 함수. */
  render: (row: T) => ReactNode;
  /** 정렬 방향. 기본값은 `"left"`. */
  align?: "left" | "center" | "right";
  /** 열 너비(CSS width 값). */
  width?: string;
  /** 참이면 `<th scope="row">`로 그려 행의 기준 이름임을 알린다. */
  isRowHeader?: boolean;
  /** 참이면 이 셀을 눌러도 행 클릭(`onRowClick`)으로 번지지 않는다. */
  isAction?: boolean;
}

/** {@link DataTable}의 속성. */
export interface DataTableProps<T> {
  /** 표의 접근성 이름(`<caption>`). 화면에서는 시각 숨김 처리한다. */
  caption: string;
  columns: readonly DataTableColumn<T>[];
  rows: readonly T[];
  /** 각 행의 고유 키를 꺼내는 함수. */
  getRowKey: (row: T) => string | number;
  /** 현재 선택된 행의 키. */
  selectedKey?: string | number | null;
  /** 행 강조 성격. 선발 결과 대기자 '연락할 차례'에는 `"turn"`을 쓴다. */
  rowTone?: (row: T) => "default" | "turn";
  /** 행을 눌렀을 때 실행할 함수. 셀 안의 버튼·입력 칸을 누른 클릭은 무시한다. */
  onRowClick?: (row: T) => void;
  /** 불러오는 중 여부. */
  loading?: boolean;
  /** 불러오는 중 문구. */
  loadingText?: string;
  /** 불러오기 실패 문구. */
  error?: string | null;
  /** 실패 시 다시 불러오기 함수. */
  onRetry?: () => void;
  /** 행이 0개일 때 보여 줄 안내 문구. */
  emptyMessage?: string;
  className?: string;
}

function isInteractiveTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) {
    return false;
  }
  return Boolean(
    target.closest(
      "button, a, input, select, textarea, label, [role='button']",
    ),
  );
}

/**
 * 시맨틱 `<table>`과 고정 머리줄, 키보드 스크롤 영역을 갖춘 공통 데이터 표.
 *
 * 머리줄은 `14px Bold`(40px), 본문은 `16px`이며, 빈 목록과 불러오기 오류를 일관되게 표시한다.
 */
export function DataTable<T>({
  caption,
  columns,
  rows,
  getRowKey,
  selectedKey = null,
  rowTone,
  onRowClick,
  loading = false,
  loadingText = "불러오는 중…",
  error = null,
  onRetry,
  emptyMessage = "표시할 항목이 없습니다.",
  className,
}: DataTableProps<T>) {
  const captionId = useId();

  if (loading) {
    return (
      <div className={styles.stateBox}>
        <NoticeBanner message={loadingText} />
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.stateBox}>
        <NoticeBanner
          tone="error"
          message={error}
          actionLabel={onRetry ? "다시 해 보기" : undefined}
          onAction={onRetry}
        />
      </div>
    );
  }

  if (rows.length === 0) {
    return (
      <div className={styles.stateBox}>
        <NoticeBanner message={emptyMessage} />
      </div>
    );
  }

  function handleRowClick(row: T, event: MouseEvent<HTMLTableRowElement>) {
    if (!onRowClick || isInteractiveTarget(event.target)) {
      return;
    }
    onRowClick(row);
  }

  return (
    <div
      tabIndex={0}
      role="region"
      aria-labelledby={captionId}
      className={[styles.scrollRegion, className].filter(Boolean).join(" ")}
    >
      <table className={styles.table}>
        <caption id={captionId} className="visually-hidden">
          {caption}
        </caption>
        <thead className={styles.head}>
          <tr>
            {columns.map((col) => (
              <th
                key={col.key}
                scope="col"
                style={col.width ? { width: col.width } : undefined}
                className={[
                  styles.th,
                  col.align ? styles[col.align] : styles.left,
                ].join(" ")}
              >
                {col.header || <span className="visually-hidden">동작</span>}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const key = getRowKey(row);
            const selected = selectedKey !== null && key === selectedKey;
            const tone = rowTone ? rowTone(row) : "default";
            const rowClasses = [
              styles.row,
              onRowClick ? styles.clickable : "",
              selected ? styles.selected : "",
              tone === "turn" ? styles.turnRow : "",
            ]
              .filter(Boolean)
              .join(" ");

            return (
              <tr
                key={key}
                data-selected={selected ? "true" : undefined}
                className={rowClasses}
                onClick={
                  onRowClick ? (event) => handleRowClick(row, event) : undefined
                }
              >
                {columns.map((col) => {
                  const cellClasses = [
                    styles.td,
                    col.align ? styles[col.align] : styles.left,
                    col.isAction ? styles.actionCell : "",
                  ]
                    .filter(Boolean)
                    .join(" ");

                  if (col.isRowHeader) {
                    return (
                      <th key={col.key} scope="row" className={cellClasses}>
                        {col.render(row)}
                      </th>
                    );
                  }
                  return (
                    <td
                      key={col.key}
                      className={cellClasses}
                      onClick={
                        col.isAction
                          ? (event) => event.stopPropagation()
                          : undefined
                      }
                    >
                      {col.render(row)}
                    </td>
                  );
                })}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
