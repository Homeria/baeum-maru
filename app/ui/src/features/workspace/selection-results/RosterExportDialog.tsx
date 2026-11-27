import { useRef, useState } from "react";
import type { RosterColumnCode } from "../../../api/generated/RosterColumnCode";
import type { RosterKindCode } from "../../../api/generated/RosterKindCode";
import { saveFile } from "../../../api/saveFile";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { Checkbox } from "../../../components/ui/Checkbox";
import { Radio } from "../../../components/ui/Radio";
import {
  COLUMN_LABELS,
  KIND_RULES,
  MASK_CHAR_LABELS,
  MASK_CHARS,
  ROSTER_KINDS,
  ROSTER_KIND_LABELS,
  exportRequest,
  isIncluded,
  rememberChoices,
  rememberedChoices,
  rememberedKind,
  rosterFileName,
  toggle,
  type RosterChoices,
} from "./rosterChoices";
import { exportRoster } from "./rostersApi";
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
 * 결과 명단 내보내기 모달 대화상자(BR-P1~P9).
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
  const kinds = ROSTER_KINDS.filter((kind) => canExportRaw || kind !== "raw");
  const [kind, setKind] = useState(() => rememberedKind(localStorage, kinds));
  const [choices, setChoices] = useState<RosterChoices>(() =>
    rememberedChoices(localStorage, kind),
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const exportButtonRef = useRef<HTMLButtonElement>(null);

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
    <ModalDialog
      open
      size="md"
      title="결과 명단 내보내기"
      busy={busy}
      initialFocusRef={exportButtonRef}
      onClose={onCancel}
      footer={
        <>
          <Button variant="secondary" disabled={busy} onClick={onCancel}>
            그만두기
          </Button>
          <Button
            ref={exportButtonRef}
            variant="primary"
            busy={busy}
            busyLabel="만드는 중…"
            onClick={() => void submit()}
          >
            내보내기
          </Button>
        </>
      }
    >
      <div className={styles.dialogBody}>
        <p className={styles.dialogDesc}>
          {termName}에서 선발을 마감한 강좌가 들어갑니다. 포기한 신청은 넣지
          않고, 자리 보장 당첨은 색칠합니다.
        </p>

        <fieldset className={styles.exportGroup} disabled={busy}>
          <legend className={styles.exportLegend}>명단 종류</legend>
          <div className={styles.exportStack}>
            {kinds.map((candidate) => (
              <Radio
                key={candidate}
                name="roster-kind"
                label={ROSTER_KIND_LABELS[candidate]}
                description={KIND_HINTS[candidate]}
                checked={kind === candidate}
                onChange={() => chooseKind(candidate)}
              />
            ))}
          </div>
        </fieldset>

        <fieldset className={styles.exportGroup} disabled={busy}>
          <legend className={styles.exportLegend}>더 넣을 칸</legend>
          <div className={styles.exportChoices}>
            {rule.optional.map((column) => (
              <Checkbox
                key={column}
                label={
                  rule.alwaysMasked.includes(column)
                    ? `${COLUMN_LABELS[column]} (늘 가림)`
                    : COLUMN_LABELS[column]
                }
                checked={choices.include.includes(column)}
                onChange={(event) => setIncluded(column, event.target.checked)}
              />
            ))}
          </div>
        </fieldset>

        <fieldset className={styles.exportGroup} disabled={busy}>
          <legend className={styles.exportLegend}>가릴 칸</legend>
          <div className={styles.exportChoices}>
            {rule.maskable.map((column) => (
              <Checkbox
                key={column}
                label={COLUMN_LABELS[column]}
                checked={choices.mask.includes(column)}
                disabled={!isIncluded(kind, choices, column)}
                onChange={(event) => setMasked(column, event.target.checked)}
              />
            ))}
          </div>
          <div className={styles.exportChoices}>
            <span className={styles.exportLegend}>가림 글자</span>
            {MASK_CHARS.map((candidate) => (
              <Radio
                key={candidate}
                name="mask-char"
                label={MASK_CHAR_LABELS[candidate]}
                checked={choices.maskChar === candidate}
                onChange={() => setChoices({ ...choices, maskChar: candidate })}
              />
            ))}
          </div>
          <p className={styles.exportHint}>
            예: 홍길동 → 홍{MASK_CHAR_LABELS[choices.maskChar]}동
          </p>
        </fieldset>

        {error && <NoticeBanner tone="error" message={error} />}
      </div>
    </ModalDialog>
  );
}
