package repository

import (
	"context"
	"database/sql"
	"fmt"

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

func (r *RegistrationRepository) Cancel(ctx context.Context, id int64) (domain.Registration, error) {
	result, err := r.db.ExecContext(ctx, `
UPDATE registrations
SET status = 'cancelled',
    cancelled_at = CURRENT_TIMESTAMP,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ? AND status != 'cancelled';
`, id)
	if err != nil {
		return domain.Registration{}, fmt.Errorf("cancel registration %d: %w", id, err)
	}

	affected, err := result.RowsAffected()
	if err != nil {
		return domain.Registration{}, fmt.Errorf("read affected registration rows: %w", err)
	}
	if affected == 0 {
		return domain.Registration{}, sql.ErrNoRows
	}
	return r.Get(ctx, id)
}

func (r *RegistrationRepository) Get(ctx context.Context, id int64) (domain.Registration, error) {
	row := r.db.QueryRowContext(ctx, registrationSelectSQL()+`
WHERE r.id = ?;
`, id)
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
	if limit <= 0 || limit > 300 {
		limit = 100
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

func registrationSelectSQL() string {
	return `
SELECT r.id,
       m.id,
       m.name,
       COALESCE(m.member_no, ''),
       co.id,
       c.title,
       t.name,
       r.status,
       r.created_at,
       r.updated_at,
       COALESCE(r.cancelled_at, '')
FROM registrations r
JOIN members m ON m.id = r.member_id
JOIN course_offerings co ON co.id = r.offering_id
JOIN courses c ON c.id = co.course_id
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
