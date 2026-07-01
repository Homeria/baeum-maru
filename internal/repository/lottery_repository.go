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
