package service

import (
	"context"
	"errors"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type AuditRepository interface {
	Create(context.Context, repository.CreateAuditLogParams) (domain.AuditLog, error)
	ListRecent(context.Context, int) ([]domain.AuditLog, error)
}

type AuditService struct {
	audits AuditRepository
}

type AuditEvent struct {
	ActorUserID int64
	Action      string
	EntityType  string
	EntityID    int64
	Summary     string
}

func NewAuditService(audits AuditRepository) *AuditService {
	return &AuditService{audits: audits}
}

func (s *AuditService) Record(ctx context.Context, event AuditEvent) error {
	if s == nil || s.audits == nil {
		return nil
	}
	if err := validateAuditEvent(event); err != nil {
		return err
	}
	_, err := s.audits.Create(ctx, repository.CreateAuditLogParams{
		ActorUserID: event.ActorUserID,
		Action:      event.Action,
		EntityType:  event.EntityType,
		EntityID:    event.EntityID,
		Summary:     event.Summary,
	})
	return err
}

func (s *AuditService) ListRecent(ctx context.Context, limit int) ([]domain.AuditLog, error) {
	if s == nil || s.audits == nil {
		return nil, errors.New("audit repository is not configured")
	}
	return s.audits.ListRecent(ctx, limit)
}

func validateAuditEvent(event AuditEvent) error {
	if strings.TrimSpace(event.Action) == "" {
		return errors.New("audit action is required")
	}
	if strings.TrimSpace(event.EntityType) == "" {
		return errors.New("audit entity type is required")
	}
	if strings.TrimSpace(event.Summary) == "" {
		return errors.New("audit summary is required")
	}
	return nil
}
