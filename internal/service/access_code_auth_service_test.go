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
	if authenticated.AccessCodeID != issued.AccessCode.ID {
		t.Fatalf("AccessCodeID = %d, want %d", authenticated.AccessCodeID, issued.AccessCode.ID)
	}
	if err := service.ValidateAccessSession(context.Background(), issued.User.ID, issued.AccessCode.ID); err != nil {
		t.Fatalf("ValidateAccessSession() error = %v", err)
	}
	if !repo.markUsedCalled {
		t.Fatal("markUsedCalled = false, want true")
	}

	summaries, err := service.ListRecentAccessCodes(context.Background(), 10)
	if err != nil {
		t.Fatalf("ListRecentAccessCodes() error = %v", err)
	}
	if len(summaries) != 1 || summaries[0].DisplayName != "김접수" {
		t.Fatalf("summaries = %+v, want issued user", summaries)
	}
	if summaries[0].Code == "" {
		t.Fatalf("summaries[0].Code = empty, want display code")
	}
	if err := service.RevokeAccessCode(context.Background(), issued.AccessCode.ID); err != nil {
		t.Fatalf("RevokeAccessCode() error = %v", err)
	}
	if !repo.revokeCalled {
		t.Fatal("revokeCalled = false, want true")
	}
}

func TestAccessCodeAuthServiceExtendsCode(t *testing.T) {
	repo := newFakeAccessCodeRepository()
	service := NewAccessCodeAuthService(repo, "secret")
	now := time.Date(2026, 7, 2, 10, 0, 0, 0, time.UTC)
	service.now = func() time.Time { return now }

	if err := service.ExtendAccessCode(context.Background(), 7, now.Add(2*time.Hour)); err != nil {
		t.Fatalf("ExtendAccessCode() error = %v", err)
	}
	if !repo.extendCalled {
		t.Fatal("extendCalled = false, want true")
	}
	if repo.extendedID != 7 {
		t.Fatalf("extendedID = %d, want 7", repo.extendedID)
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

func TestAccessCodeAuthServiceRejectsRevokedSession(t *testing.T) {
	repo := newFakeAccessCodeRepository()
	service := NewAccessCodeAuthService(repo, "secret")
	now := time.Date(2026, 7, 2, 10, 0, 0, 0, time.UTC)
	service.now = func() time.Time { return now }
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
			Status:    domain.AccessCodeStatusRevoked,
			RevokedAt: now.Format(time.RFC3339),
			ExpiresAt: now.Add(time.Hour).Format(time.RFC3339),
		},
	}

	if err := service.ValidateAccessSession(context.Background(), 1, 2); err == nil {
		t.Fatal("ValidateAccessSession() error = nil, want revoked error")
	}
}

type fakeAccessCodeRepository struct {
	nextUserID     int64
	nextCodeID     int64
	principal      domain.AccessCodePrincipal
	markUsedCalled bool
	revokeCalled   bool
	extendCalled   bool
	extendedID     int64
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
		ID:          f.nextCodeID,
		UserID:      params.UserID,
		CodeHash:    params.CodeHash,
		DisplayCode: params.DisplayCode,
		Status:      domain.AccessCodeStatusActive,
		ExpiresAt:   params.ExpiresAt,
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

func (f *fakeAccessCodeRepository) FindPrincipalByAccessCodeID(_ context.Context, userID int64, accessCodeID int64) (domain.AccessCodePrincipal, error) {
	if f.principal.User.ID != userID || f.principal.AccessCode.ID != accessCodeID {
		return domain.AccessCodePrincipal{}, errors.New("not found")
	}
	return f.principal, nil
}

func (f *fakeAccessCodeRepository) MarkAccessCodeUsed(context.Context, int64, int64, string) error {
	f.markUsedCalled = true
	return nil
}

func (f *fakeAccessCodeRepository) ListRecentAccessCodes(context.Context, int) ([]repository.AccessCodeListItem, error) {
	if f.principal.User.ID == 0 || f.principal.AccessCode.ID == 0 {
		return nil, nil
	}
	return []repository.AccessCodeListItem{{
		User:       f.principal.User,
		AccessCode: f.principal.AccessCode,
	}}, nil
}

func (f *fakeAccessCodeRepository) RevokeAccessCode(context.Context, int64, string) error {
	f.revokeCalled = true
	return nil
}

func (f *fakeAccessCodeRepository) ExtendAccessCode(_ context.Context, accessCodeID int64, expiresAt string) error {
	f.extendCalled = true
	f.extendedID = accessCodeID
	f.principal.AccessCode.ExpiresAt = expiresAt
	f.principal.User.ExpiresAt = expiresAt
	return nil
}
