import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";
import type { SubjectResponse } from "../../../api/generated/SubjectResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { Checkbox } from "../../../components/ui/Checkbox";
import { StatusPill } from "../../../components/ui/StatusPill";
import { TextField } from "../../../components/ui/TextField";
import {
  createPriorityReason,
  deletePriorityReason,
} from "../../workspace/priority-reasons/api";
import {
  priorityReasonFormErrorsFrom,
  type PriorityReasonFormErrors,
} from "../../workspace/priority-reasons/priorityReasonFormData";
import { createSubject, deleteSubject } from "../../workspace/subjects/api";
import {
  categoryNames,
  firstErrorField,
  subjectFormErrorsFrom,
  toSubjectRequest,
  type SubjectField,
  type SubjectFormErrors,
} from "../../workspace/subjects/subjectFormData";
import styles from "./SetupBasicsPage.module.css";

const SUBJECT_CATEGORY_LIST_ID = "setup-basics-categories";
// 민감정보에 해당하지 않는 기본 추천 사유만 둔다(BR-S3, BR-S15, docs/privacy-notice.md 제23조)
const PRESET_PRIORITY_REASON = "지난 학기 반장";

/** {@link SubjectsAndReasonsStep}의 속성. */
export interface SubjectsAndReasonsStepProps {
  subjects: readonly SubjectResponse[];
  subjectsLoading: boolean;
  subjectsError: string | null;
  onRetrySubjects: () => void;
  reasons: readonly PriorityReasonResponse[];
  reasonsLoading: boolean;
  reasonsError: string | null;
  onRetryReasons: () => void;
}

/** 3단계 하위 화면 2: 과목(분류 유지 연속 입력)과 우선 배정 사유를 나란히 입력한다. */
export function SubjectsAndReasonsStep({
  subjects,
  subjectsLoading,
  subjectsError,
  onRetrySubjects,
  reasons,
  reasonsLoading,
  reasonsError,
  onRetryReasons,
}: SubjectsAndReasonsStepProps) {
  return (
    <div className={styles.subjectsGrid}>
      <SubjectsCard
        subjects={subjects}
        loading={subjectsLoading}
        error={subjectsError}
        onRetry={onRetrySubjects}
      />
      <PriorityReasonsCard
        reasons={reasons}
        loading={reasonsLoading}
        error={reasonsError}
        onRetry={onRetryReasons}
      />
    </div>
  );
}

interface SubjectsCardProps {
  subjects: readonly SubjectResponse[];
  loading: boolean;
  error: string | null;
  onRetry: () => void;
}

function SubjectsCard({
  subjects,
  loading,
  error,
  onRetry,
}: SubjectsCardProps) {
  const queryClient = useQueryClient();
  const [category, setCategory] = useState("");
  const [name, setName] = useState("");
  const [errors, setErrors] = useState<SubjectFormErrors>(NO_FORM_ERRORS);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const inputs = useRef<Partial<Record<SubjectField, HTMLInputElement | null>>>(
    {},
  );

  const showErrors = (failure: unknown) => {
    const next = subjectFormErrorsFrom(failure);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const add = useMutation({
    mutationFn: () => createSubject(toSubjectRequest({ category, name })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["subjects"] });
      setErrors(NO_FORM_ERRORS);
      // 분류는 유지하고 과목 이름만 비워 같은 분류의 과목을 연속 입력하기 쉽게 한다
      setName("");
      inputs.current.name?.focus();
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteSubject,
    onMutate: (id) => {
      setDeletingId(id);
      setErrors(NO_FORM_ERRORS);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["subjects"] });
    },
    onError: showErrors,
    onSettled: () => setDeletingId(null),
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (add.isPending) {
      return;
    }
    add.mutate();
  };

  const columns: readonly DataTableColumn<SubjectResponse>[] = [
    {
      key: "category",
      header: "분류",
      width: "34%",
      render: (subject) => subject.category ?? "—",
    },
    {
      key: "name",
      header: "과목 이름",
      isRowHeader: true,
      render: (subject) => subject.name,
    },
    {
      key: "actions",
      header: "",
      align: "right",
      width: "96px",
      isAction: true,
      render: (subject) => (
        <Button
          size="sm"
          variant="danger"
          aria-label={`${subject.name} 빼기`}
          disabled={deletingId === subject.id}
          onClick={() => remove.mutate(subject.id)}
        >
          빼기
        </Button>
      ),
    },
  ];

  const categories = categoryNames(subjects);

  return (
    <Card
      title="과목"
      actions={
        <span className={styles.countBadge}>{subjects.length}개 등록됨</span>
      }
    >
      <div className={styles.cardStack}>
        <form className={styles.subjectForm} onSubmit={submit} noValidate>
          <TextField
            id="setup-subject-category"
            ref={(element) => {
              inputs.current.category = element;
            }}
            label="분류"
            list={SUBJECT_CATEGORY_LIST_ID}
            placeholder="예: 서예·미술"
            value={category}
            error={errors.fields.category}
            onChange={(event) => {
              setCategory(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, category: undefined },
              }));
            }}
          />
          <datalist id={SUBJECT_CATEGORY_LIST_ID}>
            {categories.map((item) => (
              <option key={item} value={item} />
            ))}
          </datalist>

          <TextField
            id="setup-subject-name"
            ref={(element) => {
              inputs.current.name = element;
            }}
            label="과목 이름"
            required
            placeholder="예: 서예, 스마트폰"
            value={name}
            error={errors.fields.name}
            onChange={(event) => {
              setName(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, name: undefined },
              }));
            }}
          />

          <div className={styles.addButtonWrap}>
            <Button
              type="submit"
              variant="primary"
              busy={add.isPending}
              busyLabel="더하는 중…"
            >
              더하기
            </Button>
          </div>
        </form>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}

        <div className={styles.tableScroll}>
          <DataTable
            caption="등록한 과목 목록"
            columns={columns}
            rows={subjects}
            getRowKey={(subject) => subject.id}
            loading={loading}
            loadingText="과목을 불러오는 중…"
            error={error}
            onRetry={onRetry}
            emptyMessage="아직 등록한 과목이 없습니다. 위에서 과목 이름을 넣어 더해 주세요."
          />
        </div>
      </div>
    </Card>
  );
}

