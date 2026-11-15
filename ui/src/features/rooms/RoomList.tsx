import type { UseQueryResult } from "@tanstack/react-query";
import type { RoomListResponse } from "../../api/generated/RoomListResponse";
import tableStyles from "../../components/table/table.module.css";
import { floorLabel } from "./floor";
import { hasSeveralBuildings } from "./roomFormData";
import styles from "./RoomsPage.module.css";

/** {@link RoomList}의 속성. */
export interface RoomListProps {
  rooms: UseQueryResult<RoomListResponse>;
}

/** 강의실 목록. 건물이 하나뿐이면 건물 열을 숨긴다. */
export function RoomList({ rooms }: RoomListProps) {
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
              <tr key={room.id}>
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
