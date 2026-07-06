package service

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type fakeLocationRepository struct {
	created                repository.CreateLocationParams
	updated                repository.UpdateLocationParams
	listed                 repository.ListLocationsParams
	listedRoles            repository.ListLocationRolesParams
	listedBuildings        repository.ListBuildingsParams
	createdRole            string
	updatedRoleID          int64
	updatedRoleName        string
	deactivatedRoleID      int64
	activatedRoleID        int64
	deletedRoleID          int64
	createdBuilding        string
	updatedBuildingID      int64
	updatedBuildingName    string
	deactivatedBuildingID  int64
	deletedBuildingID      int64
	activatedBuildingID    int64
	listedFloors           repository.ListBuildingFloorsParams
	createdFloorBuildingID int64
	createdFloorLabel      string
	deletedFloorID         int64
	activatedFloorID       int64
}

func (f *fakeLocationRepository) Create(_ context.Context, params repository.CreateLocationParams) (domain.Location, error) {
	f.created = params
	return domain.Location{ID: 1, Name: params.Name, Type: params.Type, Roles: params.Roles, IsClassroom: params.IsClassroom, IsActive: params.IsActive}, nil
}

func (f *fakeLocationRepository) Update(_ context.Context, params repository.UpdateLocationParams) (domain.Location, error) {
	f.updated = params
	return domain.Location{ID: params.ID, Name: params.Name, Type: params.Type, Roles: params.Roles, IsClassroom: params.IsClassroom, IsActive: params.IsActive}, nil
}

func (f *fakeLocationRepository) Get(_ context.Context, id int64) (domain.Location, error) {
	return domain.Location{ID: id}, nil
}

func (f *fakeLocationRepository) List(_ context.Context, params repository.ListLocationsParams) ([]domain.Location, error) {
	f.listed = params
	return nil, nil
}

func (f *fakeLocationRepository) ListRoles(_ context.Context, params repository.ListLocationRolesParams) ([]domain.LocationRole, error) {
	f.listedRoles = params
	return []domain.LocationRole{{ID: 1, Name: "classroom", IsActive: true}}, nil
}

func (f *fakeLocationRepository) CreateRole(_ context.Context, name string) (domain.LocationRole, error) {
	f.createdRole = name
	return domain.LocationRole{ID: 2, Name: name, IsActive: true}, nil
}

func (f *fakeLocationRepository) UpdateRole(_ context.Context, id int64, name string) (domain.LocationRole, error) {
	f.updatedRoleID = id
	f.updatedRoleName = name
	return domain.LocationRole{ID: id, Name: name, IsActive: true}, nil
}

func (f *fakeLocationRepository) DeactivateRole(_ context.Context, id int64) error {
	f.deactivatedRoleID = id
	return nil
}

func (f *fakeLocationRepository) ActivateRole(_ context.Context, id int64) error {
	f.activatedRoleID = id
	return nil
}

func (f *fakeLocationRepository) DeleteRole(_ context.Context, id int64) error {
	f.deletedRoleID = id
	return nil
}

func (f *fakeLocationRepository) ListBuildings(_ context.Context, params repository.ListBuildingsParams) ([]domain.Building, error) {
	f.listedBuildings = params
	return []domain.Building{{ID: 1, Name: "본관", IsActive: true}}, nil
}

func (f *fakeLocationRepository) CreateBuilding(_ context.Context, name string) (domain.Building, error) {
	f.createdBuilding = name
	return domain.Building{ID: 2, Name: name, IsActive: true}, nil
}

func (f *fakeLocationRepository) UpdateBuilding(_ context.Context, id int64, name string) (domain.Building, error) {
	f.updatedBuildingID = id
	f.updatedBuildingName = name
	return domain.Building{ID: id, Name: name, IsActive: true}, nil
}

func (f *fakeLocationRepository) DeactivateBuilding(_ context.Context, id int64) error {
	f.deactivatedBuildingID = id
	return nil
}

