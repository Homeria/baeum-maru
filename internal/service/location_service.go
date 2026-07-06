package service

import (
	"context"
	"errors"
	"sort"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type LocationRepository interface {
	Create(context.Context, repository.CreateLocationParams) (domain.Location, error)
	Update(context.Context, repository.UpdateLocationParams) (domain.Location, error)
	Get(context.Context, int64) (domain.Location, error)
	List(context.Context, repository.ListLocationsParams) ([]domain.Location, error)
	ListRoles(context.Context, repository.ListLocationRolesParams) ([]domain.LocationRole, error)
	CreateRole(context.Context, string) (domain.LocationRole, error)
	UpdateRole(context.Context, int64, string) (domain.LocationRole, error)
	DeactivateRole(context.Context, int64) error
	ActivateRole(context.Context, int64) error
	DeleteRole(context.Context, int64) error
	ListBuildings(context.Context, repository.ListBuildingsParams) ([]domain.Building, error)
	CreateBuilding(context.Context, string) (domain.Building, error)
	UpdateBuilding(context.Context, int64, string) (domain.Building, error)
	DeactivateBuilding(context.Context, int64) error
	ActivateBuilding(context.Context, int64) error
	DeleteBuilding(context.Context, int64) error
	ListBuildingFloors(context.Context, repository.ListBuildingFloorsParams) ([]domain.BuildingFloor, error)
	CreateBuildingFloor(context.Context, int64, string) (domain.BuildingFloor, error)
	DeactivateBuildingFloor(context.Context, int64) error
	ActivateBuildingFloor(context.Context, int64) error
}

type LocationService struct {
	locations LocationRepository
}

type LocationInput struct {
	Name        string
	Building    string
	Floor       string
	Type        string
	Roles       []string
	IsClassroom bool
	IsActive    bool
	Note        string
}

type LocationListInput struct {
	Query           string
	Type            string
	ClassroomOnly   bool
	IncludeInactive bool
	Limit           int
}

type LocationRoleListInput struct {
	IncludeInactive bool
	Limit           int
}

type BuildingListInput struct {
	IncludeInactive bool
	Limit           int
}

type BuildingFloorListInput struct {
	BuildingID      int64
	IncludeInactive bool
	Limit           int
}

func NewLocationService(locations LocationRepository) *LocationService {
	return &LocationService{locations: locations}
}

func (s *LocationService) Create(ctx context.Context, input LocationInput) (domain.Location, error) {
	normalized, err := normalizeLocationInput(input)
	if err != nil {
		return domain.Location{}, err
	}
	normalized.IsActive = true
	return s.locations.Create(ctx, repository.CreateLocationParams(normalized))
}

func (s *LocationService) Update(ctx context.Context, id int64, input LocationInput) (domain.Location, error) {
	if id <= 0 {
		return domain.Location{}, errors.New("location id is required")
	}
	normalized, err := normalizeLocationInput(input)
	if err != nil {
		return domain.Location{}, err
	}
	params := repository.UpdateLocationParams{
		ID:          id,
		Name:        normalized.Name,
		Building:    normalized.Building,
		Floor:       normalized.Floor,
		Type:        normalized.Type,
		Roles:       normalized.Roles,
		IsClassroom: normalized.IsClassroom,
		IsActive:    normalized.IsActive,
		Note:        normalized.Note,
	}
	return s.locations.Update(ctx, params)
}

func (s *LocationService) Get(ctx context.Context, id int64) (domain.Location, error) {
	if id <= 0 {
		return domain.Location{}, errors.New("location id is required")
	}
	return s.locations.Get(ctx, id)
}

func (s *LocationService) List(ctx context.Context, input LocationListInput) ([]domain.Location, error) {
	locationType, err := normalizeLocationType(input.Type)
	if err != nil {
		return nil, err
	}
	return s.locations.List(ctx, repository.ListLocationsParams{
		Query:           input.Query,
		Type:            locationType,
		ClassroomOnly:   input.ClassroomOnly,
		IncludeInactive: input.IncludeInactive,
		Limit:           input.Limit,
	})
}

func (s *LocationService) ListClassrooms(ctx context.Context, limit int) ([]domain.Location, error) {
	return s.List(ctx, LocationListInput{ClassroomOnly: true, Limit: limit})
}

func (s *LocationService) ListRoles(ctx context.Context, input LocationRoleListInput) ([]domain.LocationRole, error) {
	return s.locations.ListRoles(ctx, repository.ListLocationRolesParams{
		IncludeInactive: input.IncludeInactive,
		Limit:           input.Limit,
	})
}

func (s *LocationService) CreateRole(ctx context.Context, name string) (domain.LocationRole, error) {
	name = strings.TrimSpace(name)
	if name == "" {
		return domain.LocationRole{}, errors.New("location role name is required")
	}
	if name == domain.LocationTypeOther {
		name = "common"
	}
	return s.locations.CreateRole(ctx, name)
}

func (s *LocationService) UpdateRole(ctx context.Context, id int64, name string) (domain.LocationRole, error) {
	if id <= 0 {
		return domain.LocationRole{}, errors.New("location role id is required")
	}
	name = strings.TrimSpace(name)
	if name == "" {
		return domain.LocationRole{}, errors.New("location role name is required")
	}
	if name == domain.LocationTypeOther {
		name = "common"
	}
	return s.locations.UpdateRole(ctx, id, name)
}

func (s *LocationService) DeactivateRole(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("location role id is required")
	}
	return s.locations.DeactivateRole(ctx, id)
}

