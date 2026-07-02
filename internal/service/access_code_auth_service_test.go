package service

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

func TestAccessCodeAuthServiceIssuesAndAuthenticatesCode(t *testing.T) {
	repo := newFakeAccessCodeRepository()
	service := NewAccessCodeAuthService(repo, "secret")
	now := time.Date(2026, 7, 2, 10, 0, 0, 0, time.UTC)
	service.now = func() time.Time { return now }

	issued, err := service.IssueAccessCode(context.Background(), AccessCodeIssueInput{
		DisplayName: "김접수",
		Role:        domain.UserRoleTemporaryStaff,
		ExpiresAt:   now.Add(4 * time.Hour),
	})
	if err != nil {
		t.Fatalf("IssueAccessCode() error = %v", err)
	}
	if issued.Code == "" {
		t.Fatal("issued.Code = empty, want generated code")
	}

	authenticated, err := service.AuthenticateAccessCode(context.Background(), issued.Code)
	if err != nil {
		t.Fatalf("AuthenticateAccessCode() error = %v", err)
	}
	if authenticated.UserID != issued.User.ID {
		t.Fatalf("UserID = %d, want %d", authenticated.UserID, issued.User.ID)
	}
	if !repo.markUsedCalled {
		t.Fatal("markUsedCalled = false, want true")
	}
}

func TestAccessCodeAuthServiceRejectsExpiredCode(t *testing.T) {
	repo := newFakeAccessCodeRepository()
	service := NewAccessCodeAuthService(repo, "secret")
	now := time.Date(2026, 7, 2, 10, 0, 0, 0, time.UTC)
	service.now = func() time.Time { return now }

	code := "ABCD-2345"
	repo.principal = domain.AccessCodePrincipal{
		User: domain.User{
			ID:          1,
			DisplayName: "김접수",
			Role:        domain.UserRoleStaff,
			Status:      domain.UserStatusActive,
		},
		AccessCode: domain.AccessCode{
			ID:        2,
			UserID:    1,
			CodeHash:  service.hashCode(code),
			Status:    domain.AccessCodeStatusActive,
			ExpiresAt: now.Add(-time.Minute).Format(time.RFC3339),
		},
	}

	if _, err := service.AuthenticateAccessCode(context.Background(), code); err == nil {
		t.Fatal("AuthenticateAccessCode() error = nil, want expired error")
	}
}

type fakeAccessCodeRepository struct {
	nextUserID     int64
	nextCodeID     int64
	principal      domain.AccessCodePrincipal
	markUsedCalled bool
}

func newFakeAccessCodeRepository() *fakeAccessCodeRepository {
	return &fakeAccessCodeRepository{
		nextUserID: 1,
		nextCodeID: 1,
	}
}

func (f *fakeAccessCodeRepository) CreateUser(_ context.Context, params repository.CreateAccessUserParams) (domain.User, error) {
	user := domain.User{
		ID:          f.nextUserID,
		Username:    params.Username,
		DisplayName: params.DisplayName,
		Role:        params.Role,
		Status:      domain.UserStatusActive,
		ExpiresAt:   params.ExpiresAt,
	}
	f.nextUserID++
	f.principal.User = user
	return user, nil
}

func (f *fakeAccessCodeRepository) CreateAccessCode(_ context.Context, params repository.CreateAccessCodeParams) (domain.AccessCode, error) {
	code := domain.AccessCode{
		ID:        f.nextCodeID,
		UserID:    params.UserID,
		CodeHash:  params.CodeHash,
		Status:    domain.AccessCodeStatusActive,
		ExpiresAt: params.ExpiresAt,
	}
	f.nextCodeID++
	f.principal.AccessCode = code
	return code, nil
}

func (f *fakeAccessCodeRepository) FindPrincipalByCodeHash(_ context.Context, codeHash string) (domain.AccessCodePrincipal, error) {
	if f.principal.AccessCode.CodeHash != codeHash {
		return domain.AccessCodePrincipal{}, errors.New("not found")
	}
	return f.principal, nil
}

func (f *fakeAccessCodeRepository) MarkAccessCodeUsed(context.Context, int64, int64, string) error {
	f.markUsedCalled = true
	return nil
}
