import { useEffect, useId, useRef, useState } from "react";
import type { RosterColumnCode } from "../../api/generated/RosterColumnCode";
import type { RosterKindCode } from "../../api/generated/RosterKindCode";
import { exportRoster } from "../../api/rosters";
import { saveFile } from "../../api/saveFile";
import dialogStyles from "../../components/dialog/ConfirmDialog.module.css";
import formStyles from "../../components/form/form.module.css";
import {
  COLUMN_LABELS,
  exportRequest,
  isIncluded,
  KIND_RULES,
  MASK_CHAR_LABELS,
  MASK_CHARS,
  rememberChoices,
  rememberedChoices,
  rememberedKind,
  ROSTER_KIND_LABELS,
  ROSTER_KINDS,
  rosterFileName,
  toggle,
  type RosterChoices,
} from "./rosterChoices";
import styles from "./SelectionResultsPage.module.css";

/** 명단 종류마다 모양 설명(BR-P2~P4). */
const KIND_HINTS: Record<RosterKindCode, string> = {
  raw: "강좌마다 시트 하나. 당첨자와 대기자를 나란히 두고 대기 순번을 붙입니다.",
  posted_by_course:
    "강좌마다 시트 하나. 큰 글씨로 당첨자 다음에 대기자를 순번대로 둡니다.",
  posted_by_member:
    "회원 한 명이 한 줄(가나다순). 강좌마다 결과 칸을 따로 둡니다.",
};

/** {@link RosterExportDialog}의 속성. */
export interface RosterExportDialogProps {
  termId: number;
  /** 파일 이름에 쓴다. 예: 2027년 1학기 */
  termName: string;
  /** 원본 명단을 내보낼 수 있는 역할인지(BR-P10). 아니면 원본을 보여 주지 않는다. */
  canExportRaw: boolean;
  /** 저장했다. 저장한 파일 경로. */
  onSaved: (path: string) => void;
  onCancel: () => void;
}

/**
 * 결과 명단 내보내기 창(BR-P1~P9).
 *
 * 명단 종류를 고르고, 넣을 칸과 가릴 칸, 가림 글자를 고른다. 내보내면 호스트에서 파일을 받아
 * 이 PC의 저장 창을 띄운다(ADR 0018). 고른 것은 이 PC에 기억한다.
 */
export function RosterExportDialog({
  termId,
  termName,
  canExportRaw,
  onSaved,
  onCancel,
}: RosterExportDialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const kinds = ROSTER_KINDS.filter((kind) => canExportRaw || kind !== "raw");
  const [kind, setKind] = useState(() => rememberedKind(localStorage, kinds));
  const [choices, setChoices] = useState<RosterChoices>(() =>
    rememberedChoices(localStorage, kind),
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    dialog.current?.showModal();
  }, []);

  const rule = KIND_RULES[kind];
  const chooseKind = (next: RosterKindCode) => {
    setKind(next);
    setChoices(rememberedChoices(localStorage, next));
    setError(null);
  };
  const setIncluded = (column: RosterColumnCode, on: boolean) =>
    setChoices({
      ...choices,
      include: toggle(choices.include, column, on, rule.optional),
    });
  const setMasked = (column: RosterColumnCode, on: boolean) =>
    setChoices({
      ...choices,
      mask: toggle(choices.mask, column, on, rule.maskable),
    });

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      const contents = await exportRoster(termId, exportRequest(kind, choices));
      rememberChoices(localStorage, kind, choices);
      const path = await saveFile(rosterFileName(termName, kind), contents);
      if (path === null) {
        // 저장 창을 닫음. 이 창은 그대로 두어 다시 고를 수 있게
        setBusy(false);
        return;
      }
      onSaved(path);
    } catch (failure) {
      setError(
        failure instanceof Error
          ? failure.message
          : "명단을 내보내지 못했습니다",
      );
      setBusy(false);
    }
  };

  return (
    <dialog
      ref={dialog}
      className={`${dialogStyles.dialog} ${styles.exportDialog}`}
      aria-labelledby={titleId}
      onCancel={(event) => {
        event.preventDefault();
        if (!busy) {
          onCancel();
        }
      }}
    >
      <h2 id={titleId} className={dialogStyles.title}>
        결과 명단 내보내기
      </h2>
      <p className={dialogStyles.body}>
        {termName}에서 선발을 마감한 강좌가 들어갑니다. 포기한 신청은 넣지 않고,
        자리 보장 당첨은 색칠합니다.
      </p>

      <fieldset className={styles.exportGroup} disabled={busy}>
        <legend className={formStyles.label}>명단 종류</legend>
        <div className={formStyles.choices}>
          {kinds.map((candidate) => (
            <label key={candidate} className={formStyles.choice}>
              <input
                type="radio"
                name="roster-kind"
                checked={kind === candidate}
                onChange={() => chooseKind(candidate)}
              />
              {ROSTER_KIND_LABELS[candidate]}
            </label>
          ))}
        </div>
        <p className={styles.exportHint}>{KIND_HINTS[kind]}</p>
      </fieldset>

      <fieldset className={styles.exportGroup} disabled={busy}>
        <legend className={formStyles.label}>더 넣을 칸</legend>
        <div className={styles.exportChoices}>
          {rule.optional.map((column) => (
            <label key={column} className={formStyles.choice}>
              <input
                type="checkbox"
                checked={choices.include.includes(column)}
                onChange={(event) => setIncluded(column, event.target.checked)}
              />
              {COLUMN_LABELS[column]}
              {rule.alwaysMasked.includes(column) && (
                <span className={styles.small}>(늘 가림)</span>
              )}
            </label>
          ))}
        </div>
      </fieldset>

      <fieldset className={styles.exportGroup} disabled={busy}>
        <legend className={formStyles.label}>가릴 칸</legend>
        <div className={styles.exportChoices}>
          {rule.maskable.map((column) => (
            <label key={column} className={formStyles.choice}>
              <input
                type="checkbox"
                checked={choices.mask.includes(column)}
                disabled={!isIncluded(kind, choices, column)}
                onChange={(event) => setMasked(column, event.target.checked)}
              />
              {COLUMN_LABELS[column]}
            </label>
          ))}
        </div>
        <div className={formStyles.choices}>
          <span>가림 글자</span>
          {MASK_CHARS.map((candidate) => (
            <label key={candidate} className={formStyles.choice}>
              <input
                type="radio"
                name="mask-char"
                checked={choices.maskChar === candidate}
                onChange={() => setChoices({ ...choices, maskChar: candidate })}
              />
              {MASK_CHAR_LABELS[candidate]}
            </label>
          ))}
        </div>
        <p className={styles.exportHint}>
          예: 홍길동 → 홍{MASK_CHAR_LABELS[choices.maskChar]}동
        </p>
      </fieldset>

      {error && (
        <p className={formStyles.formError} role="alert">
          {error}
        </p>
      )}

      <div className={dialogStyles.actions}>
        <button
          type="button"
          className={dialogStyles.cancel}
          onClick={onCancel}
          disabled={busy}
        >
          그만두기
        </button>
        <button
          type="button"
          className={formStyles.submit}
          disabled={busy}
          onClick={() => void submit()}
          autoFocus
        >
          {busy ? "만드는 중…" : "내보내기"}
        </button>
      </div>
    </dialog>
  );
}