func (f *fakeLocationRepository) ActivateBuilding(_ context.Context, id int64) error {
	f.activatedBuildingID = id
	return nil
}

func (f *fakeLocationRepository) DeleteBuilding(_ context.Context, id int64) error {
	f.deletedBuildingID = id
	return nil
}

func (f *fakeLocationRepository) ListBuildingFloors(_ context.Context, params repository.ListBuildingFloorsParams) ([]domain.BuildingFloor, error) {
	f.listedFloors = params
	return []domain.BuildingFloor{{ID: 1, BuildingID: params.BuildingID, Label: "2층", IsActive: true}}, nil
}

func (f *fakeLocationRepository) CreateBuildingFloor(_ context.Context, buildingID int64, label string) (domain.BuildingFloor, error) {
	f.createdFloorBuildingID = buildingID
	f.createdFloorLabel = label
	return domain.BuildingFloor{ID: 2, BuildingID: buildingID, Label: label, IsActive: true}, nil
}

func (f *fakeLocationRepository) DeactivateBuildingFloor(_ context.Context, id int64) error {
	f.deletedFloorID = id
	return nil
}

func (f *fakeLocationRepository) ActivateBuildingFloor(_ context.Context, id int64) error {
	f.activatedFloorID = id
	return nil
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

func TestLocationServiceCreatesMultiRoleLocation(t *testing.T) {
	repo := &fakeLocationRepository{}
	service := NewLocationService(repo)

	location, err := service.Create(context.Background(), LocationInput{
		Name:     "2층 다용도실",
		Roles:    []string{domain.LocationTypeHall, domain.LocationTypeClassroom},
		IsActive: true,
	})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if !location.IsClassroom || location.Type != domain.LocationTypeClassroom {
		t.Fatalf("location = %+v, want classroom-preferred multi-role location", location)
	}
	if len(repo.created.Roles) != 2 {
		t.Fatalf("created.Roles = %+v, want two roles", repo.created.Roles)
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

func TestLocationServiceManagesRoles(t *testing.T) {
	repo := &fakeLocationRepository{}
	service := NewLocationService(repo)

	roles, err := service.ListRoles(context.Background(), LocationRoleListInput{Limit: 20})
	if err != nil {
		t.Fatalf("ListRoles() error = %v", err)
	}
	if len(roles) != 1 || repo.listedRoles.Limit != 20 {
		t.Fatalf("roles = %+v listedRoles = %+v, want listed roles", roles, repo.listedRoles)
	}

	created, err := service.CreateRole(context.Background(), " 행사 ")
	if err != nil {
		t.Fatalf("CreateRole() error = %v", err)
	}
	if created.Name != "행사" || repo.createdRole != "행사" {
		t.Fatalf("created = %+v createdRole = %q, want trimmed role", created, repo.createdRole)
	}

	updated, err := service.UpdateRole(context.Background(), 7, " 안내 ")
	if err != nil {
		t.Fatalf("UpdateRole() error = %v", err)
	}
	if updated.Name != "안내" || repo.updatedRoleID != 7 || repo.updatedRoleName != "안내" {
		t.Fatalf("updated = %+v repo = %+v, want trimmed updated role", updated, repo)
	}

	if err := service.DeactivateRole(context.Background(), 7); err != nil {
		t.Fatalf("DeactivateRole() error = %v", err)
	}
	if repo.deactivatedRoleID != 7 {
		t.Fatalf("deactivatedRoleID = %d, want 7", repo.deactivatedRoleID)
	}

	if err := service.ActivateRole(context.Background(), 7); err != nil {
		t.Fatalf("ActivateRole() error = %v", err)
	}
	if repo.activatedRoleID != 7 {
		t.Fatalf("activatedRoleID = %d, want 7", repo.activatedRoleID)
	}

	if err := service.DeleteRole(context.Background(), 7); err != nil {
		t.Fatalf("DeleteRole() error = %v", err)
	}
	if repo.deletedRoleID != 7 {
		t.Fatalf("deletedRoleID = %d, want 7", repo.deletedRoleID)
	}
}

func TestLocationServiceManagesBuildings(t *testing.T) {
	repo := &fakeLocationRepository{}
	service := NewLocationService(repo)

	buildings, err := service.ListBuildings(context.Background(), BuildingListInput{Limit: 20})
	if err != nil {
		t.Fatalf("ListBuildings() error = %v", err)
	}
	if len(buildings) != 1 || repo.listedBuildings.Limit != 20 {
		t.Fatalf("buildings = %+v listedBuildings = %+v, want listed buildings", buildings, repo.listedBuildings)
	}

	created, err := service.CreateBuilding(context.Background(), " 본관 ")
	if err != nil {
		t.Fatalf("CreateBuilding() error = %v", err)
	}
	if created.Name != "본관" || repo.createdBuilding != "본관" {
		t.Fatalf("created = %+v createdBuilding = %q, want trimmed building", created, repo.createdBuilding)
	}

	updated, err := service.UpdateBuilding(context.Background(), 7, " 신관 ")
	if err != nil {
		t.Fatalf("UpdateBuilding() error = %v", err)
	}
	if updated.Name != "신관" || repo.updatedBuildingID != 7 || repo.updatedBuildingName != "신관" {
		t.Fatalf("updated = %+v repo = %+v, want trimmed updated building", updated, repo)
	}

	if err := service.DeactivateBuilding(context.Background(), 7); err != nil {
		t.Fatalf("DeactivateBuilding() error = %v", err)
	}
	if repo.deactivatedBuildingID != 7 {
		t.Fatalf("deactivatedBuildingID = %d, want 7", repo.deactivatedBuildingID)
	}

	if err := service.DeleteBuilding(context.Background(), 7); err != nil {
		t.Fatalf("DeleteBuilding() error = %v", err)
	}
	if repo.deletedBuildingID != 7 {
		t.Fatalf("deletedBuildingID = %d, want 7", repo.deletedBuildingID)
	}

	if err := service.ActivateBuilding(context.Background(), 7); err != nil {
		t.Fatalf("ActivateBuilding() error = %v", err)
	}
	if repo.activatedBuildingID != 7 {
		t.Fatalf("activatedBuildingID = %d, want 7", repo.activatedBuildingID)
	}
}

func TestLocationServiceManagesBuildingFloors(t *testing.T) {
	repo := &fakeLocationRepository{}
	service := NewLocationService(repo)

	floors, err := service.ListBuildingFloors(context.Background(), BuildingFloorListInput{BuildingID: 3, Limit: 20})
	if err != nil {
		t.Fatalf("ListBuildingFloors() error = %v", err)
	}
	if len(floors) != 1 || repo.listedFloors.BuildingID != 3 || repo.listedFloors.Limit != 20 {
		t.Fatalf("floors = %+v listedFloors = %+v, want listed floors", floors, repo.listedFloors)
	}

	created, err := service.CreateBuildingFloor(context.Background(), 3, " 2층 ")
	if err != nil {
		t.Fatalf("CreateBuildingFloor() error = %v", err)
	}
	if created.Label != "2층" || repo.createdFloorBuildingID != 3 || repo.createdFloorLabel != "2층" {
		t.Fatalf("created = %+v repo = %+v, want trimmed floor", created, repo)
	}

	if err := service.DeleteBuildingFloor(context.Background(), 9); err != nil {
		t.Fatalf("DeleteBuildingFloor() error = %v", err)
	}
	if repo.deletedFloorID != 9 {
		t.Fatalf("deletedFloorID = %d, want 9", repo.deletedFloorID)
	}

	if err := service.ActivateBuildingFloor(context.Background(), 9); err != nil {
		t.Fatalf("ActivateBuildingFloor() error = %v", err)
	}
	if repo.activatedFloorID != 9 {
		t.Fatalf("activatedFloorID = %d, want 9", repo.activatedFloorID)
	}
}
