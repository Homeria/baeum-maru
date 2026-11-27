import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Card } from "../../../components/surface/Card";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { StatusPill } from "../../../components/ui/StatusPill";
import { useCan } from "../../gate/auth/useCan";
import { listCourses } from "../courses/api";
import { courseName } from "../courses/courseFormData";
import { listTerms } from "../terms/api";
import { TermBar } from "../terms/TermBar";
import { currentTerm } from "../terms/termFormData";
import { listClosedCourses } from "./api";
import { CourseResultBoard } from "./CourseResultBoard";
import { openSeatsLabel } from "./resultsData";
import { RosterExportDialog } from "./RosterExportDialog";
import styles from "./SelectionResultsPage.module.css";

/**
 * 선발 결과 탭. 선발을 마감한 강좌에서 포기와 대기자 승격을 처리한다(BR-C3, BR-W).
 *
 * 왼쪽은 세로 스크롤 마감 강좌 목록, 오른쪽은 고른 강좌의 당첨자·대기자·포기 보드다.
 * 학기를 바꾸면 고른 강좌를 비운다. 위쪽 버튼으로 결과 명단을 Excel로 내보낸다(BR-P1).
 */
export function SelectionResultsPage() {
  const terms = useQuery({ queryKey: ["terms"], queryFn: listTerms });
  const [selectedTermId, setSelectedTermId] = useState<number | null>(null);
  const [courseId, setCourseId] = useState<number | null>(null);
  const [exporting, setExporting] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const canExportRaw = useCan("exportRawRoster");
  const current = terms.data
    ? currentTerm(terms.data.items, selectedTermId)
    : null;
  const termId = current?.id ?? null;

  const courses = useQuery({
    queryKey: ["courses", termId],
    // enabled가 학기 없을 때 막으므로 0으로 부를 일은 없음
    queryFn: () => listCourses(termId ?? 0),
    enabled: termId !== null,
  });
  const closed = useQuery({
    queryKey: ["closed-courses", termId],
    queryFn: () => listClosedCourses(termId ?? 0),
    enabled: termId !== null,
  });

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
    return (
      <NoticeBanner message="아직 등록된 학기가 없습니다. 강좌 탭에서 학기와 강좌를 먼저 등록해 주세요." />
    );
  }

  const failure = courses.error ?? closed.error;
  const nameOf = (id: number) => {
    const course = courses.data?.items.find((candidate) => candidate.id === id);
    return course ? courseName(course) : "지워진 강좌";
  };
  const chosen = closed.data?.items.find((item) => item.courseId === courseId);

  return (
    <div className={styles.page}>
      <div className={styles.topRow}>
        <div className={styles.termSlot}>
          <TermBar
            terms={terms.data.items}
            current={current}
            onSelect={(id) => {
              setSelectedTermId(id);
              setCourseId(null);
              setNotice(null);
            }}
            notice={null}
            onNotice={() => undefined}
            canEdit={false}
          />
        </div>
        <div className={styles.exportAction}>
          <Button
            variant="primary"
            // 마감한 강좌가 없으면 만들 명단이 없음(BR-P1)
            disabled={!closed.data || closed.data.items.length === 0}
            onClick={() => {
              setNotice(null);
              setExporting(true);
            }}
          >
            결과 명단 내보내기
          </Button>
        </div>
      </div>

      {notice && <NoticeBanner message={notice} />}

      {exporting && (
        <RosterExportDialog
          termId={current.id}
          termName={current.name}
          canExportRaw={canExportRaw}
          onSaved={(path) => {
            setExporting(false);
            setNotice(`저장했습니다: ${path}`);
          }}
          onCancel={() => setExporting(false)}
        />
      )}

      {failure ? (
        <NoticeBanner
          tone="error"
          message={failure.message}
          actionLabel="다시 해 보기"
          onAction={() => {
            void courses.refetch();
            void closed.refetch();
          }}
        />
      ) : courses.data && closed.data ? (
        <div className={styles.content}>
          <Card
            title="마감한 강좌"
            subtitle={`모두 ${closed.data.items.length}개`}
            padding="none"
          >
            {closed.data.items.length === 0 ? (
              <div className={styles.emptyWrap}>
                <NoticeBanner message="선발을 마감한 강좌가 없습니다. 추첨 탭에서 추첨하고 마감하면 여기에 나옵니다." />
              </div>
            ) : (
              <ul className={styles.courseList} aria-label="마감한 강좌 목록">
                {closed.data.items.map((item) => {
                  const active = item.courseId === courseId;
                  return (
                    <li key={item.courseId} className={styles.courseItem}>
                      <button
                        type="button"
                        className={styles.courseButton}
                        aria-current={active ? "true" : undefined}
                        onClick={() => setCourseId(item.courseId)}
                      >
                        <div className={styles.courseButtonTop}>
                          <span className={styles.courseButtonName}>
                            {nameOf(item.courseId)}
                          </span>
                          <StatusPill
                            tone={item.openSeats > 0 ? "primary" : "neutral"}
                          >
                            {openSeatsLabel(item.openSeats)}
                          </StatusPill>
                        </div>
                        <span className={styles.courseButtonMeta}>
                          당첨 {item.selected}/{item.capacity} · 대기{" "}
                          {item.waitlisted}명
                        </span>
                      </button>
                    </li>
                  );
                })}
              </ul>
            )}
          </Card>

          {chosen ? (
            <CourseResultBoard
              // 강좌를 바꾸면 검색 칸을 새로 시작
              key={chosen.courseId}
              courseId={chosen.courseId}
              courseName={nameOf(chosen.courseId)}
            />
          ) : (
            <Card>
              <NoticeBanner message="왼쪽에서 강좌를 고르면 당첨자, 대기자, 포기한 사람이 나옵니다. 포기와 대기자 연락 결과를 여기서 남깁니다." />
            </Card>
          )}
        </div>
      ) : (
        <NoticeBanner message="강좌를 불러오는 중입니다…" />
      )}
    </div>
  );
}
