package repository

import (
	"context"
	"database/sql"
	"fmt"
	"strings"

	"github.com/Homeria/baeum-maru/internal/database"
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
	var id int64
	err := database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		buildingID, err := ensureBuilding(ctx, tx, strings.TrimSpace(params.Building))
		if err != nil {
			return err
		}
		result, err := tx.ExecContext(ctx, `
INSERT INTO locations (building_id, name, floor_label, description, is_active)
VALUES (?, ?, NULLIF(?, ''), NULLIF(?, ''), ?);
`, buildingID, strings.TrimSpace(params.Name), strings.TrimSpace(params.Floor), strings.TrimSpace(params.Note), boolInt(params.IsActive))
		if err != nil {
			return fmt.Errorf("insert location: %w", err)
		}
		id, err = result.LastInsertId()
		if err != nil {
			return fmt.Errorf("read location id: %w", err)
		}
		return syncLocationRoles(ctx, tx, id, createLocationRoleNames(params))
	})
	if err != nil {
		return domain.Location{}, err
	}
	return r.Get(ctx, id)
}

func (r *LocationRepository) Update(ctx context.Context, params UpdateLocationParams) (domain.Location, error) {
	err := database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		buildingID, err := ensureBuilding(ctx, tx, strings.TrimSpace(params.Building))
		if err != nil {
			return err
		}
		result, err := tx.ExecContext(ctx, `
UPDATE locations
SET building_id = ?,
    name = ?,
    floor_label = NULLIF(?, ''),
    description = NULLIF(?, ''),
    is_active = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;
`, buildingID, strings.TrimSpace(params.Name), strings.TrimSpace(params.Floor), strings.TrimSpace(params.Note), boolInt(params.IsActive), params.ID)
		if err != nil {
			return fmt.Errorf("update location %d: %w", params.ID, err)
		}
		affected, err := result.RowsAffected()
		if err != nil {
			return fmt.Errorf("read affected location rows: %w", err)
		}
		if affected == 0 {
			return sql.ErrNoRows
		}
		return syncLocationRoles(ctx, tx, params.ID, updateLocationRoleNames(params))
	})
	if err != nil {
		return domain.Location{}, err
	}
	return r.Get(ctx, params.ID)
}

