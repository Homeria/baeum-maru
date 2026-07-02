package service

import (
	"context"
	"errors"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type CourseRepository interface {
	CreateOffering(context.Context, repository.CreateCourseOfferingParams) (domain.CourseOffering, error)
	UpdateOffering(context.Context, repository.UpdateCourseOfferingParams) (domain.CourseOffering, error)
	ListOfferings(context.Context, int) ([]domain.CourseOffering, error)
}

type CourseService struct {
	courses CourseRepository
}

type CourseOfferingInput struct {
	TermName       string
	CategoryName   string
	CourseTitle    string
	InstructorName string
	ClassroomName  string
	Capacity       int
	Weekday        int
	StartTime      string
	EndTime        string
	Note           string
}

func NewCourseService(courses CourseRepository) *CourseService {
	return &CourseService{courses: courses}
}

func (s *CourseService) CreateOffering(ctx context.Context, input CourseOfferingInput) (domain.CourseOffering, error) {
	if err := validateCourseOfferingInput(input); err != nil {
		return domain.CourseOffering{}, err
	}
	return s.courses.CreateOffering(ctx, repository.CreateCourseOfferingParams(input))
}

func (s *CourseService) UpdateOffering(ctx context.Context, id int64, input CourseOfferingInput) (domain.CourseOffering, error) {
	if id <= 0 {
		return domain.CourseOffering{}, errors.New("course offering id is required")
	}
	if err := validateCourseOfferingInput(input); err != nil {
		return domain.CourseOffering{}, err
	}
	params := repository.UpdateCourseOfferingParams{
		ID:             id,
		TermName:       input.TermName,
		CategoryName:   input.CategoryName,
		CourseTitle:    input.CourseTitle,
		InstructorName: input.InstructorName,
		ClassroomName:  input.ClassroomName,
		Capacity:       input.Capacity,
		Weekday:        input.Weekday,
		StartTime:      input.StartTime,
		EndTime:        input.EndTime,
		Note:           input.Note,
	}
	return s.courses.UpdateOffering(ctx, params)
}

func (s *CourseService) ListOfferings(ctx context.Context, limit int) ([]domain.CourseOffering, error) {
	return s.courses.ListOfferings(ctx, limit)
}

func validateCourseOfferingInput(input CourseOfferingInput) error {
	if strings.TrimSpace(input.CourseTitle) == "" {
		return errors.New("course title is required")
	}
	if input.Capacity < 0 {
		return errors.New("course capacity must be zero or greater")
	}
	if input.Weekday < 0 || input.Weekday > 6 {
		return errors.New("course weekday must be between 0 and 6")
	}
	if strings.TrimSpace(input.StartTime) == "" {
		return errors.New("course start time is required")
	}
	if strings.TrimSpace(input.EndTime) == "" {
		return errors.New("course end time is required")
	}
	if input.StartTime >= input.EndTime {
		return errors.New("course end time must be after start time")
	}
	return nil
}
