import { useState } from "react";
import { Tabs, type TabItem } from "../../components/Tabs/Tabs";
import { RoomsPage } from "../rooms/RoomsPage";
import styles from "./ReferenceDataPage.module.css";

type ListKey = "rooms";

const LISTS: readonly TabItem<ListKey>[] = [{ key: "rooms", label: "강의실" }];

/**
 * 기초 정보 탭. 학기가 바뀌어도 이어지는 목록을 작은 탭으로 나눠 보여 준다.
 *
 * 한 번 등록해 두고 강좌를 열 때 고르는 목록이라, 매일 쓰는 탭과 떼어 둔다.
 */
export function ReferenceDataPage() {
  const [list, setList] = useState<ListKey>("rooms");

  return (
    <div className={styles.page}>
      <Tabs
        label="기초 정보"
        items={LISTS}
        selected={list}
        onSelect={setList}
      />
      <div role="tabpanel" className={styles.content}>
        {list === "rooms" && <RoomsPage />}
      </div>
    </div>
  );
}
