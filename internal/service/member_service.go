// Package service contains application business workflows.
package service

import (
	"context"
	"errors"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type MemberRepository interface {
	Create(context.Context, repository.CreateMemberParams) (domain.Member, error)
	Update(context.Context, repository.UpdateMemberParams) (domain.Member, error)
	Get(context.Context, int64) (domain.Member, error)
	Search(context.Context, string, int) ([]domain.Member, error)
}

type MemberService struct {
	members MemberRepository
}

type MemberInput struct {
	MemberNo   string
	Name       string
	GenderCode string
	BirthDate  string
	Phone      string
	Note       string
}

func NewMemberService(members MemberRepository) *MemberService {
	return &MemberService{members: members}
}

func (s *MemberService) Create(ctx context.Context, input MemberInput) (domain.Member, error) {
	if err := validateMemberInput(input); err != nil {
		return domain.Member{}, err
	}
	return s.members.Create(ctx, repository.CreateMemberParams(input))
}

func (s *MemberService) Update(ctx context.Context, id int64, input MemberInput) (domain.Member, error) {
	if id <= 0 {
		return domain.Member{}, errors.New("member id is required")
	}
	if err := validateMemberInput(input); err != nil {
		return domain.Member{}, err
	}
	params := repository.UpdateMemberParams{
		ID:         id,
		MemberNo:   input.MemberNo,
		Name:       input.Name,
		GenderCode: input.GenderCode,
		BirthDate:  input.BirthDate,
		Phone:      input.Phone,
		Note:       input.Note,
	}
	return s.members.Update(ctx, params)
}

func (s *MemberService) Search(ctx context.Context, query string, limit int) ([]domain.Member, error) {
	return s.members.Search(ctx, strings.TrimSpace(query), limit)
}

func validateMemberInput(input MemberInput) error {
	if strings.TrimSpace(input.Name) == "" {
		return errors.New("member name is required")
	}
	return nil
}
