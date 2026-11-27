import { useState } from "react";
import type { Gender } from "../../../api/generated/Gender";
import { formatPhoneNumber } from "./formatPhoneNumber";
import { useMemberSearch } from "./useMemberSearch";
import { useSearchText } from "./useSearchText";
import tableStyles from "../../../components/table/table.module.css";
import styles from "./MemberList.module.css";

const PAGE_SIZE = 20;

const GENDER_LABEL: Record<Gender, string> = {
  male: "남",
  female: "여",
};

/** {@link MemberList}의 속성. */
export interface MemberListProps {
  /** 고치는 중인 회원. 그 줄을 강조한다. */
  selectedId: number | null;
  onSelect: (id: number) => void;
}

/** 검색창과 회원 목록. 이름, 회원번호, 연락처 일부로 찾는다. 줄을 누르면 그 회원을 고른다. */
export function MemberList({ selectedId, onSelect }: MemberListProps) {
  const { query, inputProps } = useSearchText();
  // 검색어가 바뀌면 첫 쪽부터
  const [paging, setPaging] = useState({ query, page: 1 });
  const page = paging.query === query ? paging.page : 1;
  const goTo = (next: number) => setPaging({ query, page: next });

  const { data, isPending, isError, error, isPlaceholderData } =
    useMemberSearch({ query, page, pageSize: PAGE_SIZE });

  const lastPage = data ? Math.max(1, Math.ceil(data.total / PAGE_SIZE)) : 1;

  return (
    <section className={styles.list} aria-label="회원 찾기">
      <input
        {...inputProps}
        type="search"
        className={styles.search}
        placeholder="이름, 회원번호, 연락처로 찾기"
        aria-label="회원 찾기"
        autoComplete="off"
      />

      {isPending ? (
        <p className={tableStyles.message}>찾는 중입니다…</p>
      ) : isError ? (
        <p className={tableStyles.error} role="alert">
          {error.message}
        </p>
      ) : (
        <>
          <div className={tableStyles.tableWrap} aria-busy={isPlaceholderData}>
            <table className={tableStyles.table} aria-label="회원 목록">
              <thead>
                <tr>
                  <th scope="col">회원번호</th>
                  <th scope="col">이름</th>
                  <th scope="col">연락처</th>
                  <th scope="col">생년월일</th>
                  <th scope="col">성별</th>
                </tr>
              </thead>
              <tbody>
                {data.items.map((member) => (
                  <tr
                    key={member.id}
                    className={tableStyles.row}
                    tabIndex={0}
                    aria-current={member.id === selectedId ? "true" : undefined}
                    onClick={() => onSelect(member.id)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        onSelect(member.id);
                      }
                    }}
                  >
                    <td>{member.memberNo}</td>
                    <td>{member.name}</td>
                    <td>{formatPhoneNumber(member.phoneNumber)}</td>
                    <td>{member.birthDate ?? ""}</td>
                    <td>{member.gender ? GENDER_LABEL[member.gender] : ""}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {data.items.length === 0 && (
              <p className={tableStyles.message}>
                {query ? "찾는 회원이 없습니다" : "등록된 회원이 없습니다"}
              </p>
            )}
          </div>

          <div className={styles.pager}>
            <button
              type="button"
              onClick={() => goTo(page - 1)}
              disabled={page <= 1}
            >
              이전
            </button>
            <span>
              {page} / {lastPage}쪽 · 모두 {data.total}명
            </span>
            <button
              type="button"
              onClick={() => goTo(page + 1)}
              disabled={page >= lastPage}
            >
              다음
            </button>
          </div>
        </>
      )}
    </section>
  );
}
