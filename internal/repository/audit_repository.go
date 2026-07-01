package repository

import (
	"context"
	"database/sql"
	"fmt"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
)

type AuditRepository struct {
	db *sql.DB
}

type CreateAuditLogParams struct {
	ActorUserID int64
	Action      string
	EntityType  string
	EntityID    int64
	Summary     string
}

func NewAuditRepository(db *sql.DB) *AuditRepository {
	return &AuditRepository{db: db}
}

func (r *AuditRepository) Create(ctx context.Context, params CreateAuditLogParams) (domain.AuditLog, error) {
	var actorUserID any
	if params.ActorUserID > 0 {
		actorUserID = params.ActorUserID
	}
	var entityID any
	if params.EntityID > 0 {
		entityID = params.EntityID
	}

	result, err := r.db.ExecContext(ctx, `
INSERT INTO audit_logs (actor_user_id, action, entity_type, entity_id, summary)
VALUES (?, ?, ?, ?, ?);
`, actorUserID, strings.TrimSpace(params.Action), strings.TrimSpace(params.EntityType), entityID, strings.TrimSpace(params.Summary))
	if err != nil {
		return domain.AuditLog{}, fmt.Errorf("insert audit log: %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return domain.AuditLog{}, fmt.Errorf("read audit log id: %w", err)
	}
	return r.get(ctx, id)
}

func (r *AuditRepository) ListRecent(ctx context.Context, limit int) ([]domain.AuditLog, error) {
	if limit <= 0 {
		limit = 100
	}
	if limit > 1000 {
		limit = 1000
	}

	rows, err := r.db.QueryContext(ctx, `
SELECT id, actor_user_id, action, entity_type, entity_id, summary, created_at
FROM audit_logs
ORDER BY id DESC
LIMIT ?;
`, limit)
	if err != nil {
		return nil, fmt.Errorf("list audit logs: %w", err)
	}
	defer rows.Close()

	var logs []domain.AuditLog
	for rows.Next() {
		log, err := scanAuditLog(rows)
		if err != nil {
			return nil, err
		}
		logs = append(logs, log)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate audit logs: %w", err)
	}
	return logs, nil
}

func (r *AuditRepository) get(ctx context.Context, id int64) (domain.AuditLog, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT id, actor_user_id, action, entity_type, entity_id, summary, created_at
FROM audit_logs
WHERE id = ?;
`, id)
	return scanAuditLog(row)
}

type auditLogScanner interface {
	Scan(dest ...any) error
}

func scanAuditLog(scanner auditLogScanner) (domain.AuditLog, error) {
	var log domain.AuditLog
	var actorUserID sql.NullInt64
	var entityID sql.NullInt64
	if err := scanner.Scan(
		&log.ID,
		&actorUserID,
		&log.Action,
		&log.EntityType,
		&entityID,
		&log.Summary,
		&log.CreatedAt,
	); err != nil {
		return domain.AuditLog{}, fmt.Errorf("scan audit log: %w", err)
	}
	if actorUserID.Valid {
		log.ActorUserID = actorUserID.Int64
	}
	if entityID.Valid {
		log.EntityID = entityID.Int64
	}
	return log, nil
}
