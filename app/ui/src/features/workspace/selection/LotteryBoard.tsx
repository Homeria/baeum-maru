import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { CourseLotteryResponse } from "../../../api/generated/CourseLotteryResponse";
import type { EntrantResponse } from "../../../api/generated/EntrantResponse";
import type { SelectionStageCode } from "../../../api/generated/SelectionStageCode";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { Card } from "../../../components/surface/Card";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import {
  StatusPill,
  type StatusPillTone,
} from "../../../components/ui/StatusPill";
import { STAGE_LABELS } from "../courses/courseStatusData";
import { closeSelection, getLottery, runLottery } from "./api";
import {
  SHUFFLE_INTERVAL_MS,
  drawPool,
  entrantLabel,
  guaranteedOf,
  lotteryButtons,
  needsShuffle,
  resultColumns,
  shuffled,
} from "./selectionData";
import styles from "./SelectionPage.module.css";

/** {@link LotteryBoard}의 속성. */
export interface LotteryBoardProps {
  termId: number;
  courseId: number;
  /** 화면 위에 보여 줄 강좌 이름. 예: 서예 초급 1반 */
  courseName: string;
}

const STAGE_TONE: Record<SelectionStageCode, StatusPillTone> = {
  not_drawn: "neutral",
  drawn: "primary",
  closed: "ok",
};

/**
 * 강좌 하나의 추첨 보드. 신청자, 당첨자, 대기자 세 칸 카드로 구성한다.
 *
 * 시작을 누르면 신청자 목록을 0.25초마다 섞어 보여 준다. 멈춤을 누르면 호스트가 그 순간
 * 순서를 정하고(ADR 0016), 신청자 목록이 그 순서로 멈춘 뒤 앞에서 정원만큼 당첨, 나머지는
 * 그 순서대로 대기가 된다(BR-S7). 화면은 결과를 계산하지 않는다.
 *
 * 자리 보장 신청(BR-S4)은 추첨하지 않아 신청자 목록에서 빠지고, 당첨자 목록 맨 위에 늘
 * 고정된다. 옅은 바탕과 "우선" 꼬리표로 표시하고 사유 이름은 보이지 않는다(BR-S15).
 */
