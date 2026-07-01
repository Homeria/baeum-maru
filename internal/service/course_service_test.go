package service

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type fakeCourseRepository struct {
	created repository.CreateCourseOfferingParams
}

func (f *fakeCourseRepository) CreateOffering(_ context.Context, params repository.CreateCourseOfferingParams) (domain.CourseOffering, error) {
	f.created = params
	return domain.CourseOffering{ID: 1, CourseTitle: params.CourseTitle}, nil
}

func (f *fakeCourseRepository) ListOfferings(_ context.Context, _ int) ([]domain.CourseOffering, error) {
	return nil, nil
}

func TestCourseServiceRejectsInvalidOffering(t *testing.T) {
	service := NewCourseService(&fakeCourseRepository{})

	if _, err := service.CreateOffering(context.Background(), CourseOfferingInput{}); err == nil {
		t.Fatal("CreateOffering() error = nil, want validation error")
	}
}

func TestCourseServiceCreatesOffering(t *testing.T) {
	repo := &fakeCourseRepository{}
	service := NewCourseService(repo)

	offering, err := service.CreateOffering(context.Background(), CourseOfferingInput{
		CourseTitle: "요가 기초",
		Capacity:    10,
		Weekday:     1,
		StartTime:   "09:00",
		EndTime:     "10:00",
	})
	if err != nil {
		t.Fatalf("CreateOffering() error = %v", err)
	}
	if offering.CourseTitle != "요가 기초" {
		t.Fatalf("CourseTitle = %q, want 요가 기초", offering.CourseTitle)
	}
	if repo.created.CourseTitle != "요가 기초" {
		t.Fatalf("repo.created.CourseTitle = %q, want 요가 기초", repo.created.CourseTitle)
	}
}