func (s *LocationService) ActivateRole(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("location role id is required")
	}
	return s.locations.ActivateRole(ctx, id)
}

func (s *LocationService) DeleteRole(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("location role id is required")
	}
	return s.locations.DeleteRole(ctx, id)
}

func (s *LocationService) ListBuildings(ctx context.Context, input BuildingListInput) ([]domain.Building, error) {
	return s.locations.ListBuildings(ctx, repository.ListBuildingsParams{
		IncludeInactive: input.IncludeInactive,
		Limit:           input.Limit,
	})
}

func (s *LocationService) CreateBuilding(ctx context.Context, name string) (domain.Building, error) {
	name = strings.TrimSpace(name)
	if name == "" {
		return domain.Building{}, errors.New("building name is required")
	}
	return s.locations.CreateBuilding(ctx, name)
}

func (s *LocationService) UpdateBuilding(ctx context.Context, id int64, name string) (domain.Building, error) {
	if id <= 0 {
		return domain.Building{}, errors.New("building id is required")
	}
	name = strings.TrimSpace(name)
	if name == "" {
		return domain.Building{}, errors.New("building name is required")
	}
	return s.locations.UpdateBuilding(ctx, id, name)
}

func (s *LocationService) DeactivateBuilding(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("building id is required")
	}
	return s.locations.DeactivateBuilding(ctx, id)
}

func (s *LocationService) ActivateBuilding(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("building id is required")
	}
	return s.locations.ActivateBuilding(ctx, id)
}

func (s *LocationService) DeleteBuilding(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("building id is required")
	}
	return s.locations.DeleteBuilding(ctx, id)
}

func (s *LocationService) ListBuildingFloors(ctx context.Context, input BuildingFloorListInput) ([]domain.BuildingFloor, error) {
	return s.locations.ListBuildingFloors(ctx, repository.ListBuildingFloorsParams{
		BuildingID:      input.BuildingID,
		IncludeInactive: input.IncludeInactive,
		Limit:           input.Limit,
	})
}

func (s *LocationService) CreateBuildingFloor(ctx context.Context, buildingID int64, label string) (domain.BuildingFloor, error) {
	if buildingID <= 0 {
		return domain.BuildingFloor{}, errors.New("building id is required")
	}
	label = strings.TrimSpace(label)
	if label == "" {
		return domain.BuildingFloor{}, errors.New("building floor label is required")
	}
	return s.locations.CreateBuildingFloor(ctx, buildingID, label)
}

func (s *LocationService) DeleteBuildingFloor(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("building floor id is required")
	}
	return s.locations.DeactivateBuildingFloor(ctx, id)
}

func (s *LocationService) ActivateBuildingFloor(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("building floor id is required")
	}
	return s.locations.ActivateBuildingFloor(ctx, id)
}

func normalizeLocationInput(input LocationInput) (LocationInput, error) {
	input.Name = strings.TrimSpace(input.Name)
	if input.Name == "" {
		return LocationInput{}, errors.New("location name is required")
	}

	roles := normalizeLocationRoles(input.Roles, input.Type, input.IsClassroom)
	locationType := preferredLocationType(roles)
	if containsLocationRole(roles, domain.LocationTypeClassroom) {
		input.IsClassroom = true
	}

	input.Type = locationType
	input.Roles = roles
	input.Building = strings.TrimSpace(input.Building)
	input.Floor = strings.TrimSpace(input.Floor)
	input.Note = strings.TrimSpace(input.Note)
	return input, nil
}

func normalizeLocationType(value string) (string, error) {
	value = strings.TrimSpace(value)
	return value, nil
}

func normalizeLocationRoles(values []string, legacyType string, isClassroom bool) []string {
	roleSet := map[string]bool{}
	for _, value := range values {
		addNormalizedLocationRole(roleSet, value)
	}
	addNormalizedLocationRole(roleSet, legacyType)
	if isClassroom {
		addNormalizedLocationRole(roleSet, domain.LocationTypeClassroom)
	}
	if len(roleSet) == 0 {
		addNormalizedLocationRole(roleSet, domain.LocationTypeOther)
	}

	roles := make([]string, 0, len(roleSet))
	for role := range roleSet {
		roles = append(roles, role)
	}
	sort.Strings(roles)
	return roles
}

func addNormalizedLocationRole(roleSet map[string]bool, value string) {
	value = strings.TrimSpace(value)
	if value == "" {
		return
	}
	if value == domain.LocationTypeOther {
		value = "common"
	}
	roleSet[value] = true
}

func preferredLocationType(roles []string) string {
	for _, role := range []string{
		domain.LocationTypeClassroom,
		domain.LocationTypeOffice,
		domain.LocationTypeReception,
		domain.LocationTypeHall,
		domain.LocationTypeStorage,
	} {
		if containsLocationRole(roles, role) {
			return role
		}
	}
	return domain.LocationTypeOther
}

func containsLocationRole(roles []string, want string) bool {
	for _, role := range roles {
		if role == want {
			return true
		}
	}
	return false
}
