import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { closeSelection, getLottery, runLottery } from "./api";
import type { CourseLotteryResponse } from "../../../api/generated/CourseLotteryResponse";
import type { EntrantResponse } from "../../../api/generated/EntrantResponse";
import { ConfirmDialog } from "../../../components/dialog/ConfirmDialog";
import formStyles from "../../../components/form/form.module.css";
import { STAGE_LABELS } from "../courses/courseStatusData";
import {
  SHUFFLE_INTERVAL_MS,
  drawPool,
  entrantLabel,
  guaranteedOf,
  lotteryButtons,
  needsShuffle,
  shuffled,
  resultColumns,
} from "./selectionData";
import styles from "./SelectionPage.module.css";

/** {@link LotteryBoard}의 속성. */
export interface LotteryBoardProps {
  termId: number;
  courseId: number;
  /** 화면 위에 보여 줄 강좌 이름. 예: 요가 초급 */
  courseName: string;
}

/**
 * 강좌 하나의 추첨 화면. 신청자, 당첨자, 대기자 세 목록이다.
 *
 * 시작을 누르면 신청자 목록을 0.25초마다 섞어 보여 준다. 멈춤을 누르면 호스트가 그 순간
 * 순서를 정하고(ADR 0016), 신청자 목록이 그 순서로 멈춘 뒤 앞에서 정원만큼 당첨, 나머지는
 * 그 순서대로 대기가 된다(BR-S7). 화면은 결과를 계산하지 않는다.
 *
 * 자리 보장 신청(BR-S4)은 추첨하지 않아 신청자 목록에서 빠지고, 당첨자 목록 맨 위에 늘
 * 고정된다. 옅은 배경과 "우선" 글자로 표시하고 사유 이름은 보이지 않는다(BR-S15).
 *
 * 추첨·재추첨·마감은 호스트 PC에서 관리자·직원만 한다(BR-L6). 다른 PC와 역할은 결과만 본다.
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
    return <p className={formStyles.loading}>추첨 화면을 불러오는 중입니다…</p>;
  }
  if (lottery.isError) {
    return (
      <p className={formStyles.formError} role="alert">
        {lottery.error.message}
      </p>
    );
  }

  const data = lottery.data;
  /** 시작: 지금 순서부터 섞기 시작한다. 섞을 까닭이 없으면 바로 뽑는다. */
  const start = (current: CourseLotteryResponse) => {
    setError(null);
    if (needsShuffle(current)) {
      setShown(drawPool(current.entrants));
    } else {
      draw.mutate();
    }
  };
  const applicants = shown ?? drawPool(data.entrants);
  const guaranteedCount = guaranteedOf(data.entrants).length;
  // 섞는 동안에는 결과를 비운다 (재추첨 포함)
  const result = resultColumns(data, shuffling);
  const buttons = lotteryButtons(data);

  return (
    <section className={styles.board} aria-label={`${courseName} 추첨`}>
      <header className={styles.boardHeader}>
        <h2 className={styles.courseTitle}>{courseName}</h2>
        <span className={styles.meta}>
          정원 {data.capacity}명 · {STAGE_LABELS[data.stage]}
        </span>
      </header>

      <div className={styles.columns}>
        <EntrantColumn
          title="신청자"
          entrants={applicants}
          shuffling={shuffling}
          note={
            guaranteedCount > 0
              ? `우선 배정 ${guaranteedCount}명은 추첨 없이 당첨자에 있습니다`
              : undefined
          }
        />
        <EntrantColumn title="당첨자" entrants={result.selected} />
        <EntrantColumn title="대기자" entrants={result.waitlisted} numbered />
      </div>

      {error && (
        <p className={formStyles.formError} role="alert">
          {error}
        </p>
      )}

      <div className={styles.controls}>
        {buttons === "start" && (
          <button
            type="button"
            className={`${formStyles.submit} ${styles.drawButton}`}
            disabled={draw.isPending}
            onClick={() => (shuffling ? draw.mutate() : start(data))}
          >
            {draw.isPending ? "뽑는 중…" : shuffling ? "멈춤" : "시작"}
          </button>
        )}
        {buttons === "selectAll" && (
          <>
            <p className={styles.hint}>
              {guaranteedCount > 0
                ? "추첨할 신청자가 남은 자리 이하라 추첨 없이 모두 당첨입니다."
                : "신청자가 정원 이하라 추첨 없이 모두 당첨입니다."}
            </p>
            <button
              type="button"
              className={`${formStyles.submit} ${styles.drawButton}`}
              disabled={draw.isPending}
              onClick={() => start(data)}
            >
              {draw.isPending ? "처리하는 중…" : "전원 당첨"}
            </button>
          </>
        )}
        {buttons === "redraw" &&
          (shuffling ? (
            <button
              type="button"
              className={`${formStyles.submit} ${styles.drawButton}`}
              disabled={draw.isPending}
              onClick={() => draw.mutate()}
            >
              {draw.isPending ? "뽑는 중…" : "멈춤"}
            </button>
          ) : (
            <>
              <button
                type="button"
                className={formStyles.secondary}
                onClick={() => setAsking("redraw")}
              >
                다시 추첨
              </button>
              <button
                type="button"
                className={formStyles.submit}
                onClick={() => setAsking("close")}
              >
                선발 마감
              </button>
            </>
          ))}
        {buttons === "overCapacity" && (
          <p className={formStyles.formError} role="alert">
            우선 배정 신청 {guaranteedCount}명이 정원 {data.capacity}명보다 많아
            추첨할 수 없습니다. 신청 탭에서 우선 배정을 줄이거나 강좌 탭에서
            정원을 늘려 주세요.
          </p>
        )}
        {buttons === "none" && <p className={styles.hint}>{noneHint(data)}</p>}
      </div>

      {asking === "redraw" && (
        <ConfirmDialog
          title={`${courseName}을(를) 다시 추첨할까요?`}
          warning="지금 결과는 취소됨으로 기록에 남고, 신청자 모두를 처음부터 다시 뽑습니다."
          confirmLabel="다시 추첨"
          busyLabel="처리하는 중…"
          busy={draw.isPending}
          onConfirm={() => {
            setAsking(null);
            start(data);
          }}
          onCancel={() => setAsking(null)}
        />
      )}
      {asking === "close" && (
        <ConfirmDialog
          title={`${courseName}의 선발을 마감할까요?`}
          warning="마감하면 다시 추첨할 수 없습니다."
          confirmLabel="선발 마감"
          busyLabel="마감하는 중…"
          busy={close.isPending}
          onConfirm={() => close.mutate()}
          onCancel={() => setAsking(null)}
        />
      )}
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
  /** 제목 아래 붙일 짧은 안내. */
  note?: string;
}

/** 세 목록 가운데 하나. 제목에 인원을 붙인다. */
function EntrantColumn({
  title,
  entrants,
  shuffling = false,
  numbered = false,
  note,
}: EntrantColumnProps) {
  const List = numbered ? "ol" : "ul";
  return (
    <section
      className={styles.column}
      data-shuffling={shuffling ? "true" : undefined}
      aria-label={title}
      aria-busy={shuffling}
    >
      <h3 className={styles.columnTitle}>
        {title} ({entrants.length}명)
        {shuffling && <span className={styles.shuffling}> 섞는 중</span>}
      </h3>
      {note && <p className={styles.columnNote}>{note}</p>}
      <List className={numbered ? styles.numberedList : styles.entrantList}>
        {entrants.map((entrant) => (
          <li
            key={entrant.applicationId}
            className={entrant.guaranteed ? styles.guaranteed : undefined}
          >
            {entrantLabel(entrant)}
            {entrant.guaranteed && (
              <span className={styles.guaranteedTag}>우선</span>
            )}
          </li>
        ))}
      </List>
    </section>
  );
}
