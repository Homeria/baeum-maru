package service

import (
	"context"
	"crypto/hmac"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

const accessCodeAlphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"

type AccessCodeRepository interface {
	CreateUser(context.Context, repository.CreateAccessUserParams) (domain.User, error)
	CreateAccessCode(context.Context, repository.CreateAccessCodeParams) (domain.AccessCode, error)
	FindPrincipalByCodeHash(context.Context, string) (domain.AccessCodePrincipal, error)
	MarkAccessCodeUsed(context.Context, int64, int64, string) error
}

type AccessCodeAuthService struct {
	repository AccessCodeRepository
	secret     string
	now        func() time.Time
}

type AccessCodeIssueInput struct {
	DisplayName string
	Affiliation string
	ContactNote string
	Role        string
	ExpiresAt   time.Time
	Label       string
	Note        string
}

type IssuedAccessCode struct {
	User       domain.User
	AccessCode domain.AccessCode
	Code       string
}

type AuthenticatedUser struct {
	UserID      int64
	DisplayName string
	Role        string
}

func NewAccessCodeAuthService(repository AccessCodeRepository, secret string) *AccessCodeAuthService {
	return &AccessCodeAuthService{
		repository: repository,
		secret:     secret,
		now:        time.Now,
	}
}

func (s *AccessCodeAuthService) IssueAccessCode(ctx context.Context, input AccessCodeIssueInput) (IssuedAccessCode, error) {
	if s == nil || s.repository == nil {
		return IssuedAccessCode{}, errors.New("access code repository is not configured")
	}
	if strings.TrimSpace(s.secret) == "" {
		return IssuedAccessCode{}, errors.New("access code secret is required")
	}
	if strings.TrimSpace(input.DisplayName) == "" {
		return IssuedAccessCode{}, errors.New("display name is required")
	}
	role := normalizeAccessRole(input.Role)
	if role == "" {
		return IssuedAccessCode{}, errors.New("invalid access role")
	}
	if input.ExpiresAt.IsZero() || !input.ExpiresAt.After(s.now()) {
		return IssuedAccessCode{}, errors.New("access code expiration must be in the future")
	}

	code, err := generateAccessCode(8)
	if err != nil {
		return IssuedAccessCode{}, err
	}
	expiresAt := input.ExpiresAt.UTC().Format(time.RFC3339)
	user, err := s.repository.CreateUser(ctx, repository.CreateAccessUserParams{
		Username:    accessUsername(),
		DisplayName: input.DisplayName,
		Affiliation: input.Affiliation,
		ContactNote: input.ContactNote,
		Role:        role,
		ExpiresAt:   expiresAt,
	})
	if err != nil {
		return IssuedAccessCode{}, err
	}
	accessCode, err := s.repository.CreateAccessCode(ctx, repository.CreateAccessCodeParams{
		UserID:    user.ID,
		CodeHash:  s.hashCode(code),
		Label:     input.Label,
		ExpiresAt: expiresAt,
		Note:      input.Note,
	})
	if err != nil {
		return IssuedAccessCode{}, err
	}
	return IssuedAccessCode{
		User:       user,
		AccessCode: accessCode,
		Code:       formatAccessCode(code),
	}, nil
}

func (s *AccessCodeAuthService) AuthenticateAccessCode(ctx context.Context, code string) (AuthenticatedUser, error) {
	if s == nil || s.repository == nil {
		return AuthenticatedUser{}, errors.New("access code repository is not configured")
	}
	normalized := normalizeAccessCode(code)
	if len(normalized) < 6 {
		return AuthenticatedUser{}, errors.New("access code is invalid")
	}
	principal, err := s.repository.FindPrincipalByCodeHash(ctx, s.hashCode(normalized))
	if err != nil {
		return AuthenticatedUser{}, errors.New("access code is invalid")
	}
	now := s.now().UTC()
	if principal.User.Status != domain.UserStatusActive {
		return AuthenticatedUser{}, errors.New("user is not active")
	}
	if principal.User.ExpiresAt != "" && !timeStringAfter(principal.User.ExpiresAt, now) {
		return AuthenticatedUser{}, errors.New("user is expired")
	}
	if principal.AccessCode.Status != domain.AccessCodeStatusActive {
		return AuthenticatedUser{}, errors.New("access code is not active")
	}
	if principal.AccessCode.RevokedAt != "" {
		return AuthenticatedUser{}, errors.New("access code is revoked")
	}
	if !timeStringAfter(principal.AccessCode.ExpiresAt, now) {
		return AuthenticatedUser{}, errors.New("access code is expired")
	}
	if err := s.repository.MarkAccessCodeUsed(ctx, principal.AccessCode.ID, principal.User.ID, now.Format(time.RFC3339)); err != nil {
		return AuthenticatedUser{}, err
	}
	return AuthenticatedUser{
		UserID:      principal.User.ID,
		DisplayName: principal.User.DisplayName,
		Role:        principal.User.Role,
	}, nil
}

func (s *AccessCodeAuthService) hashCode(code string) string {
	mac := hmac.New(sha256.New, []byte(s.secret))
	_, _ = mac.Write([]byte(normalizeAccessCode(code)))
	return hex.EncodeToString(mac.Sum(nil))
}

func normalizeAccessRole(role string) string {
	switch strings.TrimSpace(role) {
	case "", domain.UserRoleStaff:
		return domain.UserRoleStaff
	case domain.UserRoleTemporaryStaff:
		return domain.UserRoleTemporaryStaff
	case domain.UserRoleViewer:
		return domain.UserRoleViewer
	default:
		return ""
	}
}

func normalizeAccessCode(code string) string {
	replacer := strings.NewReplacer("-", "", " ", "", "\t", "", "\r", "", "\n", "")
	return strings.ToUpper(replacer.Replace(strings.TrimSpace(code)))
}

func formatAccessCode(code string) string {
	code = normalizeAccessCode(code)
	if len(code) <= 4 {
		return code
	}
	return code[:4] + "-" + code[4:]
}

func generateAccessCode(length int) (string, error) {
	if length <= 0 {
		return "", errors.New("access code length must be positive")
	}
	data := make([]byte, length)
	if _, err := rand.Read(data); err != nil {
		return "", fmt.Errorf("generate access code: %w", err)
	}
	var builder strings.Builder
	builder.Grow(length)
	for _, value := range data {
		builder.WriteByte(accessCodeAlphabet[int(value)%len(accessCodeAlphabet)])
	}
	return builder.String(), nil
}

func accessUsername() string {
	code, err := generateAccessCode(12)
	if err != nil {
		return "access-user-" + time.Now().UTC().Format("20060102150405")
	}
	return "access-" + strings.ToLower(code)
}

func timeStringAfter(value string, now time.Time) bool {
	parsed, err := time.Parse(time.RFC3339, value)
	if err != nil {
		return false
	}
	return parsed.After(now)
}
