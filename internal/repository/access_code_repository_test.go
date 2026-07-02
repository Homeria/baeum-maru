package repository

import (
	"context"
	"testing"
	"time"

	"github.com/Homeria/baeum-maru/internal/domain"
)

func TestAccessCodeRepositoryCreatesAndFindsPrincipal(t *testing.T) {
	ctx := context.Background()
	repo := NewAccessCodeRepository(newTestDB(t))
	expiresAt := time.Now().UTC().Add(8 * time.Hour).Format(time.RFC3339)

	user, err := repo.CreateUser(ctx, CreateAccessUserParams{
		Username:    "access-test-user",
		DisplayName: "김접수",
		Affiliation: "접수팀",
		Role:        domain.UserRoleTemporaryStaff,
		ExpiresAt:   expiresAt,
	})
	if err != nil {
		t.Fatalf("CreateUser() error = %v", err)
	}
	if user.Role != domain.UserRoleTemporaryStaff {
		t.Fatalf("user.Role = %q, want temporary_staff", user.Role)
	}

	code, err := repo.CreateAccessCode(ctx, CreateAccessCodeParams{
		UserID:    user.ID,
		CodeHash:  "hash",
		Label:     "오전 접수",
		ExpiresAt: expiresAt,
	})
	if err != nil {
		t.Fatalf("CreateAccessCode() error = %v", err)
	}

	principal, err := repo.FindPrincipalByCodeHash(ctx, "hash")
	if err != nil {
		t.Fatalf("FindPrincipalByCodeHash() error = %v", err)
	}
	if principal.User.ID != user.ID || principal.AccessCode.ID != code.ID {
		t.Fatalf("principal = %+v, want user/code ids", principal)
	}
	if err := repo.MarkAccessCodeUsed(ctx, code.ID, user.ID, time.Now().UTC().Format(time.RFC3339)); err != nil {
		t.Fatalf("MarkAccessCodeUsed() error = %v", err)
	}
	used, err := repo.GetAccessCode(ctx, code.ID)
	if err != nil {
		t.Fatalf("GetAccessCode() error = %v", err)
	}
	if used.LastUsedAt == "" {
		t.Fatal("LastUsedAt = empty, want timestamp")
	}

	items, err := repo.ListRecentAccessCodes(ctx, 10)
	if err != nil {
		t.Fatalf("ListRecentAccessCodes() error = %v", err)
	}
	if len(items) != 1 || items[0].AccessCode.ID != code.ID {
		t.Fatalf("items = %+v, want created access code", items)
	}

	if err := repo.RevokeAccessCode(ctx, code.ID, time.Now().UTC().Format(time.RFC3339)); err != nil {
		t.Fatalf("RevokeAccessCode() error = %v", err)
	}
	revoked, err := repo.GetAccessCode(ctx, code.ID)
	if err != nil {
		t.Fatalf("GetAccessCode() revoked error = %v", err)
	}
	if revoked.Status != domain.AccessCodeStatusRevoked {
		t.Fatalf("revoked.Status = %q, want revoked", revoked.Status)
	}
}
