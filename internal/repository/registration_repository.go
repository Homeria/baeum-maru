package repository

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/domain"
)

type RegistrationRepository struct {
	db *sql.DB
}

type CreateRegistrationParams struct {
	MemberID   int64
	OfferingID int64
}

func NewRegistrationRepository(db *sql.DB) *RegistrationRepository {
	return &RegistrationRepository{db: db}
}

func (r *RegistrationRepository) Create(ctx context.Context, params CreateRegistrationParams) (domain.Registration, error) {
	reactivated, err := r.reactivateCancelled(ctx, params)
	if err != nil {
		return domain.Registration{}, err
	}
	if reactivated {
		return r.GetByMemberAndOffering(ctx, params.MemberID, params.OfferingID)
	}

	result, err := r.db.ExecContext(ctx, `
INSERT INTO registrations (member_id, offering_id, status)
VALUES (?, ?, 'applied');
`, params.MemberID, params.OfferingID)
	if err != nil {
		return domain.Registration{}, fmt.Errorf("insert registration: %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return domain.Registration{}, fmt.Errorf("read registration id: %w", err)
	}
	return r.Get(ctx, id)
}

func (r *RegistrationRepository) reactivateCancelled(ctx context.Context, params CreateRegistrationParams) (bool, error) {
	result, err := r.db.ExecContext(ctx, `
UPDATE registrations
SET status = 'applied',
    cancelled_at = NULL,
    updated_at = CURRENT_TIMESTAMP
WHERE member_id = ? AND offering_id = ? AND status = 'cancelled';
`, params.MemberID, params.OfferingID)
	if err != nil {
		return false, fmt.Errorf("reactivate cancelled registration: %w", err)
	}

	affected, err := result.RowsAffected()
	if err != nil {
		return false, fmt.Errorf("read reactivated registration rows: %w", err)
	}
	return affected > 0, nil
}

func (r *RegistrationRepository) Cancel(ctx context.Context, id int64) (domain.Registration, error) {
	change, err := r.CancelAndPromote(ctx, id)
	if err != nil {
		return domain.Registration{}, err
	}
	return change.Registration, nil
}

func (r *RegistrationRepository) Confirm(ctx context.Context, id int64) (domain.Registration, error) {
	var confirmed domain.Registration
	err := database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		if err := updateRegistrationStatus(ctx, tx, id, "selected", "confirmed", "registration_confirmed"); err != nil {
			return err
		}

		registration, err := r.getWithTx(ctx, tx, id)
		if err != nil {
			return err
		}
		confirmed = registration
		return nil
	})
	if err != nil {
		return domain.Registration{}, err
	}
	return confirmed, nil
}

func (r *RegistrationRepository) CancelAndPromote(ctx context.Context, id int64) (domain.RegistrationStatusChange, error) {
	var change domain.RegistrationStatusChange
	err := database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		var fromStatus string
		var offeringID int64
		if err := tx.QueryRowContext(ctx, "SELECT status, offering_id FROM registrations WHERE id = ?;", id).Scan(&fromStatus, &offeringID); err != nil {
			return fmt.Errorf("read registration for cancellation: %w", err)
		}
		if fromStatus == "cancelled" {
			return sql.ErrNoRows
		}

		result, err := tx.ExecContext(ctx, `
UPDATE registrations
SET status = 'cancelled',
    cancelled_at = CURRENT_TIMESTAMP,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ? AND status != 'cancelled';
`, id)
		if err != nil {
			return fmt.Errorf("cancel registration %d: %w", id, err)
		}

		affected, err := result.RowsAffected()
		if err != nil {
			return fmt.Errorf("read affected registration rows: %w", err)
		}
		if affected == 0 {
			return sql.ErrNoRows
		}
		if err := insertRegistrationStatusHistory(ctx, tx, id, fromStatus, "cancelled", "registration_cancelled"); err != nil {
			return err
		}

		cancelled, err := r.getWithTx(ctx, tx, id)
		if err != nil {
			return err
		}
		change.Registration = cancelled

		if fromStatus == "selected" || fromStatus == "confirmed" {
			promoted, err := promoteNextWaitlisted(ctx, tx, offeringID, id)
			if err != nil {
				return err
			}
			change.Promoted = promoted
		}
		return nil
	})
	if err != nil {
		return domain.RegistrationStatusChange{}, err
	}
	return change, nil
}

