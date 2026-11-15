import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listCourses } from "../../api/courses";
import { listRooms } from "../../api/rooms";
import { listSubjects } from "../../api/subjects";
import { listTerms } from "../../api/terms";
import formStyles from "../../components/form/form.module.css";
import listStyles from "../../components/layout/listPage.module.css";
import { hasSeveralBuildings } from "../rooms/roomFormData";
import { TermBar } from "../terms/TermBar";
import { TermForm } from "../terms/TermForm";
import { currentTerm } from "../terms/termFormData";
import { CourseForm } from "./CourseForm";
import { CourseList } from "./CourseList";
import styles from "./CoursesPage.module.css";

/**
 * 강좌 탭. 맨 위에서 학기를 고르고, 그 아래에 그 학기의 강좌를 보여 준다.
 *
 * 처음에는 시작일이 가장 늦은 학기를 보여 준다. 학기가 하나도 없으면 학기 등록 칸부터 띄운다.
 * 강좌 목록은 왼쪽, 입력 칸은 오른쪽이다. 목록에서 강좌를 고르면 그 강좌 고치기로 바뀐다.
 */
export function CoursesPage() {
  const terms = useQuery({ queryKey: ["terms"], queryFn: listTerms });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const current = terms.data ? currentTerm(terms.data.items, selectedId) : null;
  const termId = current?.id ?? null;

  const courses = useQuery({
    queryKey: ["courses", termId],
    // enabled가 학기 없을 때 막으므로 0으로 부를 일은 없음
    queryFn: () => listCourses(termId ?? 0),
    enabled: termId !== null,
  });
  const subjects = useQuery({ queryKey: ["subjects"], queryFn: listSubjects });
  const rooms = useQuery({ queryKey: ["rooms"], queryFn: listRooms });
  const [selectedCourseId, setSelectedCourseId] = useState<number | null>(null);
  const [courseNotice, setCourseNotice] = useState<string | null>(null);

  if (terms.isPending) {
    return <p className={formStyles.loading}>학기를 불러오는 중입니다…</p>;
  }
  if (terms.isError) {
    return (
      <p className={formStyles.formError} role="alert">
        학기를 불러오지 못했습니다. 다시 열어 주세요.
      </p>
    );
  }

  if (!current) {
    return (
      <div className={styles.empty}>
        <p className={styles.emptyMessage}>
          강좌를 넣으려면 먼저 학기를 등록해 주세요.
        </p>
        <TermForm
          onSaved={(id, next) => {
            setNotice(next);
            setSelectedId(id);
          }}
        />
      </div>
    );
  }

  const showBuilding = rooms.data
    ? hasSeveralBuildings(rooms.data.items)
    : false;
  const selectedCourse =
    courses.data?.items.find((course) => course.id === selectedCourseId) ??
    null;

  return (
    <div className={styles.page}>
      <TermBar
        terms={terms.data.items}
        current={current}
        onSelect={(id) => {
          setSelectedId(id);
          setSelectedCourseId(null);
          setCourseNotice(null);
        }}
        notice={notice}
        onNotice={setNotice}
      />
      <div className={`${listStyles.page} ${styles.content}`}>
        <CourseList
          courses={courses}
          showBuilding={showBuilding}
          selectedId={selectedCourseId}
          onSelect={(id) => {
            setCourseNotice(null);
            setSelectedCourseId(id);
          }}
        />
        <aside className={listStyles.side}>
          {subjects.data && rooms.data && courses.data ? (
            // 학기나 강좌를 바꾸거나 등록으로 돌아오면 칸을 새로 만듦
            <CourseForm
              key={`${current.id}-${selectedCourse?.id ?? "new"}`}
              termId={current.id}
              subjects={subjects.data.items}
              rooms={rooms.data.items}
              showBuilding={showBuilding}
              editing={selectedCourse ?? undefined}
              initialNotice={courseNotice}
              onClose={(next) => {
                setCourseNotice(next ?? null);
                setSelectedCourseId(null);
              }}
            />
          ) : (
            <p className={formStyles.loading}>
              과목과 강의실을 불러오는 중입니다…
            </p>
          )}
        </aside>
      </div>
    </div>
  );
}
