package repository

import (
	"context"
	"database/sql"
	"fmt"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
)

type AccessCodeRepository struct {
	db *sql.DB
}

type CreateAccessUserParams struct {
	Username    string
	DisplayName string
	Affiliation string
	ContactNote string
	Role        string
	ExpiresAt   string
}

type CreateAccessCodeParams struct {
	UserID    int64
	CodeHash  string
	Label     string
	ExpiresAt string
	Note      string
}

type AccessCodeListItem struct {
	User       domain.User
	AccessCode domain.AccessCode
}

func NewAccessCodeRepository(db *sql.DB) *AccessCodeRepository {
	return &AccessCodeRepository{db: db}
}

func (r *AccessCodeRepository) CreateUser(ctx context.Context, params CreateAccessUserParams) (domain.User, error) {
	result, err := r.db.ExecContext(ctx, `
INSERT INTO users (
	username, password_hash, display_name, role, is_active,
	affiliation, contact_note, access_role, status, expires_at
)
VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?);
`, strings.TrimSpace(params.Username), "access-code-only", strings.TrimSpace(params.DisplayName), "staff", nullString(params.Affiliation), nullString(params.ContactNote), strings.TrimSpace(params.Role), domain.UserStatusActive, nullString(params.ExpiresAt))
	if err != nil {
		return domain.User{}, fmt.Errorf("insert access user: %w", err)
	}
	id, err := result.LastInsertId()
	if err != nil {
		return domain.User{}, fmt.Errorf("read access user id: %w", err)
	}
	return r.GetUser(ctx, id)
}

func (r *AccessCodeRepository) GetUser(ctx context.Context, id int64) (domain.User, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT id, username, display_name, affiliation, contact_note, access_role, status, expires_at, last_login_at, created_at, updated_at
FROM users
WHERE id = ?;
`, id)
	return scanUser(row)
}

func (r *AccessCodeRepository) CreateAccessCode(ctx context.Context, params CreateAccessCodeParams) (domain.AccessCode, error) {
	result, err := r.db.ExecContext(ctx, `
INSERT INTO access_codes (user_id, code_hash, label, expires_at, note)
VALUES (?, ?, ?, ?, ?);
`, params.UserID, params.CodeHash, nullString(params.Label), params.ExpiresAt, nullString(params.Note))
	if err != nil {
		return domain.AccessCode{}, fmt.Errorf("insert access code: %w", err)
	}
	id, err := result.LastInsertId()
	if err != nil {
		return domain.AccessCode{}, fmt.Errorf("read access code id: %w", err)
	}
	return r.GetAccessCode(ctx, id)
}

func (r *AccessCodeRepository) GetAccessCode(ctx context.Context, id int64) (domain.AccessCode, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT id, user_id, code_hash, label, status, issued_at, expires_at, revoked_at, last_used_at, note, created_at, updated_at
FROM access_codes
WHERE id = ?;
`, id)
	return scanAccessCode(row)
}

