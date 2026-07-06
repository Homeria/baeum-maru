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
		Roles:       []string{domain.LocationTypeHall, domain.LocationTypeClassroom},
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
	if !hasTestLocationRole(location.Roles, domain.LocationTypeHall) || !hasTestLocationRole(location.Roles, domain.LocationTypeClassroom) {
		t.Fatalf("location.Roles = %+v, want event and classroom roles", location.Roles)
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
		Roles:       []string{domain.LocationTypeHall},
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
	if hasTestLocationRole(updated.Roles, domain.LocationTypeClassroom) || !hasTestLocationRole(updated.Roles, domain.LocationTypeHall) {
		t.Fatalf("updated.Roles = %+v, want event role only", updated.Roles)
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

func TestLocationRepositoryManagesRoles(t *testing.T) {
	ctx := context.Background()
	repo := NewLocationRepository(newTestDB(t))

	created, err := repo.CreateRole(ctx, "상담")
	if err != nil {
		t.Fatalf("CreateRole() error = %v", err)
	}
	if created.ID == 0 || created.Name != "상담" || !created.IsActive {
		t.Fatalf("created = %+v, want active custom role", created)
	}

	roles, err := repo.ListRoles(ctx, ListLocationRolesParams{})
	if err != nil {
		t.Fatalf("ListRoles() error = %v", err)
	}
	if !hasTestDomainRole(roles, "상담") {
		t.Fatalf("roles = %+v, want 상담 role", roles)
	}

	if err := repo.DeactivateRole(ctx, created.ID); err != nil {
		t.Fatalf("DeactivateRole() error = %v", err)
	}
	activeRoles, err := repo.ListRoles(ctx, ListLocationRolesParams{})
	if err != nil {
		t.Fatalf("ListRoles() active error = %v", err)
	}
	if hasTestDomainRole(activeRoles, "상담") {
		t.Fatalf("activeRoles = %+v, want 상담 hidden", activeRoles)
	}
	allRoles, err := repo.ListRoles(ctx, ListLocationRolesParams{IncludeInactive: true})
	if err != nil {
		t.Fatalf("ListRoles() all error = %v", err)
	}
	if !hasTestDomainRole(allRoles, "상담") {
		t.Fatalf("allRoles = %+v, want inactive 상담 included", allRoles)
	}

	reactivatedRole, err := repo.CreateRole(ctx, "상담")
	if err != nil {
		t.Fatalf("CreateRole() reactivate error = %v", err)
	}
	updatedRole, err := repo.UpdateRole(ctx, reactivatedRole.ID, "안내")
	if err != nil {
		t.Fatalf("UpdateRole() error = %v", err)
	}
	if updatedRole.Name != "안내" {
		t.Fatalf("updatedRole = %+v, want 안내", updatedRole)
	}
	if err := repo.ActivateRole(ctx, updatedRole.ID); err != nil {
		t.Fatalf("ActivateRole() error = %v", err)
	}
	if err := repo.DeleteRole(ctx, updatedRole.ID); err != nil {
		t.Fatalf("DeleteRole() error = %v", err)
	}
	deletedRoles, err := repo.ListRoles(ctx, ListLocationRolesParams{IncludeInactive: true})
	if err != nil {
		t.Fatalf("ListRoles() deleted error = %v", err)
	}
	if hasTestDomainRole(deletedRoles, "안내") {
		t.Fatalf("deletedRoles = %+v, want 안내 deleted", deletedRoles)
	}
}

func TestLocationRepositoryManagesBuildings(t *testing.T) {
	ctx := context.Background()
	repo := NewLocationRepository(newTestDB(t))

	created, err := repo.CreateBuilding(ctx, "본관")
	if err != nil {
		t.Fatalf("CreateBuilding() error = %v", err)
	}
	if created.ID == 0 || created.Name != "본관" || !created.IsActive {
		t.Fatalf("created = %+v, want active building", created)
	}

	buildings, err := repo.ListBuildings(ctx, ListBuildingsParams{})
	if err != nil {
		t.Fatalf("ListBuildings() error = %v", err)
	}
	if !hasTestBuilding(buildings, "본관") {
		t.Fatalf("buildings = %+v, want 본관", buildings)
	}

	if err := repo.DeactivateBuilding(ctx, created.ID); err != nil {
		t.Fatalf("DeactivateBuilding() error = %v", err)
	}
	activeBuildings, err := repo.ListBuildings(ctx, ListBuildingsParams{})
	if err != nil {
		t.Fatalf("ListBuildings() active error = %v", err)
	}
	if hasTestBuilding(activeBuildings, "본관") {
		t.Fatalf("activeBuildings = %+v, want 본관 hidden", activeBuildings)
	}

	if err := repo.ActivateBuilding(ctx, created.ID); err != nil {
		t.Fatalf("ActivateBuilding() error = %v", err)
	}
	reactivated, err := repo.ListBuildings(ctx, ListBuildingsParams{})
	if err != nil {
		t.Fatalf("ListBuildings() reactivated error = %v", err)
	}
	if !hasTestBuilding(reactivated, "본관") {
		t.Fatalf("reactivated = %+v, want 본관 visible", reactivated)
	}

	updated, err := repo.UpdateBuilding(ctx, created.ID, "신관")
	if err != nil {
		t.Fatalf("UpdateBuilding() error = %v", err)
	}
	if updated.Name != "신관" {
		t.Fatalf("updated = %+v, want 신관", updated)
	}
	if err := repo.DeleteBuilding(ctx, updated.ID); err != nil {
		t.Fatalf("DeleteBuilding() error = %v", err)
	}
	deletedBuildings, err := repo.ListBuildings(ctx, ListBuildingsParams{IncludeInactive: true})
	if err != nil {
		t.Fatalf("ListBuildings() deleted error = %v", err)
	}
	if hasTestBuilding(deletedBuildings, "신관") {
		t.Fatalf("deletedBuildings = %+v, want 신관 deleted", deletedBuildings)
	}
}

func TestLocationRepositoryManagesBuildingFloors(t *testing.T) {
	ctx := context.Background()
	repo := NewLocationRepository(newTestDB(t))

	building, err := repo.CreateBuilding(ctx, "본관")
	if err != nil {
		t.Fatalf("CreateBuilding() error = %v", err)
	}
	created, err := repo.CreateBuildingFloor(ctx, building.ID, "2층")
	if err != nil {
		t.Fatalf("CreateBuildingFloor() error = %v", err)
	}
	if created.ID == 0 || created.BuildingID != building.ID || created.Label != "2층" || !created.IsActive {
		t.Fatalf("created = %+v, want active floor", created)
	}

	floors, err := repo.ListBuildingFloors(ctx, ListBuildingFloorsParams{BuildingID: building.ID})
	if err != nil {
		t.Fatalf("ListBuildingFloors() error = %v", err)
	}
	if !hasTestBuildingFloor(floors, "2층") {
		t.Fatalf("floors = %+v, want 2층", floors)
	}

	if err := repo.DeactivateBuildingFloor(ctx, created.ID); err != nil {
		t.Fatalf("DeactivateBuildingFloor() error = %v", err)
	}
	activeFloors, err := repo.ListBuildingFloors(ctx, ListBuildingFloorsParams{BuildingID: building.ID})
	if err != nil {
		t.Fatalf("ListBuildingFloors() active error = %v", err)
	}
	if hasTestBuildingFloor(activeFloors, "2층") {
		t.Fatalf("activeFloors = %+v, want 2층 hidden", activeFloors)
	}

	if err := repo.ActivateBuildingFloor(ctx, created.ID); err != nil {
		t.Fatalf("ActivateBuildingFloor() error = %v", err)
	}
	reactivated, err := repo.ListBuildingFloors(ctx, ListBuildingFloorsParams{BuildingID: building.ID})
	if err != nil {
		t.Fatalf("ListBuildingFloors() reactivated error = %v", err)
	}
	if !hasTestBuildingFloor(reactivated, "2층") {
		t.Fatalf("reactivated = %+v, want 2층 visible", reactivated)
	}
}

func hasTestLocationRole(roles []string, want string) bool {
	for _, role := range roles {
		if role == want {
			return true
		}
	}
	return false
}

func hasTestBuilding(buildings []domain.Building, want string) bool {
	for _, building := range buildings {
		if building.Name == want {
			return true
		}
	}
	return false
}

func hasTestBuildingFloor(floors []domain.BuildingFloor, want string) bool {
	for _, floor := range floors {
		if floor.Label == want {
			return true
		}
	}
	return false
}

func hasTestDomainRole(roles []domain.LocationRole, want string) bool {
	for _, role := range roles {
		if role.Name == want {
			return true
		}
	}
	return false
}
