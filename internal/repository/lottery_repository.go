package repository

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/domain"
)

type LotteryRepository struct {
	db *sql.DB
}

type SaveLotteryRunParams struct {
	OfferingID  int64
	TermID      int64
	Seed        int64
	Assignments []domain.LotteryAssignment
}

func NewLotteryRepository(db *sql.DB) *LotteryRepository {
	return &LotteryRepository{db: db}
}

func (r *LotteryRepository) ListCandidatesByOffering(ctx context.Context, offeringID int64) ([]domain.LotteryCandidate, error) {
	rows, err := r.db.QueryContext(ctx, `
SELECT r.id,
       m.id,
       m.name,
       COALESCE(m.member_no, ''),
       r.offering_id,
       r.status,
       r.created_at
FROM registrations r
JOIN members m ON m.id = r.member_id
WHERE r.offering_id = ?
  AND r.status IN ('applied', 'selected', 'waitlisted', 'rejected')
ORDER BY r.created_at, r.id;
`, offeringID)
	if err != nil {
		return nil, fmt.Errorf("list lottery candidates: %w", err)
	}
	defer rows.Close()

	var candidates []domain.LotteryCandidate
	for rows.Next() {
		var candidate domain.LotteryCandidate
		if err := rows.Scan(
			&candidate.RegistrationID,
			&candidate.MemberID,
			&candidate.MemberName,
			&candidate.MemberNo,
			&candidate.OfferingID,
			&candidate.Status,
			&candidate.CreatedAt,
		); err != nil {
			return nil, fmt.Errorf("scan lottery candidate: %w", err)
		}
		candidates = append(candidates, candidate)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate lottery candidates: %w", err)
	}
	return candidates, nil
}

func (r *LotteryRepository) SaveRun(ctx context.Context, params SaveLotteryRunParams) (int64, error) {
	var runID int64
	err := database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		result, err := tx.ExecContext(ctx, `
INSERT INTO lottery_runs (term_id, seed, status, completed_at)
VALUES (?, ?, 'completed', CURRENT_TIMESTAMP);
`, params.TermID, params.Seed)
		if err != nil {
			return fmt.Errorf("insert lottery run: %w", err)
		}
		runID, err = result.LastInsertId()
		if err != nil {
			return fmt.Errorf("read lottery run id: %w", err)
		}
		if params.OfferingID > 0 {
			if _, err := tx.ExecContext(ctx, `
INSERT INTO lottery_run_targets (lottery_run_id, offering_id)
VALUES (?, ?);
`, runID, params.OfferingID); err != nil {
				return fmt.Errorf("insert lottery run target: %w", err)
			}
		}

		for _, assignment := range params.Assignments {
			if err := saveLotteryAssignment(ctx, tx, runID, assignment); err != nil {
				return err
			}
		}
		return nil
	})
	if err != nil {
		return 0, err
	}
	return runID, nil
}

func (r *LotteryRepository) ListRuns(ctx context.Context, limit int) ([]domain.LotteryRun, error) {
	if limit <= 0 {
		limit = 50
	}
	if limit > 1000 {
		limit = 1000
	}

	rows, err := r.db.QueryContext(ctx, `
SELECT lr.id,
       lr.term_id,
       t.name,
       lr.seed,
       lr.status,
       lr.started_at,
       COALESCE(lr.completed_at, ''),
       COALESCE(target_co.id, co.id, 0),
       COALESCE(target_co.display_name, co.display_name, ''),
       COUNT(lres.id),
       COALESCE(SUM(CASE WHEN lres.result = 'selected' THEN 1 ELSE 0 END), 0),
       COALESCE(SUM(CASE WHEN lres.result = 'waitlisted' THEN 1 ELSE 0 END), 0)
FROM lottery_runs lr
JOIN terms t ON t.id = lr.term_id
LEFT JOIN lottery_run_targets lrt ON lrt.lottery_run_id = lr.id
LEFT JOIN course_offerings target_co ON target_co.id = lrt.offering_id
LEFT JOIN lottery_results lres ON lres.lottery_run_id = lr.id
LEFT JOIN registrations r ON r.id = lres.registration_id
LEFT JOIN course_offerings co ON co.id = r.offering_id
GROUP BY lr.id, target_co.id, co.id
ORDER BY lr.started_at DESC, lr.id DESC
LIMIT ?;
`, limit)
	if err != nil {
		return nil, fmt.Errorf("list lottery runs: %w", err)
	}
	defer rows.Close()

	var runs []domain.LotteryRun
	for rows.Next() {
		var run domain.LotteryRun
		if err := rows.Scan(
			&run.ID,
			&run.TermID,
			&run.TermName,
			&run.Seed,
			&run.Status,
			&run.StartedAt,
			&run.CompletedAt,
			&run.OfferingID,
			&run.CourseTitle,
			&run.TotalCount,
			&run.SelectedCount,
			&run.WaitlistedCount,
		); err != nil {
			return nil, fmt.Errorf("scan lottery run: %w", err)
		}
		runs = append(runs, run)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate lottery runs: %w", err)
	}
	return runs, nil
}

