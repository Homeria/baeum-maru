import { useState } from "react";
import { Tabs, type TabItem } from "../../components/Tabs/Tabs";
import { RoomsPage } from "../rooms/RoomsPage";
import { SubjectsPage } from "../subjects/SubjectsPage";
import styles from "./ReferenceDataPage.module.css";

type ListKey = "subjects" | "rooms";

const LISTS: readonly TabItem<ListKey>[] = [
  { key: "subjects", label: "과목" },
  { key: "rooms", label: "강의실" },
];

/**
 * 기초 정보 탭. 학기가 바뀌어도 이어지는 목록을 작은 탭으로 나눠 보여 준다.
 *
 * 한 번 등록해 두고 강좌를 열 때 고르는 목록이라, 매일 쓰는 탭과 떼어 둔다.
 */
export function ReferenceDataPage() {
  const [list, setList] = useState<ListKey>("subjects");

  return (
    <div className={styles.page}>
      <Tabs
        label="기초 정보"
        items={LISTS}
        selected={list}
        onSelect={setList}
      />
      <div role="tabpanel" className={styles.content}>
        {list === "subjects" && <SubjectsPage />}
        {list === "rooms" && <RoomsPage />}
      </div>
    </div>
  );
}
