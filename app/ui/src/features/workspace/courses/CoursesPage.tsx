import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Drawer } from "../../../components/overlay/Drawer";
import { Card } from "../../../components/surface/Card";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { useCan } from "../../gate/auth/useCan";
import { listRooms } from "../rooms/api";
import { hasSeveralBuildings } from "../rooms/roomFormData";
import { listSubjects } from "../subjects/api";
import { listTerms } from "../terms/api";
import { TermBar } from "../terms/TermBar";
import { TermForm } from "../terms/TermForm";
import { currentTerm } from "../terms/termFormData";
import { listCourses } from "./api";
import { CourseForm } from "./CourseForm";
import { usedTimeRanges } from "./courseFormData";
import { CourseList } from "./CourseList";
import styles from "./CoursesPage.module.css";

type CourseDrawerState =
  null | { mode: "create" } | { mode: "edit"; id: number };

/**
 * 강좌 탭. 맨 위에서 학기를 고르고, 그 아래에 전체 폭 강좌 표를 보여 준다.
 *
 * 처음에는 시작일이 가장 늦은 학기를 보여 준다. 학기가 하나도 없으면 학기 등록 카드부터 띄운다.
 * 강좌 등록과 고치기는 오른쪽 서랍에서 처리한다.
 * 학기·강좌를 바꿀 수 없는 역할(BR-K3)에게는 등록·고치기 버튼을 숨긴다.
 */
export function CoursesPage() {
  const terms = useQuery({ queryKey: ["terms"], queryFn: listTerms });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const canEdit = useCan("editCourses");
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
  const [drawer, setDrawer] = useState<CourseDrawerState>(null);
  const [courseNotice, setCourseNotice] = useState<string | null>(null);

  if (terms.isPending) {
    return <NoticeBanner message="학기를 불러오는 중입니다…" />;
  }
  if (terms.isError) {
    return (
      <NoticeBanner
        tone="error"
        message="학기를 불러오지 못했습니다. 다시 시도해 주세요."
        actionLabel="다시 해 보기"
        onAction={() => void terms.refetch()}
      />
    );
  }

  if (!current) {
    if (!canEdit) {
      return (
        <div className={styles.empty}>
          <NoticeBanner message="아직 등록된 학기가 없습니다. 직원이나 관리자에게 등록을 부탁해 주세요." />
        </div>
      );
    }
    return (
      <div className={styles.empty}>
        <Card
          title="새 학기 등록"
          subtitle="강좌를 넣으려면 먼저 학기를 등록해 주세요."
        >
          <TermForm
            onSaved={(id, next) => {
              setNotice(next);
              setSelectedId(id);
            }}
          />
        </Card>
      </div>
    );
  }

  const showBuilding = rooms.data
    ? hasSeveralBuildings(rooms.data.items)
    : false;
  const selectedCourseId = drawer?.mode === "edit" ? drawer.id : null;
  const selectedCourse =
    courses.data?.items.find((course) => course.id === selectedCourseId) ??
    null;

  const closeDrawer = () => setDrawer(null);
  const finishCourse = (nextNotice: string) => {
    setCourseNotice(nextNotice);
    setDrawer(null);
  };

  return (
    <div className={styles.page}>
      <TermBar
        terms={terms.data.items}
        current={current}
        onSelect={(id) => {
          setSelectedId(id);
          setDrawer(null);
          setCourseNotice(null);
        }}
        notice={notice}
        onNotice={setNotice}
        canEdit={canEdit}
      />

      {courseNotice && <NoticeBanner message={courseNotice} />}

      <CourseList
        courses={courses}
        showBuilding={showBuilding}
        selectedId={selectedCourseId}
        canEdit={canEdit}
        onSelect={(id) => {
          setCourseNotice(null);
          setDrawer({ mode: "edit", id });
        }}
        onCreate={() => {
          setCourseNotice(null);
          setDrawer({ mode: "create" });
        }}
      />

      {canEdit && drawer !== null && (
        <>
          {subjects.data && rooms.data && courses.data ? (
            <CourseForm
              key={`${current.id}-${drawer.mode === "edit" ? drawer.id : "new"}`}
              termId={current.id}
              subjects={subjects.data.items}
              rooms={rooms.data.items}
              showBuilding={showBuilding}
              usedTimes={usedTimeRanges(courses.data.items)}
              editing={
                drawer.mode === "edit"
                  ? (selectedCourse ?? undefined)
                  : undefined
              }
              onSaved={finishCourse}
              onDeleted={finishCourse}
              onClose={closeDrawer}
            />
          ) : (
            <Drawer
              open
              title={drawer.mode === "edit" ? "강좌 고치기" : "새 강좌 등록"}
              onClose={closeDrawer}
              footerRight={<Button onClick={closeDrawer}>닫기</Button>}
            >
              {subjects.isError || rooms.isError ? (
                <NoticeBanner
                  tone="error"
                  message="과목과 강의실 정보를 불러오지 못했습니다."
                  actionLabel="다시 해 보기"
                  onAction={() => {
                    void subjects.refetch();
                    void rooms.refetch();
                  }}
                />
              ) : (
                <NoticeBanner message="과목과 강의실을 불러오는 중입니다…" />
              )}
            </Drawer>
          )}
        </>
      )}
    </div>
  );
}