func (r *LotteryRepository) LatestRunByOffering(ctx context.Context, offeringID int64) (domain.LotteryRun, bool, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT lr.id,
       lr.term_id,
       t.name,
       lr.seed,
       lr.status,
       lr.started_at,
       COALESCE(lr.completed_at, ''),
       target_co.id,
       target_co.display_name,
       COUNT(lres.id),
       COALESCE(SUM(CASE WHEN lres.result = 'selected' THEN 1 ELSE 0 END), 0),
       COALESCE(SUM(CASE WHEN lres.result = 'waitlisted' THEN 1 ELSE 0 END), 0)
FROM lottery_runs lr
JOIN terms t ON t.id = lr.term_id
JOIN lottery_run_targets lrt ON lrt.lottery_run_id = lr.id
JOIN course_offerings target_co ON target_co.id = lrt.offering_id
LEFT JOIN lottery_results lres ON lres.lottery_run_id = lr.id
LEFT JOIN registrations r ON r.id = lres.registration_id
WHERE target_co.id = ?
GROUP BY lr.id, target_co.id
ORDER BY lr.completed_at DESC, lr.started_at DESC, lr.id DESC
LIMIT 1;
`, offeringID)

	var run domain.LotteryRun
	if err := row.Scan(
		&run.ID,
		&run.TermID,
		&run.TermName,
		&run.Seed,
		&run.Status,
		&run.StartedAt,
		&run.CompletedAt,
		&run.OfferingID,
		&run.CourseTitle,
		&run.TotalCount,
		&run.SelectedCount,
		&run.WaitlistedCount,
	); err != nil {
		if err == sql.ErrNoRows {
			return domain.LotteryRun{}, false, nil
		}
		return domain.LotteryRun{}, false, fmt.Errorf("read latest lottery run by offering: %w", err)
	}
	return run, true, nil
}

func (r *LotteryRepository) ListResultsByRun(ctx context.Context, runID int64) ([]domain.LotteryResultRow, error) {
	rows, err := r.db.QueryContext(ctx, `
SELECT lr.id,
       lr.seed,
       COALESCE(lr.completed_at, ''),
       co.id,
       co.display_name,
       t.name,
       lres.result,
       lres.result_order,
       r.id,
       m.id,
       COALESCE(m.member_no, ''),
       m.name,
       r.created_at
FROM lottery_results lres
JOIN lottery_runs lr ON lr.id = lres.lottery_run_id
JOIN registrations r ON r.id = lres.registration_id
JOIN members m ON m.id = r.member_id
JOIN course_offerings co ON co.id = r.offering_id
JOIN terms t ON t.id = co.term_id
WHERE lr.id = ?
ORDER BY lres.result_order, lres.id;
`, runID)
	if err != nil {
		return nil, fmt.Errorf("list lottery results: %w", err)
	}
	defer rows.Close()

	var results []domain.LotteryResultRow
	for rows.Next() {
		var result domain.LotteryResultRow
		if err := rows.Scan(
			&result.RunID,
			&result.Seed,
			&result.CompletedAt,
			&result.OfferingID,
			&result.CourseTitle,
			&result.TermName,
			&result.Result,
			&result.ResultOrder,
			&result.RegistrationID,
			&result.MemberID,
			&result.MemberNo,
			&result.MemberName,
			&result.RegistrationCreatedAt,
		); err != nil {
			return nil, fmt.Errorf("scan lottery result: %w", err)
		}
		results = append(results, result)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate lottery results: %w", err)
	}
	return results, nil
}

func saveLotteryAssignment(ctx context.Context, tx *sql.Tx, runID int64, assignment domain.LotteryAssignment) error {
	var fromStatus string
	if err := tx.QueryRowContext(ctx, "SELECT status FROM registrations WHERE id = ?;", assignment.RegistrationID).Scan(&fromStatus); err != nil {
		return fmt.Errorf("read registration status for lottery: %w", err)
	}

	if _, err := tx.ExecContext(ctx, `
INSERT INTO lottery_results (lottery_run_id, registration_id, result, result_order)
VALUES (?, ?, ?, ?);
`, runID, assignment.RegistrationID, assignment.Result, assignment.ResultOrder); err != nil {
		return fmt.Errorf("insert lottery result: %w", err)
	}
	if _, err := tx.ExecContext(ctx, `
UPDATE registrations
SET status = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;
`, assignment.Result, assignment.RegistrationID); err != nil {
		return fmt.Errorf("update registration lottery status: %w", err)
	}
	if _, err := tx.ExecContext(ctx, `
INSERT INTO registration_status_history (registration_id, from_status, to_status, reason)
VALUES (?, ?, ?, ?);
`, assignment.RegistrationID, fromStatus, assignment.Result, fmt.Sprintf("lottery_run:%d", runID)); err != nil {
		return fmt.Errorf("insert lottery status history: %w", err)
	}
	return nil
}
