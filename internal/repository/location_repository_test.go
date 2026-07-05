package repository

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
)

func TestLocationRepositoryCreateListAndUpdate(t *testing.T) {
	ctx := context.Background()
	repo := NewLocationRepository(newTestDB(t))

	location, err := repo.Create(ctx, CreateLocationParams{
		Name:        "2층 다용도실",
		Building:    "본관",
		Floor:       "2층",
		Type:        domain.LocationTypeHall,
		IsClassroom: true,
		IsActive:    true,
		Note:        "행사와 강좌 겸용",
	})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if location.ID == 0 {
		t.Fatal("location.ID = 0, want generated ID")
	}
	if !location.IsClassroom {
		t.Fatal("location.IsClassroom = false, want true")
	}

	classrooms, err := repo.List(ctx, ListLocationsParams{ClassroomOnly: true})
	if err != nil {
		t.Fatalf("List() classrooms error = %v", err)
	}
	if len(classrooms) != 1 || classrooms[0].Name != "2층 다용도실" {
		t.Fatalf("classrooms = %+v, want created classroom-capable location", classrooms)
	}

	updated, err := repo.Update(ctx, UpdateLocationParams{
		ID:          location.ID,
		Name:        "2층 다목적실",
		Building:    "본관",
		Floor:       "2층",
		Type:        domain.LocationTypeHall,
		IsClassroom: false,
		IsActive:    false,
		Note:        "공사 중",
	})
	if err != nil {
		t.Fatalf("Update() error = %v", err)
	}
	if updated.Name != "2층 다목적실" || updated.IsClassroom || updated.IsActive {
		t.Fatalf("updated = %+v, want renamed inactive non-classroom", updated)
	}

	visible, err := repo.List(ctx, ListLocationsParams{})
	if err != nil {
		t.Fatalf("List() active error = %v", err)
	}
	if len(visible) != 0 {
		t.Fatalf("visible = %+v, want inactive location hidden by default", visible)
	}

	all, err := repo.List(ctx, ListLocationsParams{IncludeInactive: true})
	if err != nil {
		t.Fatalf("List() all error = %v", err)
	}
	if len(all) != 1 || all[0].ID != location.ID {
		t.Fatalf("all = %+v, want inactive location included", all)
	}
}