func (r *RegistrationRepository) Get(ctx context.Context, id int64) (domain.Registration, error) {
	row := r.db.QueryRowContext(ctx, registrationSelectSQL()+`
WHERE r.id = ?;
`, id)
	return scanRegistration(row)
}

func (r *RegistrationRepository) getWithTx(ctx context.Context, tx *sql.Tx, id int64) (domain.Registration, error) {
	row := tx.QueryRowContext(ctx, registrationSelectSQL()+`
WHERE r.id = ?;
`, id)
	return scanRegistration(row)
}

func (r *RegistrationRepository) GetByMemberAndOffering(ctx context.Context, memberID int64, offeringID int64) (domain.Registration, error) {
	row := r.db.QueryRowContext(ctx, registrationSelectSQL()+`
WHERE r.member_id = ? AND r.offering_id = ?;
`, memberID, offeringID)
	return scanRegistration(row)
}

func (r *RegistrationRepository) ListByMember(ctx context.Context, memberID int64) ([]domain.Registration, error) {
	rows, err := r.db.QueryContext(ctx, registrationSelectSQL()+`
WHERE r.member_id = ?
ORDER BY r.created_at DESC, r.id DESC;
`, memberID)
	if err != nil {
		return nil, fmt.Errorf("list registrations by member: %w", err)
	}
	defer rows.Close()

	return scanRegistrations(rows)
}

func (r *RegistrationRepository) ListByOffering(ctx context.Context, offeringID int64) ([]domain.Registration, error) {
	rows, err := r.db.QueryContext(ctx, registrationSelectSQL()+`
WHERE r.offering_id = ?
ORDER BY m.name, r.id;
`, offeringID)
	if err != nil {
		return nil, fmt.Errorf("list registrations by offering: %w", err)
	}
	defer rows.Close()

	return scanRegistrations(rows)
}

func (r *RegistrationRepository) ListRecent(ctx context.Context, limit int) ([]domain.Registration, error) {
	if limit <= 0 {
		limit = 100
	}
	if limit > 10000 {
		limit = 10000
	}

	rows, err := r.db.QueryContext(ctx, registrationSelectSQL()+`
ORDER BY r.created_at DESC, r.id DESC
LIMIT ?;
`, limit)
	if err != nil {
		return nil, fmt.Errorf("list recent registrations: %w", err)
	}
	defer rows.Close()

	return scanRegistrations(rows)
}

func (r *RegistrationRepository) ListActiveRuleItemsByMember(ctx context.Context, memberID int64) ([]domain.RegistrationRuleItem, error) {
	rows, err := r.db.QueryContext(ctx, `
SELECT r.id,
       r.member_id,
       r.offering_id,
       co.term_id,
       r.status,
       CASE WHEN cs.weekday = 7 THEN 0 ELSE cs.weekday END,
       ts.start_time,
       ts.end_time
FROM registrations r
JOIN course_offerings co ON co.id = r.offering_id
JOIN course_schedules cs ON cs.offering_id = co.id
JOIN time_slots ts ON ts.id = cs.time_slot_id
WHERE r.member_id = ?
  AND r.status IN ('applied', 'selected', 'waitlisted', 'confirmed')
ORDER BY r.id;
`, memberID)
	if err != nil {
		return nil, fmt.Errorf("list active registration rule items: %w", err)
	}
	defer rows.Close()

	var items []domain.RegistrationRuleItem
	for rows.Next() {
		var item domain.RegistrationRuleItem
		if err := rows.Scan(&item.ID, &item.MemberID, &item.OfferingID, &item.TermID, &item.Status, &item.Weekday, &item.StartTime, &item.EndTime); err != nil {
			return nil, fmt.Errorf("scan registration rule item: %w", err)
		}
		items = append(items, item)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate registration rule items: %w", err)
	}
	return items, nil
}

