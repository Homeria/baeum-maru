package service

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type fakeAuditRepository struct {
	created repository.CreateAuditLogParams
	logs    []domain.AuditLog
}

func (f *fakeAuditRepository) Create(_ context.Context, params repository.CreateAuditLogParams) (domain.AuditLog, error) {
	f.created = params
	return domain.AuditLog{ID: 1, Action: params.Action, EntityType: params.EntityType, EntityID: params.EntityID, Summary: params.Summary}, nil
}

func (f *fakeAuditRepository) ListRecent(context.Context, int) ([]domain.AuditLog, error) {
	return f.logs, nil
}

func TestAuditServiceRecordsEvent(t *testing.T) {
	repo := &fakeAuditRepository{}
	service := NewAuditService(repo)

	if err := service.Record(context.Background(), AuditEvent{
		Action:     "member.create",
		EntityType: "member",
		EntityID:   1,
		Summary:    "회원 등록 #1",
	}); err != nil {
		t.Fatalf("Record() error = %v", err)
	}
	if repo.created.Action != "member.create" || repo.created.EntityID != 1 {
		t.Fatalf("created = %+v, want member.create entity 1", repo.created)
	}
}

func TestAuditServiceRejectsMissingSummary(t *testing.T) {
	service := NewAuditService(&fakeAuditRepository{})

	if err := service.Record(context.Background(), AuditEvent{Action: "member.create", EntityType: "member"}); err == nil {
		t.Fatal("Record() error = nil, want validation error")
	}
}
