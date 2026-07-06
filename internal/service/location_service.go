package service

import (
	"context"
	"errors"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type LocationRepository interface {
	Create(context.Context, repository.CreateLocationParams) (domain.Location, error)
	Update(context.Context, repository.UpdateLocationParams) (domain.Location, error)
	Get(context.Context, int64) (domain.Location, error)
	List(context.Context, repository.ListLocationsParams) ([]domain.Location, error)
}

type LocationService struct {
	locations LocationRepository
}

type LocationInput struct {
	Name        string
	Building    string
	Floor       string
	Type        string
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

func normalizeLocationInput(input LocationInput) (LocationInput, error) {
	input.Name = strings.TrimSpace(input.Name)
	if input.Name == "" {
		return LocationInput{}, errors.New("location name is required")
	}

	locationType, err := normalizeLocationType(input.Type)
	if err != nil {
		return LocationInput{}, err
	}
	if locationType == "" {
		if input.IsClassroom {
			locationType = domain.LocationTypeClassroom
		} else {
			locationType = domain.LocationTypeOther
		}
	}
	if locationType == domain.LocationTypeClassroom {
		input.IsClassroom = true
	}

	input.Type = locationType
	input.Building = strings.TrimSpace(input.Building)
	input.Floor = strings.TrimSpace(input.Floor)
	input.Note = strings.TrimSpace(input.Note)
	return input, nil
}

func normalizeLocationType(value string) (string, error) {
	value = strings.TrimSpace(value)
	return value, nil
}
