import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import type { TakeOverPreviewResponse } from "../../api/generated/TakeOverPreviewResponse";
import {
  pickHandoverFile,
  previewTakeOver,
  takeOver,
} from "../../api/handover";
import formStyles from "../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";
import { AdminPinFields } from "../auth/AdminPinFields";
import { fileLines } from "./handoverData";

/** {@link TakeOverPage}의 속성. */
export interface TakeOverPageProps {
  /** 넘겨받지 않고 앞 화면으로 돌아간다. */
  onBack: () => void;
}

/**
 * 넘겨받은 파일로 이 PC를 새 호스트로 시작한다(BR-H3).
 *
 * 넘기기 파일을 고르면 학기와 수를 보여 주고, 그 파일 안의 관리자를 골라 PIN을 넣는다.
 * 시작하면 앱이 다시 켜지며 이 PC가 호스트가 된다.
 */
export function TakeOverPage({ onBack }: TakeOverPageProps) {
  const [picked, setPicked] = useState<{
    path: string;
    preview: TakeOverPreviewResponse;
  } | null>(null);
  const [staffId, setStaffId] = useState<number | null>(null);
  const [pin, setPin] = useState("");
  const [missing, setMissing] = useState(false);

  const pick = useMutation({
    mutationFn: async () => {
      const path = await pickHandoverFile();
      return path === null
        ? null
        : { path, preview: await previewTakeOver(path) };
    },
    onSuccess: (result) => {
      if (result !== null) {
        setPicked(result);
        setStaffId(null);
        setPin("");
        setMissing(false);
      }
    },
  });
  const start = useMutation({
    mutationFn: (variables: { path: string; staffId: number }) =>
      takeOver(variables.path, variables.staffId, pin),
    // 틀린 PIN은 지우고 다시 넣게
    onError: () => setPin(""),
  });
  const busy = pick.isPending || start.isPending;
  const failure = pick.error ?? start.error;

  return (
    <div className={authStyles.page}>
      <section className={`${formStyles.form} ${authStyles.card}`}>
        <h2 className={formStyles.title}>넘겨받은 파일로 호스트 시작</h2>
        <p className={authStyles.intro}>
          옛 호스트 PC에서 [호스트 넘기기]로 저장한 넘기기 파일을 골라 주세요.
          이 PC가 새 호스트가 되고, 다른 PC들은 새 주소만 넣으면 다시 승인받지
          않고 이어집니다.
        </p>
        <p className={authStyles.hint}>
          옛 호스트 PC가 넘기지 않은 채 켜져 있으면 호스트가 둘이 되어 접수
          명단이 갈라집니다. 이 PC에 남은 예전 데이터는 지우지 않고 백업으로
          남깁니다.
        </p>

        {picked !== null && (
          <>
            <ul className={authStyles.hint} aria-label="넘기기 파일">
              {fileLines(picked.preview).map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
            <AdminPinFields
              idPrefix="take-over"
              admins={picked.preview.admins}
              staffId={staffId}
              onStaffId={(id) => {
                setStaffId(id);
                setMissing(false);
              }}
              pin={pin}
              onPin={(value) => {
                setPin(value);
                setMissing(false);
              }}
              missing={missing}
            />
          </>
        )}

        {failure !== null && (
          <p className={formStyles.formError} role="alert">
            {failure.message}
          </p>
        )}
        <div className={formStyles.actions}>
          <button
            type="button"
            className={formStyles.secondary}
            disabled={busy}
            onClick={onBack}
          >
            돌아가기
          </button>
          <button
            type="button"
            className={
              picked === null ? formStyles.submit : formStyles.secondary
            }
            disabled={busy}
            onClick={() => pick.mutate()}
          >
            {pick.isPending
              ? "파일을 살펴보는 중…"
              : picked === null
                ? "넘기기 파일 고르기"
                : "다른 파일 고르기"}
          </button>
          {picked !== null && (
            <button
              type="button"
              className={formStyles.submit}
              disabled={busy}
              onClick={() => {
                if (staffId === null || pin === "") {
                  setMissing(true);
                  return;
                }
                start.mutate({ path: picked.path, staffId });
              }}
            >
              {start.isPending ? "시작하는 중…" : "이 파일로 호스트 시작"}
            </button>
          )}
        </div>
      </section>
    </div>
  );
}
