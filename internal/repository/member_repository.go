// Package repository contains database access code.
package repository

import (
	"context"
	"database/sql"
	"fmt"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
)

type MemberRepository struct {
	db *sql.DB
}

type CreateMemberParams struct {
	MemberNo   string
	Name       string
	GenderCode string
	BirthDate  string
	Phone      string
	Note       string
}

type UpdateMemberParams struct {
	ID         int64
	MemberNo   string
	Name       string
	GenderCode string
	BirthDate  string
	Phone      string
	Note       string
}

func NewMemberRepository(db *sql.DB) *MemberRepository {
	return &MemberRepository{db: db}
}

func (r *MemberRepository) Create(ctx context.Context, params CreateMemberParams) (domain.Member, error) {
	result, err := r.db.ExecContext(ctx, `
INSERT INTO members (member_no, name, gender_code, birth_date, phone, note)
VALUES (NULLIF(?, ''), ?, NULLIF(?, ''), NULLIF(?, ''), NULLIF(?, ''), NULLIF(?, ''));
`, strings.TrimSpace(params.MemberNo), strings.TrimSpace(params.Name), strings.TrimSpace(params.GenderCode), strings.TrimSpace(params.BirthDate), strings.TrimSpace(params.Phone), strings.TrimSpace(params.Note))
	if err != nil {
		return domain.Member{}, fmt.Errorf("insert member: %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return domain.Member{}, fmt.Errorf("read member id: %w", err)
	}
	return r.Get(ctx, id)
}

func (r *MemberRepository) Update(ctx context.Context, params UpdateMemberParams) (domain.Member, error) {
	result, err := r.db.ExecContext(ctx, `
UPDATE members
SET member_no = NULLIF(?, ''),
    name = ?,
    gender_code = NULLIF(?, ''),
    birth_date = NULLIF(?, ''),
    phone = NULLIF(?, ''),
    note = NULLIF(?, ''),
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;
`, strings.TrimSpace(params.MemberNo), strings.TrimSpace(params.Name), strings.TrimSpace(params.GenderCode), strings.TrimSpace(params.BirthDate), strings.TrimSpace(params.Phone), strings.TrimSpace(params.Note), params.ID)
	if err != nil {
		return domain.Member{}, fmt.Errorf("update member %d: %w", params.ID, err)
	}

	affected, err := result.RowsAffected()
	if err != nil {
		return domain.Member{}, fmt.Errorf("read affected member rows: %w", err)
	}
	if affected == 0 {
		return domain.Member{}, sql.ErrNoRows
	}
	return r.Get(ctx, params.ID)
}

func (r *MemberRepository) Get(ctx context.Context, id int64) (domain.Member, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT id, COALESCE(member_no, ''), name, COALESCE(gender_code, ''), COALESCE(birth_date, ''),
       COALESCE(phone, ''), COALESCE(note, ''), created_at, updated_at
FROM members
WHERE id = ?;
`, id)
	return scanMember(row)
}

func (r *MemberRepository) Search(ctx context.Context, query string, limit int) ([]domain.Member, error) {
	if limit <= 0 {
		limit = 50
	}
	if limit > 10000 {
		limit = 10000
	}

	query = strings.TrimSpace(query)
	like := "%" + query + "%"
	rows, err := r.db.QueryContext(ctx, `
SELECT id, COALESCE(member_no, ''), name, COALESCE(gender_code, ''), COALESCE(birth_date, ''),
       COALESCE(phone, ''), COALESCE(note, ''), created_at, updated_at
FROM members
WHERE ? = ''
   OR name LIKE ?
   OR COALESCE(member_no, '') LIKE ?
   OR COALESCE(phone, '') LIKE ?
ORDER BY name, id
LIMIT ?;
`, query, like, like, like, limit)
	if err != nil {
		return nil, fmt.Errorf("search members: %w", err)
	}
	defer rows.Close()

	var members []domain.Member
	for rows.Next() {
		member, err := scanMember(rows)
		if err != nil {
			return nil, err
		}
		members = append(members, member)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate members: %w", err)
	}
	return members, nil
}

type memberScanner interface {
	Scan(dest ...any) error
}

func scanMember(scanner memberScanner) (domain.Member, error) {
	var member domain.Member
	if err := scanner.Scan(
		&member.ID,
		&member.MemberNo,
		&member.Name,
		&member.GenderCode,
		&member.BirthDate,
		&member.Phone,
		&member.Note,
		&member.CreatedAt,
		&member.UpdatedAt,
	); err != nil {
		return domain.Member{}, fmt.Errorf("scan member: %w", err)
	}
	return member, nil
}
