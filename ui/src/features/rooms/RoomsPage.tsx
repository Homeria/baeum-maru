import { useQuery } from "@tanstack/react-query";
import { listRooms } from "../../api/rooms";
import formStyles from "../../components/form/form.module.css";
import { RoomForm } from "./RoomForm";
import { RoomList } from "./RoomList";
import styles from "./RoomsPage.module.css";

/** 강의실 탭. 왼쪽은 강의실 목록, 오른쪽은 등록 칸. */
export function RoomsPage() {
  const rooms = useQuery({ queryKey: ["rooms"], queryFn: listRooms });

  return (
    <div className={styles.page}>
      <RoomList rooms={rooms} />
      <aside className={styles.side}>
        {rooms.data ? (
          <RoomForm rooms={rooms.data.items} />
        ) : (
          <p className={formStyles.loading}>강의실을 불러오는 중입니다…</p>
        )}
      </aside>
    </div>
  );
}