func registrationSelectSQL() string {
	return `
SELECT r.id,
       m.id,
       m.name,
       COALESCE(m.member_no, ''),
       co.id,
       co.display_name,
       t.name,
       r.status,
       r.created_at,
       r.updated_at,
       COALESCE(r.cancelled_at, '')
FROM registrations r
JOIN members m ON m.id = r.member_id
JOIN course_offerings co ON co.id = r.offering_id
JOIN terms t ON t.id = co.term_id
`
}

func scanRegistrations(rows *sql.Rows) ([]domain.Registration, error) {
	var registrations []domain.Registration
	for rows.Next() {
		registration, err := scanRegistration(rows)
		if err != nil {
			return nil, err
		}
		registrations = append(registrations, registration)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate registrations: %w", err)
	}
	return registrations, nil
}

func scanRegistration(scanner interface{ Scan(dest ...any) error }) (domain.Registration, error) {
	var registration domain.Registration
	if err := scanner.Scan(
		&registration.ID,
		&registration.MemberID,
		&registration.MemberName,
		&registration.MemberNo,
		&registration.OfferingID,
		&registration.CourseTitle,
		&registration.TermName,
		&registration.Status,
		&registration.CreatedAt,
		&registration.UpdatedAt,
		&registration.CancelledAt,
	); err != nil {
		return domain.Registration{}, fmt.Errorf("scan registration: %w", err)
	}
	return registration, nil
}

func updateRegistrationStatus(ctx context.Context, tx *sql.Tx, id int64, fromStatus string, toStatus string, reason string) error {
	result, err := tx.ExecContext(ctx, `
UPDATE registrations
SET status = ?,
    cancelled_at = CASE WHEN ? = 'cancelled' THEN CURRENT_TIMESTAMP ELSE cancelled_at END,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ? AND status = ?;
`, toStatus, toStatus, id, fromStatus)
	if err != nil {
		return fmt.Errorf("update registration status: %w", err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("read affected registration rows: %w", err)
	}
	if affected == 0 {
		return sql.ErrNoRows
	}
	return insertRegistrationStatusHistory(ctx, tx, id, fromStatus, toStatus, reason)
}

func promoteNextWaitlisted(ctx context.Context, tx *sql.Tx, offeringID int64, cancelledRegistrationID int64) (*domain.Registration, error) {
	var promotedID int64
	err := tx.QueryRowContext(ctx, `
SELECT r.id
FROM registrations r
LEFT JOIN lottery_results lr ON lr.registration_id = r.id
WHERE r.offering_id = ?
  AND r.status = 'waitlisted'
ORDER BY COALESCE(lr.result_order, r.id), r.created_at, r.id
LIMIT 1;
`, offeringID).Scan(&promotedID)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("find next waitlisted registration: %w", err)
	}
	if err := updateRegistrationStatus(ctx, tx, promotedID, "waitlisted", "selected", fmt.Sprintf("waitlist_promoted_after_cancel:%d", cancelledRegistrationID)); err != nil {
		return nil, err
	}

	row := tx.QueryRowContext(ctx, registrationSelectSQL()+`
WHERE r.id = ?;
`, promotedID)
	promoted, err := scanRegistration(row)
	if err != nil {
		return nil, err
	}
	return &promoted, nil
}

func insertRegistrationStatusHistory(ctx context.Context, tx *sql.Tx, registrationID int64, fromStatus string, toStatus string, reason string) error {
	if _, err := tx.ExecContext(ctx, `
INSERT INTO registration_status_history (registration_id, from_status, to_status, reason)
VALUES (?, ?, ?, ?);
`, registrationID, fromStatus, toStatus, reason); err != nil {
		return fmt.Errorf("insert registration status history: %w", err)
	}
	return nil
}
