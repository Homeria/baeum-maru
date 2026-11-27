import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { SplitAuthLayout } from "../../../components/shell/SplitAuthLayout";
import { Card } from "../../../components/surface/Card";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import authStyles from "../auth/auth.module.css";
import { abortHandover, handoverFile, saveHandoverFile } from "./api";
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
    <SplitAuthLayout
      panelTitle="호스트 넘기기"
      panelDescription="다른 PC는 지금 이 PC에 접속할 수 없습니다. 넘기기 파일을 USB에 저장해 새 호스트 PC로 옮겨 주세요."
      cardTitle="호스트 넘기기"
      cardSubtitle="넘기기 파일을 저장한 뒤 새 호스트 PC에서 [넘겨받은 파일로 호스트 시작]을 눌러 주세요."
    >
      <div className={authStyles.stack}>
        {file.isPending && (
          <NoticeBanner message="넘기기 파일을 확인하는 중…" />
        )}
        {file.isError && (
          <NoticeBanner tone="error" message={file.error.message} />
        )}
        {file.data !== undefined && (
          <Card headingLevel={3} title="넘기기 파일 내용">
            <ul className={authStyles.steps} aria-label="넘기기 파일">
              {fileLines(file.data).map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          </Card>
        )}

        {failure !== null ? (
          <NoticeBanner tone="error" message={failure.message} />
        ) : (
          notice !== null && <NoticeBanner message={notice} />
        )}

        <div className={authStyles.actionsBetween}>
          <Button
            variant="secondary"
            disabled={busy}
            busy={abort.isPending}
            busyLabel="돌아가는 중…"
            onClick={() => {
              setNotice(null);
              abort.mutate();
            }}
          >
            넘기지 않고 돌아가기
          </Button>
          {file.data !== undefined && (
            <Button
              variant="primary"
              disabled={busy}
              busy={save.isPending}
              busyLabel="저장하는 중…"
              onClick={() => {
                setNotice(null);
                save.mutate();
              }}
            >
              넘기기 파일 저장
            </Button>
          )}
        </div>
      </div>
    </SplitAuthLayout>
  );
}
