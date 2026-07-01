package repository

import (
	"context"
	"testing"
)

func TestMemberRepositoryCreateSearchAndUpdate(t *testing.T) {
	ctx := context.Background()
	repo := NewMemberRepository(newTestDB(t))

	member, err := repo.Create(ctx, CreateMemberParams{
		MemberNo: "M-001",
		Name:     "김배움",
		Phone:    "010-0000-0000",
	})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if member.ID == 0 {
		t.Fatal("member ID = 0, want generated ID")
	}

	members, err := repo.Search(ctx, "배움", 10)
	if err != nil {
		t.Fatalf("Search() error = %v", err)
	}
	if len(members) != 1 {
		t.Fatalf("len(members) = %d, want 1", len(members))
	}

	updated, err := repo.Update(ctx, UpdateMemberParams{
		ID:       member.ID,
		MemberNo: "M-001",
		Name:     "김마루",
		Phone:    "010-1111-1111",
	})
	if err != nil {
		t.Fatalf("Update() error = %v", err)
	}
	if updated.Name != "김마루" {
		t.Fatalf("updated.Name = %q, want 김마루", updated.Name)
	}
}
