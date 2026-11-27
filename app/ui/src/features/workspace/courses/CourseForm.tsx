import { useId, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { RoomResponse } from "../../../api/generated/RoomResponse";
import type { SubjectResponse } from "../../../api/generated/SubjectResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { Drawer } from "../../../components/overlay/Drawer";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { SelectField } from "../../../components/ui/SelectField";
import { StatusPill } from "../../../components/ui/StatusPill";
import { TextField } from "../../../components/ui/TextField";
import { createCourse, deleteCourse, updateCourse } from "./api";
import {
  EMPTY_COURSE_FORM,
  WEEKDAYS,
  addScheduleRow,
  courseFormErrorsFrom,
  courseFormValuesFrom,
  courseName,
  firstErrorField,
  roomLabel,
  timeRangeLabel,
  toCourseRequest,
  type CourseField,
  type CourseFormErrors,
  type CourseFormValues,
  type ScheduleRowValues,
  type TimeRange,
} from "./courseFormData";
import styles from "./CourseForm.module.css";

/** {@link CourseForm}의 속성. */
export interface CourseFormProps {
  /** 새 강좌를 넣을 학기. */
  termId: number;
  /** 고를 수 있는 과목. */
  subjects: readonly SubjectResponse[];
  /** 고를 수 있는 강의실. */
  rooms: readonly RoomResponse[];
  /** 강의실 이름에 건물을 붙일지. */
  showBuilding: boolean;
  /** 이 학기 강좌들이 이미 쓰는 시간대. 줄마다 골라 시각을 채운다. 비었으면 고르는 칸을 숨긴다. */
  usedTimes: readonly TimeRange[];
  /** 고칠 강좌. 없으면 새 강좌 등록. */
  editing?: CourseResponse;
  /** 저장이나 등록을 마친 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onSaved: (notice: string) => void;
  /** 강좌를 지운 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onDeleted?: (notice: string) => void;
  /** 저장하지 않고 서랍을 닫는다. */
  onClose: () => void;
}

function schedulesEqual(
  a: readonly ScheduleRowValues[],
  b: readonly ScheduleRowValues[],
): boolean {
  if (a.length !== b.length) {
    return false;
  }
  return a.every(
    (row, index) =>
      row.weekday === b[index].weekday &&
      row.start === b[index].start &&
      row.end === b[index].end &&
      row.roomId === b[index].roomId,
  );
}

/**
 * 강좌 등록과 고치기를 맡는 오른쪽 편집 서랍.
 *
 * 수업 시간은 줄마다 요일, 시작·종료 시각, 강의실(선택)을 넣는다. 줄을 더하면 앞 줄의 시각과
 * 강의실을 그대로 쓴다.
 */
export function CourseForm({
  termId,
  subjects,
  rooms,
  showBuilding,
  usedTimes,
  editing,
  onSaved,
  onDeleted,
  onClose,
}: CourseFormProps) {
  const formId = useId();
  const queryClient = useQueryClient();
  const initialValues = editing
    ? courseFormValuesFrom(editing)
    : EMPTY_COURSE_FORM;
  const [values, setValues] = useState<CourseFormValues>(() => initialValues);
  const [errors, setErrors] = useState<CourseFormErrors>(NO_FORM_ERRORS);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const cancelDeleteRef = useRef<HTMLButtonElement>(null);
  // 수업 시간 칸은 첫 줄의 요일에 커서를 둔다
  const inputs = useRef<Partial<Record<CourseField, HTMLElement | null>>>({});

  const isDirty =
    values.subjectId !== initialValues.subjectId ||
    values.level !== initialValues.level ||
    values.section !== initialValues.section ||
    values.capacity !== initialValues.capacity ||
    !schedulesEqual(values.schedules, initialValues.schedules);

  const showErrors = (error: unknown) => {
    const next = courseFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  /** 저장한 강좌의 이름. 등록은 응답에 이름이 없어 고른 과목에서 만든다. */
  const nameOf = (nextValues: CourseFormValues) =>
    courseName({
      subjectName:
        subjects.find((subject) => String(subject.id) === nextValues.subjectId)
          ?.name ?? "",
      level: nextValues.level.trim(),
      section: nextValues.section.trim(),
    });

  const save = useMutation({
    mutationFn: (nextValues: CourseFormValues) => {
      const request = toCourseRequest(nextValues);
      return editing
        ? updateCourse(editing.id, request).then(courseName)
        : createCourse(termId, request).then(() => nameOf(nextValues));
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["courses", termId] });
      setErrors(NO_FORM_ERRORS);
      onSaved(editing ? `저장했습니다: ${name}` : `등록했습니다: ${name}`);
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteCourse,
    onSuccess: () => {
      setConfirmingDelete(false);
      void queryClient.invalidateQueries({ queryKey: ["courses", termId] });
      (onDeleted ?? onSaved)(
        `지웠습니다: ${editing ? courseName(editing) : ""}`,
      );
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  const change = (patch: Partial<CourseFormValues>, field: CourseField) => {
    setValues((current) => ({ ...current, ...patch }));
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
  };

  const changeRow = (key: number, patch: Partial<ScheduleRowValues>) =>
    change(
      {
        schedules: values.schedules.map((row) =>
          row.key === key ? { ...row, ...patch } : row,
        ),
      },
      "schedules",
    );

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    save.mutate(values);
  };

  const scheduleError = errors.fields.schedules;

  return (
    <Drawer
      open
      title={editing ? "강좌 고치기" : "새 강좌 등록"}
      subtitle={
        editing
          ? courseName(editing)
          : "과목, 정원, 수업 시간을 입력한 뒤 등록하세요"
      }
      isDirty={isDirty}
      busy={busy}
      onClose={onClose}
      cancelLabel="그만두기"
      footerLeft={
        editing ? (
          <Button
            variant="danger"
            disabled={busy}
            onClick={() => setConfirmingDelete(true)}
          >
            지우기
          </Button>
        ) : undefined
      }
      footerRight={
        <Button
          type="submit"
          form={formId}
          variant="primary"
          busy={save.isPending}
          busyLabel={editing ? "저장하는 중…" : "등록하는 중…"}
          disabled={remove.isPending}
        >
          {editing ? "저장" : "등록"}
        </Button>
      }
    >
      <form id={formId} className={styles.form} onSubmit={submit} noValidate>
        <div>
          <SelectField
            ref={(element) => {
              inputs.current.subjectId = element;
            }}
            label="과목"
            required
            value={values.subjectId}
            onChange={(event) =>
              change({ subjectId: event.target.value }, "subjectId")
            }
            error={errors.fields.subjectId}
          >
            <option value="">고르세요</option>
            {subjects.map((subject) => (
              <option key={subject.id} value={subject.id}>
                {subject.name}
              </option>
            ))}
          </SelectField>
          {subjects.length === 0 && (
            <p className={styles.empty}>
              과목이 없습니다. 기초 정보 탭에서 과목을 먼저 등록해 주세요.
            </p>
          )}
        </div>

        <TextField
          ref={(element) => {
            inputs.current.level = element;
          }}
          label="수준"
          autoComplete="off"
          placeholder="예: 초급, 중급"
          value={values.level}
          onChange={(event) => change({ level: event.target.value }, "level")}
          error={errors.fields.level}
        />

        <TextField
          ref={(element) => {
            inputs.current.section = element;
          }}
          label="반"
          autoComplete="off"
          placeholder="예: 1반, 오전반"
          hint="같은 과목을 여러 강좌로 열 때 수준이나 반으로 나눕니다. 하나만 열면 비워 두세요."
          value={values.section}
          onChange={(event) =>
            change({ section: event.target.value }, "section")
          }
          error={errors.fields.section}
        />

        <TextField
          ref={(element) => {
            inputs.current.capacity = element;
          }}
          label="정원"
          required
          type="number"
          inputMode="numeric"
          min={1}
          max={200}
          placeholder="예: 20"
          hint="명 단위로 숫자만 입력하세요."
          value={values.capacity}
          onChange={(event) =>
            change({ capacity: event.target.value }, "capacity")
          }
          error={errors.fields.capacity}
        />

        <fieldset
          className={styles.fieldset}
          aria-describedby={
            scheduleError ? "course-schedules-error" : undefined
          }
        >
          <legend className={styles.legendRow}>
            수업 시간
            <StatusPill tone="bad">필수</StatusPill>
          </legend>
          <div className={styles.rows}>
            {values.schedules.map((row, index) => {
              const number = index + 1;
              return (
                <div key={row.key} className={styles.row}>
                  <span className={styles.rowNumber}>{number}</span>
                  <select
                    ref={(element) => {
                      if (index === 0) {
                        inputs.current.schedules = element;
                      }
                    }}
                    className={styles.control}
                    aria-label={`${number}번째 요일`}
                    value={row.weekday}
                    onChange={(event) =>
                      changeRow(row.key, { weekday: event.target.value })
                    }
                  >
                    {WEEKDAYS.map((label, day) => (
                      <option key={label} value={day + 1}>
                        {label}
                      </option>
                    ))}
                  </select>
                  {usedTimes.length > 0 && (
                    <select
                      className={styles.control}
                      aria-label={`${number}번째 시간대 고르기`}
                      value={
                        usedTimes.some(
                          (time) =>
                            timeRangeLabel(time) === timeRangeLabel(row),
                        )
                          ? timeRangeLabel(row)
                          : ""
                      }
                      onChange={(event) => {
                        const range = usedTimes.find(
                          (time) => timeRangeLabel(time) === event.target.value,
                        );
                        if (range) {
                          changeRow(row.key, {
                            start: range.start,
                            end: range.end,
                          });
                        }
                      }}
                    >
                      <option value="" hidden>
                        쓴 시간대
                      </option>
                      {usedTimes.map((time) => (
                        <option
                          key={timeRangeLabel(time)}
                          value={timeRangeLabel(time)}
                        >
                          {timeRangeLabel(time)}
                        </option>
                      ))}
                    </select>
                  )}
                  <input
                    className={styles.control}
                    type="time"
                    aria-label={`${number}번째 시작 시각`}
                    value={row.start}
                    onChange={(event) =>
                      changeRow(row.key, { start: event.target.value })
                    }
                  />
                  <span>~</span>
                  <input
                    className={styles.control}
                    type="time"
                    aria-label={`${number}번째 종료 시각`}
                    value={row.end}
                    onChange={(event) =>
                      changeRow(row.key, { end: event.target.value })
                    }
                  />
                  <select
                    className={`${styles.control} ${styles.room}`}
                    aria-label={`${number}번째 강의실`}
                    value={row.roomId}
                    onChange={(event) =>
                      changeRow(row.key, { roomId: event.target.value })
                    }
                  >
                    <option value="">강의실 없음</option>
                    {rooms.map((room) => (
                      <option key={room.id} value={room.id}>
                        {roomLabel(room, showBuilding)}
                      </option>
                    ))}
                  </select>
                  <Button
                    size="sm"
                    variant="secondary"
                    aria-label={`${number}번째 수업 시간 빼기`}
                    // 수업 시간은 한 줄 이상이라 마지막 줄은 뺄 수 없음 (BR-T4)
                    disabled={values.schedules.length === 1}
                    onClick={() =>
                      change(
                        {
                          schedules: values.schedules.filter(
                            (other) => other.key !== row.key,
                          ),
                        },
                        "schedules",
                      )
                    }
                  >
                    빼기
                  </Button>
                </div>
              );
            })}
          </div>
          <Button
            size="sm"
            variant="secondary"
            className={styles.addButton}
            onClick={() =>
              change(
                { schedules: addScheduleRow(values.schedules) },
                "schedules",
              )
            }
          >
            수업 시간 더하기
          </Button>
          {scheduleError && (
            <ul
              id="course-schedules-error"
              className={styles.errors}
              role="alert"
            >
              {scheduleError.split("\n").map((message) => (
                <li key={message}>{message}</li>
              ))}
            </ul>
          )}
        </fieldset>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}
      </form>

      <ModalDialog
        open={Boolean(editing && confirmingDelete)}
        size="sm"
        title={`${editing ? courseName(editing) : ""} 강좌를 지울까요?`}
        busy={remove.isPending}
        initialFocusRef={cancelDeleteRef}
        onClose={() => setConfirmingDelete(false)}
        footer={
          <>
            <Button
              ref={cancelDeleteRef}
              variant="secondary"
              disabled={remove.isPending}
              onClick={() => setConfirmingDelete(false)}
            >
              그만두기
            </Button>
            <Button
              variant="dangerSolid"
              busy={remove.isPending}
              busyLabel="지우는 중…"
              onClick={() => {
                if (editing) {
                  remove.mutate(editing.id);
                }
              }}
            >
              지우기
            </Button>
          </>
        }
      >
        <p className={styles.confirmText}>
          지우면 되돌릴 수 없습니다. 계속할까요?
        </p>
      </ModalDialog>
    </Drawer>
  );
}
