package repository

import (
	"context"
	"database/sql"
	"fmt"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
)

type LocationRepository struct {
	db *sql.DB
}

type CreateLocationParams struct {
	Name        string
	Building    string
	Floor       string
	Type        string
	IsClassroom bool
	IsActive    bool
	Note        string
}

type UpdateLocationParams struct {
	ID          int64
	Name        string
	Building    string
	Floor       string
	Type        string
	IsClassroom bool
	IsActive    bool
	Note        string
}

type ListLocationsParams struct {
	Query           string
	Type            string
	ClassroomOnly   bool
	IncludeInactive bool
	Limit           int
}

func NewLocationRepository(db *sql.DB) *LocationRepository {
	return &LocationRepository{db: db}
}

func (r *LocationRepository) Create(ctx context.Context, params CreateLocationParams) (domain.Location, error) {
	result, err := r.db.ExecContext(ctx, `
INSERT INTO locations (name, building, floor, type, is_classroom, is_active, note)
VALUES (?, NULLIF(?, ''), NULLIF(?, ''), ?, ?, ?, NULLIF(?, ''));
`, strings.TrimSpace(params.Name), strings.TrimSpace(params.Building), strings.TrimSpace(params.Floor), strings.TrimSpace(params.Type), boolInt(params.IsClassroom), boolInt(params.IsActive), strings.TrimSpace(params.Note))
	if err != nil {
		return domain.Location{}, fmt.Errorf("insert location: %w", err)
	}
	id, err := result.LastInsertId()
	if err != nil {
		return domain.Location{}, fmt.Errorf("read location id: %w", err)
	}
	return r.Get(ctx, id)
}

func (r *LocationRepository) Update(ctx context.Context, params UpdateLocationParams) (domain.Location, error) {
	result, err := r.db.ExecContext(ctx, `
UPDATE locations
SET name = ?,
    building = NULLIF(?, ''),
    floor = NULLIF(?, ''),
    type = ?,
    is_classroom = ?,
    is_active = ?,
    note = NULLIF(?, ''),
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;
`, strings.TrimSpace(params.Name), strings.TrimSpace(params.Building), strings.TrimSpace(params.Floor), strings.TrimSpace(params.Type), boolInt(params.IsClassroom), boolInt(params.IsActive), strings.TrimSpace(params.Note), params.ID)
	if err != nil {
		return domain.Location{}, fmt.Errorf("update location %d: %w", params.ID, err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return domain.Location{}, fmt.Errorf("read affected location rows: %w", err)
	}
	if affected == 0 {
		return domain.Location{}, sql.ErrNoRows
	}
	return r.Get(ctx, params.ID)
}

func (r *LocationRepository) Get(ctx context.Context, id int64) (domain.Location, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT id, name, COALESCE(building, ''), COALESCE(floor, ''), type, is_classroom, is_active,
       COALESCE(note, ''), created_at, updated_at
FROM locations
WHERE id = ?;
`, id)
	return scanLocation(row)
}

func (r *LocationRepository) List(ctx context.Context, params ListLocationsParams) ([]domain.Location, error) {
	if params.Limit <= 0 {
		params.Limit = 100
	}
	if params.Limit > 10000 {
		params.Limit = 10000
	}

	query := strings.TrimSpace(params.Query)
	locationType := strings.TrimSpace(params.Type)
	like := "%" + query + "%"
	rows, err := r.db.QueryContext(ctx, `
SELECT id, name, COALESCE(building, ''), COALESCE(floor, ''), type, is_classroom, is_active,
       COALESCE(note, ''), created_at, updated_at
FROM locations
WHERE (? = '' OR name LIKE ? OR COALESCE(building, '') LIKE ? OR COALESCE(floor, '') LIKE ? OR COALESCE(note, '') LIKE ?)
  AND (? = '' OR type = ?)
  AND (? = 0 OR is_classroom = 1)
  AND (? = 1 OR is_active = 1)
ORDER BY is_active DESC, is_classroom DESC, name, id
LIMIT ?;
`, query, like, like, like, like, locationType, locationType, boolInt(params.ClassroomOnly), boolInt(params.IncludeInactive), params.Limit)
	if err != nil {
		return nil, fmt.Errorf("list locations: %w", err)
	}
	defer rows.Close()

	var locations []domain.Location
	for rows.Next() {
		location, err := scanLocation(rows)
		if err != nil {
			return nil, err
		}
		locations = append(locations, location)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate locations: %w", err)
	}
	return locations, nil
}

type locationScanner interface {
	Scan(dest ...any) error
}

func scanLocation(scanner locationScanner) (domain.Location, error) {
	var location domain.Location
	var isClassroom, isActive int
	if err := scanner.Scan(
		&location.ID,
		&location.Name,
		&location.Building,
		&location.Floor,
		&location.Type,
		&isClassroom,
		&isActive,
		&location.Note,
		&location.CreatedAt,
		&location.UpdatedAt,
	); err != nil {
		return domain.Location{}, fmt.Errorf("scan location: %w", err)
	}
	location.IsClassroom = isClassroom == 1
	location.IsActive = isActive == 1
	return location, nil
}

func boolInt(value bool) int {
	if value {
		return 1
	}
	return 0
}
