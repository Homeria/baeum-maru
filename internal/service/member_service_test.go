package service

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type fakeMemberRepository struct {
	created repository.CreateMemberParams
}

func (f *fakeMemberRepository) Create(_ context.Context, params repository.CreateMemberParams) (domain.Member, error) {
	f.created = params
	return domain.Member{ID: 1, Name: params.Name}, nil
}

func (f *fakeMemberRepository) Update(_ context.Context, params repository.UpdateMemberParams) (domain.Member, error) {
	return domain.Member{ID: params.ID, Name: params.Name}, nil
}

func (f *fakeMemberRepository) Get(_ context.Context, id int64) (domain.Member, error) {
	return domain.Member{ID: id}, nil
}

func (f *fakeMemberRepository) Search(_ context.Context, _ string, _ int) ([]domain.Member, error) {
	return nil, nil
}

func TestMemberServiceRejectsMissingName(t *testing.T) {
	service := NewMemberService(&fakeMemberRepository{})

	if _, err := service.Create(context.Background(), MemberInput{}); err == nil {
		t.Fatal("Create() error = nil, want validation error")
	}
}

func TestMemberServiceCreatesMember(t *testing.T) {
	repo := &fakeMemberRepository{}
	service := NewMemberService(repo)

	member, err := service.Create(context.Background(), MemberInput{Name: "김배움"})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if member.Name != "김배움" {
		t.Fatalf("member.Name = %q, want 김배움", member.Name)
	}
	if repo.created.Name != "김배움" {
		t.Fatalf("repo.created.Name = %q, want 김배움", repo.created.Name)
	}
}