func (r *LocationRepository) Get(ctx context.Context, id int64) (domain.Location, error) {
	row := r.db.QueryRowContext(ctx, locationSelectSQL()+`
WHERE l.id = ?
GROUP BY l.id;
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
	role := strings.TrimSpace(params.Type)
	if params.ClassroomOnly {
		role = domain.LocationTypeClassroom
	}
	like := "%" + query + "%"
	rows, err := r.db.QueryContext(ctx, locationSelectSQL()+`
WHERE (? = '' OR l.name LIKE ? OR COALESCE(b.name, '') LIKE ? OR COALESCE(l.floor_label, '') LIKE ? OR COALESCE(l.description, '') LIKE ?)
  AND (? = '' OR EXISTS (
    SELECT 1
    FROM location_role_assignments lra
    JOIN location_roles lr ON lr.id = lra.role_id
    WHERE lra.location_id = l.id AND lr.name = ?
  ))
  AND (? = 1 OR l.is_active = 1)
GROUP BY l.id
ORDER BY l.is_active DESC, l.name, l.id
LIMIT ?;
`, query, like, like, like, like, role, role, boolInt(params.IncludeInactive), params.Limit)
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

func ensureBuilding(ctx context.Context, tx *sql.Tx, name string) (sql.NullInt64, error) {
	if name == "" {
		return sql.NullInt64{}, nil
	}
	if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO buildings (name) VALUES (?);", name); err != nil {
		return sql.NullInt64{}, fmt.Errorf("ensure building: %w", err)
	}
	id, err := selectID(ctx, tx, "buildings", "name", name)
	if err != nil {
		return sql.NullInt64{}, err
	}
	return sql.NullInt64{Int64: id, Valid: true}, nil
}

func syncLocationRoles(ctx context.Context, tx *sql.Tx, locationID int64, roles []string) error {
	if _, err := tx.ExecContext(ctx, "DELETE FROM location_role_assignments WHERE location_id = ?;", locationID); err != nil {
		return fmt.Errorf("clear location roles: %w", err)
	}
	for _, role := range roles {
		if role == "" {
			continue
		}
		if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO location_roles (name) VALUES (?);", role); err != nil {
			return fmt.Errorf("ensure location role: %w", err)
		}
		roleID, err := selectID(ctx, tx, "location_roles", "name", role)
		if err != nil {
			return err
		}
		if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO location_role_assignments (location_id, role_id) VALUES (?, ?);", locationID, roleID); err != nil {
			return fmt.Errorf("assign location role: %w", err)
		}
	}
	return nil
}

func createLocationRoleNames(params CreateLocationParams) []string {
	roleSet := map[string]bool{}
	addLocationRole(roleSet, params.Type)
	if params.IsClassroom {
		addLocationRole(roleSet, domain.LocationTypeClassroom)
	}
	if len(roleSet) == 0 {
		addLocationRole(roleSet, domain.LocationTypeOther)
	}
	return roleSetNames(roleSet)
}

func updateLocationRoleNames(params UpdateLocationParams) []string {
	roleSet := map[string]bool{}
	addLocationRole(roleSet, params.Type)
	if params.IsClassroom {
		addLocationRole(roleSet, domain.LocationTypeClassroom)
	}
	if len(roleSet) == 0 {
		addLocationRole(roleSet, domain.LocationTypeOther)
	}
	return roleSetNames(roleSet)
}

func addLocationRole(roleSet map[string]bool, role string) {
	role = strings.TrimSpace(role)
	if role == "" {
		return
	}
	if role == domain.LocationTypeOther {
		role = "common"
	}
	roleSet[role] = true
}

func roleSetNames(roleSet map[string]bool) []string {
	roles := make([]string, 0, len(roleSet))
	for role := range roleSet {
		roles = append(roles, role)
	}
	return roles
}

func locationSelectSQL() string {
	return `
SELECT l.id,
       COALESCE(l.building_id, 0),
       l.name,
       COALESCE(b.name, ''),
       COALESCE(l.floor_label, ''),
       l.is_active,
       COALESCE(l.description, ''),
       l.created_at,
       l.updated_at,
       COALESCE(GROUP_CONCAT(lr.name, ','), '')
FROM locations l
LEFT JOIN buildings b ON b.id = l.building_id
LEFT JOIN location_role_assignments lra ON lra.location_id = l.id
LEFT JOIN location_roles lr ON lr.id = lra.role_id
`
}

type locationScanner interface {
	Scan(dest ...any) error
}

func scanLocation(scanner locationScanner) (domain.Location, error) {
	var location domain.Location
	var isActive int
	var roleCSV string
	if err := scanner.Scan(
		&location.ID,
		&location.BuildingID,
		&location.Name,
		&location.Building,
		&location.Floor,
		&isActive,
		&location.Note,
		&location.CreatedAt,
		&location.UpdatedAt,
		&roleCSV,
	); err != nil {
		return domain.Location{}, fmt.Errorf("scan location: %w", err)
	}
	location.FloorLabel = location.Floor
	location.IsActive = isActive == 1
	location.Roles = splitCSV(roleCSV)
	location.Type = domain.LocationTypeOther
	for _, role := range location.Roles {
		if role == domain.LocationTypeClassroom {
			location.IsClassroom = true
			location.Type = domain.LocationTypeClassroom
			break
		}
		if role != "" && role != "common" {
			location.Type = role
		}
	}
	return location, nil
}

func splitCSV(value string) []string {
	if strings.TrimSpace(value) == "" {
		return nil
	}
	parts := strings.Split(value, ",")
	roles := make([]string, 0, len(parts))
	for _, part := range parts {
		part = strings.TrimSpace(part)
		if part != "" {
			roles = append(roles, part)
		}
	}
	return roles
}

func boolInt(value bool) int {
	if value {
		return 1
	}
	return 0
}