interface PriorityReasonsCardProps {
  reasons: readonly PriorityReasonResponse[];
  loading: boolean;
  error: string | null;
  onRetry: () => void;
}

function PriorityReasonsCard({
  reasons,
  loading,
  error,
  onRetry,
}: PriorityReasonsCardProps) {
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [errors, setErrors] =
    useState<PriorityReasonFormErrors>(NO_FORM_ERRORS);
  const [togglingPreset, setTogglingPreset] = useState(false);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const nameInput = useRef<HTMLInputElement | null>(null);

  const add = useMutation({
    mutationFn: (reasonName: string) =>
      createPriorityReason({ name: reasonName, method: "guaranteed" }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
      setErrors(NO_FORM_ERRORS);
      setName("");
      nameInput.current?.focus();
    },
    onError: (failure) => {
      setErrors(priorityReasonFormErrorsFrom(failure));
      nameInput.current?.focus();
    },
  });

  const togglePreset = useMutation({
    mutationFn: async ({
      existing,
      checked,
    }: {
      existing: PriorityReasonResponse | undefined;
      checked: boolean;
    }) => {
      if (checked && !existing) {
        await createPriorityReason({
          name: PRESET_PRIORITY_REASON,
          method: "guaranteed",
        });
      } else if (!checked && existing) {
        await deletePriorityReason(existing.id);
      }
    },
    onMutate: () => {
      setTogglingPreset(true);
      setErrors(NO_FORM_ERRORS);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
    },
    onError: (failure) => setErrors(priorityReasonFormErrorsFrom(failure)),
    onSettled: () => setTogglingPreset(false),
  });

  const remove = useMutation({
    mutationFn: deletePriorityReason,
    onMutate: (id) => {
      setDeletingId(id);
      setErrors(NO_FORM_ERRORS);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
    },
    onError: (failure) => setErrors(priorityReasonFormErrorsFrom(failure)),
    onSettled: () => setDeletingId(null),
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (add.isPending) {
      return;
    }
    add.mutate(name);
  };

  const presetExisting = reasons.find(
    (reason) => reason.name === PRESET_PRIORITY_REASON,
  );
  const presetChecked = presetExisting !== undefined;
  const customReasons = reasons.filter(
    (reason) => reason.name !== PRESET_PRIORITY_REASON,
  );

  return (
    <Card
      title="우선 배정 사유"
      actions={
        <span className={styles.countBadge}>{reasons.length}개 선택됨</span>
      }
    >
      <div className={styles.cardStack}>
        <form className={styles.reasonForm} onSubmit={submit} noValidate>
          <TextField
            id="setup-priority-reason-name"
            ref={nameInput}
            label="사유 이름"
            placeholder="예: 다자녀, 봉사자"
            value={name}
            error={errors.fields.name}
            onChange={(event) => {
              setName(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, name: undefined },
              }));
            }}
          />
          <div className={styles.addButtonWrap}>
            <Button
              type="submit"
              variant="primary"
              busy={add.isPending}
              busyLabel="더하는 중…"
            >
              더하기
            </Button>
          </div>
        </form>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}

        {error ? (
          <NoticeBanner
            tone="error"
            message={error}
            actionLabel="다시 해 보기"
            onAction={onRetry}
          />
        ) : loading ? (
          <NoticeBanner message="우선 배정 사유를 불러오는 중…" />
        ) : (
          <>
            <div
              className={[
                styles.presetRow,
                presetChecked ? styles.presetRowChecked : "",
              ]
                .filter(Boolean)
                .join(" ")}
            >
              <Checkbox
                label={PRESET_PRIORITY_REASON}
                checked={presetChecked}
                disabled={togglingPreset}
                onChange={(event) =>
                  togglePreset.mutate({
                    existing: presetExisting,
                    checked: event.target.checked,
                  })
                }
              />
              <StatusPill tone={presetChecked ? "primary" : "neutral"}>
                자리 보장
              </StatusPill>
            </div>

            {customReasons.length > 0 && (
              <ul
                className={styles.customReasonList}
                aria-label="직접 추가한 우선 배정 사유"
              >
                {customReasons.map((reason) => (
                  <li key={reason.id} className={styles.customReasonRow}>
                    <div className={styles.customReasonMain}>
                      <span className={styles.customReasonName}>
                        {reason.name}
                      </span>
                      <StatusPill tone="primary">자리 보장</StatusPill>
                    </div>
                    <Button
                      size="sm"
                      variant="danger"
                      aria-label={`${reason.name} 빼기`}
                      disabled={deletingId === reason.id}
                      onClick={() => remove.mutate(reason.id)}
                    >
                      빼기
                    </Button>
                  </li>
                ))}
              </ul>
            )}
          </>
        )}

        <p className={styles.reasonHint}>
          추첨 없이 자리를 보장할 대상을 고르거나 위에서 직접 추가하세요. 민감한
          사유(보훈·장애 등)는 기관 개인정보 안내·동의 절차를 확인한 뒤 직접
          추가하세요.
        </p>
      </div>
    </Card>
  );
}