export function LotteryBoard({
  termId,
  courseId,
  courseName,
}: LotteryBoardProps) {
  const queryClient = useQueryClient();
  const lottery = useQuery({
    queryKey: ["lottery", courseId],
    queryFn: () => getLottery(courseId),
  });
  // 섞는 동안 보여 주는 순서. 섞지 않을 때는 null
  const [shown, setShown] = useState<readonly EntrantResponse[] | null>(null);
  const [asking, setAsking] = useState<"redraw" | "close" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const cancelAskRef = useRef<HTMLButtonElement>(null);
  const shuffling = shown !== null;

  useEffect(() => {
    if (!shuffling) {
      return;
    }
    const timer = setInterval(() => {
      setShown((prev) => (prev ? shuffled(prev) : prev));
    }, SHUFFLE_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [shuffling]);

  const afterChange = () => {
    void queryClient.invalidateQueries({ queryKey: ["selections", termId] });
    // 신청 탭의 신청 목록과 고를 수 있는 강좌가 바뀐다
    void queryClient.invalidateQueries({ queryKey: ["applications"] });
  };

  const draw = useMutation({
    mutationFn: () => runLottery(courseId),
    onSuccess: (result) => {
      queryClient.setQueryData(["lottery", courseId], result);
      setShown(null);
      setAsking(null);
      afterChange();
    },
    onError: (failure) => {
      setShown(null);
      setAsking(null);
      setError(failure.message);
      void queryClient.invalidateQueries({ queryKey: ["lottery", courseId] });
    },
  });

  const close = useMutation({
    mutationFn: () => closeSelection(courseId),
    onSuccess: () => {
      setAsking(null);
      void queryClient.invalidateQueries({ queryKey: ["lottery", courseId] });
      afterChange();
    },
    onError: (failure) => {
      setAsking(null);
      setError(failure.message);
    },
  });

  if (lottery.isPending) {
    return <NoticeBanner message="추첨 화면을 불러오는 중입니다…" />;
  }
  if (lottery.isError) {
    return (
      <NoticeBanner
        tone="error"
        message={lottery.error.message}
        actionLabel="다시 해 보기"
        onAction={() => void lottery.refetch()}
      />
    );
  }

  const data = lottery.data;
  /** 시작: 지금 순서부터 섞기 시작한다. 섞을 까닭이 없으면 바로 뽑는다. */
  const start = (current: CourseLotteryResponse) => {
    setError(null);
    if (needsShuffle(current)) {
      setShown(shuffled(drawPool(current.entrants)));
    } else {
      draw.mutate();
    }
  };
  const applicants = drawPool(data.entrants);
  const guaranteedCount = guaranteedOf(data.entrants).length;
  // 섞는 동안에는 신청자 칸은 고정하고 당첨자·대기자 칸이 실시간으로 섞인다
  const result = resultColumns(data, shown);
  const buttons = lotteryButtons(data);

  return (
    <section className={styles.board} aria-label={`${courseName} 추첨`}>
      <div className={styles.boardHeader}>
        <h2 className={styles.courseTitle}>{courseName}</h2>
        <div className={styles.metaGroup}>
          <span className={styles.meta}>정원 {data.capacity}명</span>
          <StatusPill tone={STAGE_TONE[data.stage]}>
            {STAGE_LABELS[data.stage]}
          </StatusPill>
        </div>
      </div>

      <div className={styles.columns}>
        <EntrantColumn
          title="신청자"
          entrants={applicants}
          note={
            guaranteedCount > 0
              ? `우선 배정 ${guaranteedCount}명은 추첨 없이 당첨자에 있습니다`
              : undefined
          }
          emptyText="추첨 대상 신청자가 없습니다."
        />
        <EntrantColumn
          title="당첨자"
          entrants={result.selected}
          shuffling={shuffling}
          highlightHeader
          emptyText="아직 당첨자가 없습니다."
        />
        <EntrantColumn
          title="대기자"
          entrants={result.waitlisted}
          shuffling={shuffling}
          numbered
          emptyText="대기자가 없습니다."
        />
      </div>

      {error && <NoticeBanner tone="error" message={error} />}

      {buttons === "overCapacity" ? (
        <NoticeBanner
          tone="error"
          message={`우선 배정 신청 ${guaranteedCount}명이 정원 ${data.capacity}명보다 많아 추첨할 수 없습니다. 신청 탭에서 우선 배정을 줄이거나 강좌 탭에서 정원을 늘려 주세요.`}
        />
      ) : buttons === "none" ? (
        <NoticeBanner message={noneHint(data)} />
      ) : (
        <Card>
          <div className={styles.controls}>
            <div className={styles.controlLeft}>
              {buttons === "start" && (
                <p className={styles.hint}>
                  {shuffling
                    ? "당첨자와 대기자 순서를 섞는 중입니다. 멈춤을 누르면 결과가 확정됩니다."
                    : "시작을 누르면 당첨자·대기자 목록이 섞이고, 멈춤을 누른 순간 추첨이 실행됩니다."}
                </p>
              )}
              {buttons === "selectAll" && (
                <p className={styles.hint}>
                  {guaranteedCount > 0
                    ? "추첨할 신청자가 남은 자리 이하라 추첨 없이 모두 당첨입니다."
                    : "신청자가 정원 이하라 추첨 없이 모두 당첨입니다."}
                </p>
              )}
              {buttons === "redraw" && !shuffling && (
                <p className={styles.hint}>
                  추첨 결과를 확인한 뒤 선발을 마감하거나 다시 추첨할 수
                  있습니다.
                </p>
              )}
              {buttons === "redraw" && shuffling && (
                <p className={styles.hint}>
                  당첨자와 대기자 순서를 다시 섞는 중입니다. 멈춤을 누르면 새
                  결과가 확정됩니다.
                </p>
              )}
            </div>

            <div className={styles.controlActions}>
              {buttons === "start" && (
                <Button
                  variant="primary"
                  className={styles.drawButton}
                  busy={draw.isPending}
                  busyLabel="뽑는 중…"
                  onClick={() => (shuffling ? draw.mutate() : start(data))}
                >
                  {shuffling ? "멈춤" : "시작"}
                </Button>
              )}
              {buttons === "selectAll" && (
                <Button
                  variant="primary"
                  className={styles.drawButton}
                  busy={draw.isPending}
                  busyLabel="처리하는 중…"
                  onClick={() => start(data)}
                >
                  전원 당첨
                </Button>
              )}
              {buttons === "redraw" &&
                (shuffling ? (
                  <Button
                    variant="primary"
                    className={styles.drawButton}
                    busy={draw.isPending}
                    busyLabel="뽑는 중…"
                    onClick={() => draw.mutate()}
                  >
                    멈춤
                  </Button>
                ) : (
                  <>
                    <Button
                      variant="secondary"
                      onClick={() => setAsking("redraw")}
                    >
                      다시 추첨
                    </Button>
                    <Button
                      variant="primary"
                      onClick={() => setAsking("close")}
                    >
                      선발 마감
                    </Button>
                  </>
                ))}
            </div>
          </div>
        </Card>
      )}

      <ModalDialog
        open={asking === "redraw"}
        size="sm"
        title={`${courseName}을(를) 다시 추첨할까요?`}
        busy={draw.isPending}
        initialFocusRef={cancelAskRef}
        onClose={() => setAsking(null)}
        footer={
          <>
            <Button
              ref={cancelAskRef}
              variant="secondary"
              disabled={draw.isPending}
              onClick={() => setAsking(null)}
            >
              그만두기
            </Button>
            <Button
              variant="dangerSolid"
              busy={draw.isPending}
              busyLabel="처리하는 중…"
              onClick={() => {
                setAsking(null);
                start(data);
              }}
            >
              다시 추첨
            </Button>
          </>
        }
      >
        <NoticeBanner
          tone="error"
          message="지금 결과는 취소됨으로 기록에 남고, 신청자 모두를 처음부터 다시 뽑습니다."
        />
      </ModalDialog>

      <ModalDialog
        open={asking === "close"}
        size="sm"
        title={`${courseName}의 선발을 마감할까요?`}
        busy={close.isPending}
        initialFocusRef={cancelAskRef}
        onClose={() => setAsking(null)}
        footer={
          <>
            <Button
              ref={cancelAskRef}
              variant="secondary"
              disabled={close.isPending}
              onClick={() => setAsking(null)}
            >
              그만두기
            </Button>
            <Button
              variant="dangerSolid"
              busy={close.isPending}
              busyLabel="마감하는 중…"
              onClick={() => close.mutate()}
            >
              선발 마감
            </Button>
          </>
        }
      >
        <NoticeBanner
          tone="error"
          message="마감하면 다시 추첨할 수 없습니다."
        />
      </ModalDialog>
    </section>
  );
}

/** 버튼이 없을 때 까닭. */
function noneHint(lottery: CourseLotteryResponse): string {
  if (lottery.stage === "closed") {
    return "선발을 마감했습니다.";
  }
  if (!lottery.canDraw) {
    return "추첨과 선발 마감은 호스트 PC에서 관리자·직원이 합니다. 이 화면에서는 결과만 봅니다.";
  }
  return "추첨할 신청이 없습니다.";
}

/** {@link EntrantColumn}의 속성. */
interface EntrantColumnProps {
  title: string;
  entrants: readonly EntrantResponse[];
  /** 섞는 중이면 목록을 강조한다. */
  shuffling?: boolean;
  /** 대기자처럼 순번을 붙일지. */
  numbered?: boolean;
  /** 당첨자 칸처럼 파란 머리줄을 쓸지. */
  highlightHeader?: boolean;
  /** 제목 아래 붙일 짧은 안내. */
  note?: string;
  /** 비어 있을 때 안내 문구. */
  emptyText: string;
}

/** 세 목록 가운데 하나. 제목과 인원을 보여 준다. */
function EntrantColumn({
  title,
  entrants,
  shuffling = false,
  numbered = false,
  highlightHeader = false,
  note,
  emptyText,
}: EntrantColumnProps) {
  return (
    <section
      className={styles.column}
      data-shuffling={shuffling ? "true" : undefined}
      aria-label={title}
      aria-busy={shuffling}
    >
      <div
        className={[
          styles.columnHeader,
          highlightHeader ? styles.columnWinnerHeader : "",
        ]
          .filter(Boolean)
          .join(" ")}
      >
        <h3 className={styles.columnTitle}>{title}</h3>
        <div className={styles.metaGroup}>
          {shuffling && (
            <StatusPill tone={highlightHeader ? "neutral" : "primary"}>
              섞는 중
            </StatusPill>
          )}
          <span className={styles.columnCount}>{entrants.length}명</span>
        </div>
      </div>

      {note && <p className={styles.columnNote}>{note}</p>}

      {entrants.length === 0 ? (
        <p className={styles.emptyColumn}>{emptyText}</p>
      ) : numbered ? (
        <ol className={styles.numberedList}>
          {entrants.map((entrant) => (
            <li key={entrant.applicationId} className={styles.numberedItem}>
              {entrantLabel(entrant)}
            </li>
          ))}
        </ol>
      ) : (
        <ul className={styles.entrantList}>
          {entrants.map((entrant) => (
            <li
              key={entrant.applicationId}
              className={[
                styles.entrantItem,
                entrant.guaranteed ? styles.guaranteed : "",
              ]
                .filter(Boolean)
                .join(" ")}
            >
              <span>{entrantLabel(entrant)}</span>
              {entrant.guaranteed && (
                <StatusPill tone="primary">우선</StatusPill>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
