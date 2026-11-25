import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  abortHandover,
  handoverFile,
  saveHandoverFile,
} from "../../api/handover";
import formStyles from "../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";
import { fileLines } from "./handoverData";

/**
 * 호스트를 넘기는 중인 PC의 화면(BR-H1). 서버가 꺼져 있어 다른 PC는 쓸 수 없다.
 *
 * 켤 때 만든 넘기기 파일을 저장 창에서 고른 곳(USB 등)에 저장한다. 저장하면 앱이 다시 켜지며
 * "넘김" 화면이 된다. 넘기지 않고 돌아가면 다시 호스트로 켠다.
 */
export function HandingOverPage() {
  const [notice, setNotice] = useState<string | null>(null);
  const file = useQuery({
    queryKey: ["handoverFile"],
    queryFn: handoverFile,
    retry: false,
    staleTime: Infinity,
  });
  const save = useMutation({
    mutationFn: saveHandoverFile,
    onSuccess: (saved) => {
      if (!saved) {
        setNotice(
          "저장하지 않았습니다. 다시 저장하거나 넘기지 않고 돌아가 주세요.",
        );
      }
    },
  });
  const abort = useMutation({ mutationFn: abortHandover });
  const busy = save.isPending || abort.isPending;
  const failure = save.error ?? abort.error;

  return (
    <div className={authStyles.page}>
      <section className={`${formStyles.form} ${authStyles.card}`}>
        <h2 className={formStyles.title}>호스트 넘기기</h2>
        <p className={authStyles.intro}>
          다른 PC는 지금 이 PC에 접속할 수 없습니다. 넘기기 파일을 USB 같은 곳에
          저장한 뒤, 새 호스트 PC에서 [넘겨받은 파일로 호스트 시작]을 눌러
          주세요.
        </p>
        {file.isPending && (
          <p className={authStyles.hint}>넘기기 파일을 확인하는 중…</p>
        )}
        {file.isError && (
          <p className={formStyles.formError} role="alert">
            {file.error.message}
          </p>
        )}
        {file.data !== undefined && (
          <ul className={authStyles.hint} aria-label="넘기기 파일">
            {fileLines(file.data).map((line) => (
              <li key={line}>{line}</li>
            ))}
          </ul>
        )}
        {failure !== null ? (
          <p className={formStyles.formError} role="alert">
            {failure.message}
          </p>
        ) : (
          notice !== null && (
            <p className={authStyles.hint} role="status">
              {notice}
            </p>
          )
        )}
        <div className={formStyles.actions}>
          <button
            type="button"
            className={formStyles.secondary}
            disabled={busy}
            onClick={() => {
              setNotice(null);
              abort.mutate();
            }}
          >
            {abort.isPending ? "돌아가는 중…" : "넘기지 않고 돌아가기"}
          </button>
          {file.data !== undefined && (
            <button
              type="button"
              className={formStyles.submit}
              disabled={busy}
              onClick={() => {
                setNotice(null);
                save.mutate();
              }}
            >
              {save.isPending ? "저장하는 중…" : "넘기기 파일 저장"}
            </button>
          )}
        </div>
      </section>
    </div>
  );
}