func (r *AccessCodeRepository) FindPrincipalByCodeHash(ctx context.Context, codeHash string) (domain.AccessCodePrincipal, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT
	u.id, u.username, u.display_name, u.affiliation, u.contact_note, u.access_role, u.status, u.expires_at, u.last_login_at, u.created_at, u.updated_at,
	c.id, c.user_id, c.code_hash, c.label, c.status, c.issued_at, c.expires_at, c.revoked_at, c.last_used_at, c.note, c.created_at, c.updated_at
FROM access_codes c
JOIN users u ON u.id = c.user_id
WHERE c.code_hash = ?
LIMIT 1;
`, codeHash)

	var principal domain.AccessCodePrincipal
	var err error
	principal.User, principal.AccessCode, err = scanUserAndAccessCode(row)
	if err != nil {
		return domain.AccessCodePrincipal{}, err
	}
	return principal, nil
}

func (r *AccessCodeRepository) MarkAccessCodeUsed(ctx context.Context, accessCodeID int64, userID int64, usedAt string) error {
	_, err := r.db.ExecContext(ctx, `
UPDATE access_codes
SET last_used_at = ?, updated_at = CURRENT_TIMESTAMP
WHERE id = ?;
`, usedAt, accessCodeID)
	if err != nil {
		return fmt.Errorf("mark access code used: %w", err)
	}
	_, err = r.db.ExecContext(ctx, `
UPDATE users
SET last_login_at = ?, updated_at = CURRENT_TIMESTAMP
WHERE id = ?;
`, usedAt, userID)
	if err != nil {
		return fmt.Errorf("mark access user login: %w", err)
	}
	return nil
}

func (r *AccessCodeRepository) ListRecentAccessCodes(ctx context.Context, limit int) ([]AccessCodeListItem, error) {
	if limit <= 0 {
		limit = 50
	}
	if limit > 500 {
		limit = 500
	}
	rows, err := r.db.QueryContext(ctx, `
SELECT
	u.id, u.username, u.display_name, u.affiliation, u.contact_note, u.access_role, u.status, u.expires_at, u.last_login_at, u.created_at, u.updated_at,
	c.id, c.user_id, c.code_hash, c.label, c.status, c.issued_at, c.expires_at, c.revoked_at, c.last_used_at, c.note, c.created_at, c.updated_at
FROM access_codes c
JOIN users u ON u.id = c.user_id
ORDER BY c.id DESC
LIMIT ?;
`, limit)
	if err != nil {
		return nil, fmt.Errorf("list access codes: %w", err)
	}
	defer rows.Close()

	var items []AccessCodeListItem
	for rows.Next() {
		user, code, err := scanUserAndAccessCode(rows)
		if err != nil {
			return nil, err
		}
		items = append(items, AccessCodeListItem{User: user, AccessCode: code})
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate access codes: %w", err)
	}
	return items, nil
}

func (r *AccessCodeRepository) RevokeAccessCode(ctx context.Context, accessCodeID int64, revokedAt string) error {
	result, err := r.db.ExecContext(ctx, `
UPDATE access_codes
SET status = ?, revoked_at = ?, updated_at = CURRENT_TIMESTAMP
WHERE id = ? AND status = ?;
`, domain.AccessCodeStatusRevoked, revokedAt, accessCodeID, domain.AccessCodeStatusActive)
	if err != nil {
		return fmt.Errorf("revoke access code: %w", err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("read revoked access code count: %w", err)
	}
	if affected == 0 {
		return sql.ErrNoRows
	}
	return nil
}

type userScanner interface {
	Scan(dest ...any) error
}

func scanUser(scanner userScanner) (domain.User, error) {
	var user domain.User
	var affiliation, contactNote, expiresAt, lastLoginAt sql.NullString
	if err := scanner.Scan(
		&user.ID,
		&user.Username,
		&user.DisplayName,
		&affiliation,
		&contactNote,
		&user.Role,
		&user.Status,
		&expiresAt,
		&lastLoginAt,
		&user.CreatedAt,
		&user.UpdatedAt,
	); err != nil {
		return domain.User{}, fmt.Errorf("scan user: %w", err)
	}
	user.Affiliation = affiliation.String
	user.ContactNote = contactNote.String
	user.ExpiresAt = expiresAt.String
	user.LastLoginAt = lastLoginAt.String
	return user, nil
}

func scanAccessCode(scanner userScanner) (domain.AccessCode, error) {
	var code domain.AccessCode
	var label, revokedAt, lastUsedAt, note sql.NullString
	if err := scanner.Scan(
		&code.ID,
		&code.UserID,
		&code.CodeHash,
		&label,
		&code.Status,
		&code.IssuedAt,
		&code.ExpiresAt,
		&revokedAt,
		&lastUsedAt,
		&note,
		&code.CreatedAt,
		&code.UpdatedAt,
	); err != nil {
		return domain.AccessCode{}, fmt.Errorf("scan access code: %w", err)
	}
	code.Label = label.String
	code.RevokedAt = revokedAt.String
	code.LastUsedAt = lastUsedAt.String
	code.Note = note.String
	return code, nil
}

func scanUserAndAccessCode(scanner userScanner) (domain.User, domain.AccessCode, error) {
	var user domain.User
	var code domain.AccessCode
	var affiliation, contactNote, userExpiresAt, lastLoginAt sql.NullString
	var label, revokedAt, lastUsedAt, note sql.NullString
	if err := scanner.Scan(
		&user.ID,
		&user.Username,
		&user.DisplayName,
		&affiliation,
		&contactNote,
		&user.Role,
		&user.Status,
		&userExpiresAt,
		&lastLoginAt,
		&user.CreatedAt,
		&user.UpdatedAt,
		&code.ID,
		&code.UserID,
		&code.CodeHash,
		&label,
		&code.Status,
		&code.IssuedAt,
		&code.ExpiresAt,
		&revokedAt,
		&lastUsedAt,
		&note,
		&code.CreatedAt,
		&code.UpdatedAt,
	); err != nil {
		return domain.User{}, domain.AccessCode{}, fmt.Errorf("scan access code principal: %w", err)
	}
	user.Affiliation = affiliation.String
	user.ContactNote = contactNote.String
	user.ExpiresAt = userExpiresAt.String
	user.LastLoginAt = lastLoginAt.String
	code.Label = label.String
	code.RevokedAt = revokedAt.String
	code.LastUsedAt = lastUsedAt.String
	code.Note = note.String
	return user, code, nil
}

func nullString(value string) any {
	value = strings.TrimSpace(value)
	if value == "" {
		return nil
	}
	return value
}
