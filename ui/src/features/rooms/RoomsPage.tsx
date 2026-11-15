import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listRooms } from "../../api/rooms";
import formStyles from "../../components/form/form.module.css";
import { RoomForm } from "./RoomForm";
import { RoomList } from "./RoomList";
import styles from "../../components/layout/listPage.module.css";

/**
 * 강의실 탭. 왼쪽은 강의실 목록, 오른쪽은 입력 칸.
 *
 * 평소 오른쪽은 새 강의실 등록이고, 목록에서 강의실을 고르면 그 강의실 고치기로 바뀐다.
 * 강의실은 칸이 적어 목록에 받은 값으로 바로 고친다.
 */
export function RoomsPage() {
  const rooms = useQuery({ queryKey: ["rooms"], queryFn: listRooms });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const selected =
    rooms.data?.items.find((room) => room.id === selectedId) ?? null;

  const select = (id: number) => {
    setNotice(null);
    setSelectedId(id);
  };

  const close = (next?: string) => {
    setNotice(next ?? null);
    setSelectedId(null);
  };

  return (
    <div className={styles.page}>
      <RoomList rooms={rooms} selectedId={selectedId} onSelect={select} />
      <aside className={styles.side}>
        {rooms.data ? (
          // 다른 강의실을 고르거나 등록으로 돌아오면 칸을 새로 만듦
          <RoomForm
            key={selected?.id ?? "new"}
            rooms={rooms.data.items}
            editing={selected ?? undefined}
            initialNotice={notice}
            onClose={close}
          />
        ) : (
          <p className={formStyles.loading}>강의실을 불러오는 중입니다…</p>
        )}
      </aside>
    </div>
  );
}
