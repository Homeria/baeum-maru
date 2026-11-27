import { useState } from "react";
import type { Gender } from "../../../api/generated/Gender";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { Button } from "../../../components/ui/Button";
import { SearchInput } from "../../../components/ui/SearchInput";
import { formatPhoneNumber } from "./formatPhoneNumber";
import { useMemberSearch } from "./useMemberSearch";
import { useSearchText } from "./useSearchText";
import styles from "./MemberList.module.css";

const PAGE_SIZE = 20;

const GENDER_LABEL: Record<Gender, string> = {
  male: "남",
  female: "여",
};

/** {@link MemberList}의 속성. */
export interface MemberListProps {
  /** 고치는 중이거나 선택된 회원. 그 줄을 강조한다. */
  selectedId: number | null;
  /** 회원을 등록하거나 고칠 수 있는지 여부. 참이면 등록·고치기 버튼을 보여 준다. */
  canEdit?: boolean;
  /** 회원을 고를 때 호출된다. 없으면 줄 클릭을 끈다. */
  onSelect?: (id: number) => void;
  /** 새 회원 등록 버튼을 눌렀을 때 호출된다. */
  onCreate?: () => void;
}

/** 검색창과 전체 폭 회원 표. 이름, 회원번호, 연락처 일부로 찾는다. */
export function MemberList({
  selectedId,
  canEdit = false,
  onSelect,
  onCreate,
}: MemberListProps) {
  const { text, setText, query } = useSearchText();
  // 검색어가 바뀌면 첫 쪽부터
  const [paging, setPaging] = useState({ query, page: 1 });
  const page = paging.query === query ? paging.page : 1;
  const goTo = (next: number) => setPaging({ query, page: next });

  const { data, isPending, isError, error, isPlaceholderData, refetch } =
    useMemberSearch({ query, page, pageSize: PAGE_SIZE });

  type MemberRow = NonNullable<typeof data>["items"][number];

  const lastPage = data ? Math.max(1, Math.ceil(data.total / PAGE_SIZE)) : 1;

  const columns: DataTableColumn<MemberRow>[] = [
    {
      key: "memberNo",
      header: "회원번호",
      width: "120px",
      render: (member) => member.memberNo,
    },
    {
      key: "name",
      header: "이름",
      isRowHeader: true,
      render: (member) => member.name,
    },
    {
      key: "phoneNumber",
      header: "연락처",
      render: (member) => formatPhoneNumber(member.phoneNumber),
    },
    {
      key: "birthDate",
      header: "생년월일",
      width: "130px",
      render: (member) => member.birthDate ?? "",
    },
    {
      key: "gender",
      header: "성별",
      width: "72px",
      align: "center",
      render: (member) => (member.gender ? GENDER_LABEL[member.gender] : ""),
    },
  ];

  if (canEdit && onSelect) {
    columns.push({
      key: "actions",
      header: "",
      width: "96px",
      align: "right",
      isAction: true,
      render: (member) => (
        <Button
          size="sm"
          variant="secondary"
          onClick={() => onSelect(member.id)}
        >
          고치기
        </Button>
      ),
    });
  }

  return (
    <Card padding="none">
      <div className={styles.toolbar}>
        <div className={styles.searchGroup}>
          <SearchInput
            label="회원 찾기"
            hideLabel
            value={text}
            onChange={setText}
            placeholder="이름, 회원번호, 연락처로 찾기"
            className={styles.searchInput}
          />
          {data && <p className={styles.total}>모두 {data.total}명</p>}
        </div>
        {canEdit && onCreate && (
          <Button variant="primary" onClick={onCreate}>
            회원 등록
          </Button>
        )}
      </div>

      <div className={styles.tableWrap} aria-busy={isPlaceholderData}>
        <DataTable
          caption="회원 목록"
          columns={columns}
          rows={data?.items ?? []}
          getRowKey={(member) => member.id}
          selectedKey={selectedId}
          onRowClick={onSelect ? (member) => onSelect(member.id) : undefined}
          loading={isPending}
          loadingText="회원 목록을 불러오는 중입니다…"
          error={isError ? error.message : null}
          onRetry={() => void refetch()}
          emptyMessage={
            query ? "찾는 회원이 없습니다" : "등록된 회원이 없습니다"
          }
        />
      </div>

      {data && data.total > 0 && (
        <div className={styles.pager}>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => goTo(page - 1)}
            disabled={page <= 1}
          >
            이전
          </Button>
          <span className={styles.pageInfo}>
            {page} / {lastPage}쪽 · 모두 {data.total}명
          </span>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => goTo(page + 1)}
            disabled={page >= lastPage}
          >
            다음
          </Button>
        </div>
      )}
    </Card>
  );
}
