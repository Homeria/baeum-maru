package repository

import (
	"context"
	"testing"
)

func TestAuditRepositoryCreateAndListRecent(t *testing.T) {
	ctx := context.Background()
	repo := NewAuditRepository(newTestDB(t))

	created, err := repo.Create(ctx, CreateAuditLogParams{
		Action:     "member.create",
		EntityType: "member",
		EntityID:   7,
		Summary:    "회원 등록 #7",
	})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if created.ID == 0 {
		t.Fatal("created.ID = 0, want generated ID")
	}

	logs, err := repo.ListRecent(ctx, 10)
	if err != nil {
		t.Fatalf("ListRecent() error = %v", err)
	}
	if len(logs) != 1 {
		t.Fatalf("len(logs) = %d, want 1", len(logs))
	}
	if logs[0].Action != "member.create" || logs[0].EntityID != 7 {
		t.Fatalf("logs[0] = %+v, want member.create entity 7", logs[0])
	}
}
