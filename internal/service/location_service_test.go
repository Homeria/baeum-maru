package service

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type fakeLocationRepository struct {
	created repository.CreateLocationParams
	updated repository.UpdateLocationParams
	listed  repository.ListLocationsParams
}

func (f *fakeLocationRepository) Create(_ context.Context, params repository.CreateLocationParams) (domain.Location, error) {
	f.created = params
	return domain.Location{ID: 1, Name: params.Name, Type: params.Type, IsClassroom: params.IsClassroom, IsActive: params.IsActive}, nil
}

func (f *fakeLocationRepository) Update(_ context.Context, params repository.UpdateLocationParams) (domain.Location, error) {
	f.updated = params
	return domain.Location{ID: params.ID, Name: params.Name, Type: params.Type, IsClassroom: params.IsClassroom, IsActive: params.IsActive}, nil
}

func (f *fakeLocationRepository) Get(_ context.Context, id int64) (domain.Location, error) {
	return domain.Location{ID: id}, nil
}

func (f *fakeLocationRepository) List(_ context.Context, params repository.ListLocationsParams) ([]domain.Location, error) {
	f.listed = params
	return nil, nil
}

func TestLocationServiceRejectsInvalidLocation(t *testing.T) {
	service := NewLocationService(&fakeLocationRepository{})

	if _, err := service.Create(context.Background(), LocationInput{}); err == nil {
		t.Fatal("Create() error = nil, want validation error")
	}
	if _, err := service.Create(context.Background(), LocationInput{Name: "사무실", Type: "custom_role", IsActive: true}); err != nil {
		t.Fatalf("Create() custom role error = %v, want nil", err)
	}
}

func TestLocationServiceCreatesClassroomLocation(t *testing.T) {
	repo := &fakeLocationRepository{}
	service := NewLocationService(repo)

	location, err := service.Create(context.Background(), LocationInput{
		Name:     "101호",
		Type:     domain.LocationTypeClassroom,
		IsActive: true,
	})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if !location.IsClassroom {
		t.Fatal("location.IsClassroom = false, want true")
	}
	if repo.created.Type != domain.LocationTypeClassroom || !repo.created.IsClassroom {
		t.Fatalf("created = %+v, want classroom type and flag", repo.created)
	}
}

func TestLocationServiceListsClassrooms(t *testing.T) {
	repo := &fakeLocationRepository{}
	service := NewLocationService(repo)

	if _, err := service.ListClassrooms(context.Background(), 20); err != nil {
		t.Fatalf("ListClassrooms() error = %v", err)
	}
	if !repo.listed.ClassroomOnly || repo.listed.Limit != 20 {
		t.Fatalf("listed = %+v, want classroom-only limit", repo.listed)
	}
}
