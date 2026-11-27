import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { useCan } from "../../gate/auth/useCan";
import { listRooms } from "./api";
import { RoomForm } from "./RoomForm";
import { RoomList } from "./RoomList";
import styles from "./RoomsPage.module.css";

type RoomDrawerState = null | { mode: "create" } | { mode: "edit"; id: number };

/**
 * 강의실 목록. 전체 폭 표를 보여 주고, 등록과 고치기는 오른쪽 서랍에서 처리한다.
 *
 * 기초 정보를 바꿀 수 없는 역할(BR-K3)에게는 등록·고치기 버튼을 숨긴다.
 */
export function RoomsPage() {
  const rooms = useQuery({ queryKey: ["rooms"], queryFn: listRooms });
  const [drawer, setDrawer] = useState<RoomDrawerState>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const canEdit = useCan("editReferenceData");

  const selectedId = drawer?.mode === "edit" ? drawer.id : null;
  const selected =
    rooms.data?.items.find((room) => room.id === selectedId) ?? null;

  const finish = (nextNotice: string) => {
    setNotice(nextNotice);
    setDrawer(null);
  };

  return (
    <div className={styles.page}>
      {notice && <NoticeBanner message={notice} />}
      <RoomList
        rooms={rooms}
        selectedId={selectedId}
        canEdit={canEdit}
        onSelect={(id) => {
          setNotice(null);
          setDrawer({ mode: "edit", id });
        }}
        onCreate={() => {
          setNotice(null);
          setDrawer({ mode: "create" });
        }}
      />

      {canEdit && drawer !== null && rooms.data && (
        <RoomForm
          key={drawer.mode === "edit" ? drawer.id : "new"}
          rooms={rooms.data.items}
          editing={drawer.mode === "edit" ? (selected ?? undefined) : undefined}
          onSaved={finish}
          onDeleted={finish}
          onClose={() => setDrawer(null)}
        />
      )}
    </div>
  );
}
