import type { UseQueryResult } from "@tanstack/react-query";
import type { RoomListResponse } from "../../../api/generated/RoomListResponse";
import tableStyles from "../../../components/table/table.module.css";
import { floorLabel } from "./floor";
import { hasSeveralBuildings } from "./roomFormData";
import styles from "../../../components/layout/listPage.module.css";

/** {@link RoomList}의 속성. */
export interface RoomListProps {
  rooms: UseQueryResult<RoomListResponse>;
  /** 고치는 중인 강의실. 그 줄을 강조한다. */
  selectedId: number | null;
  onSelect: (id: number) => void;
}

/** 강의실 목록. 건물이 하나뿐이면 건물 열을 숨긴다. 줄을 누르면 그 강의실을 고른다. */
export function RoomList({ rooms, selectedId, onSelect }: RoomListProps) {
  const { data, isPending, isError, error } = rooms;

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

  const showBuilding = hasSeveralBuildings(data.items);

  return (
    <section className={styles.list} aria-label="강의실">
      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table} aria-label="강의실 목록">
          <thead>
            <tr>
              {showBuilding && <th scope="col">건물</th>}
              <th scope="col">층</th>
              <th scope="col">이름</th>
            </tr>
          </thead>
          <tbody>
            {data.items.map((room) => (
              <tr
                key={room.id}
                className={tableStyles.row}
                tabIndex={0}
                aria-current={room.id === selectedId ? "true" : undefined}
                onClick={() => onSelect(room.id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onSelect(room.id);
                  }
                }}
              >
                {showBuilding && <td>{room.building}</td>}
                <td>{floorLabel(room.floor)}</td>
                <td>{room.name}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {data.items.length === 0 && (
          <p className={tableStyles.message}>
            등록된 강의실이 없습니다. 오른쪽 칸에서 등록해 주세요
          </p>
        )}
      </div>
      <p className={styles.summary}>모두 {data.items.length}개</p>
    </section>
  );
}
