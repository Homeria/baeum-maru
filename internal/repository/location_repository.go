package repository

import (
	"context"
	"database/sql"
	"fmt"
	"sort"
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
	Roles       []string
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
	Roles       []string
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

type ListLocationRolesParams struct {
	IncludeInactive bool
	Limit           int
}

type ListBuildingsParams struct {
	IncludeInactive bool
	Limit           int
}

type ListBuildingFloorsParams struct {
	BuildingID      int64
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

func (r *LocationRepository) ListRoles(ctx context.Context, params ListLocationRolesParams) ([]domain.LocationRole, error) {
	if params.Limit <= 0 {
		params.Limit = 100
	}
	if params.Limit > 10000 {
		params.Limit = 10000
	}

	rows, err := r.db.QueryContext(ctx, `
SELECT id, name, is_active, sort_order
FROM location_roles
WHERE (? = 1 OR is_active = 1)
ORDER BY sort_order, name, id
LIMIT ?;
`, boolInt(params.IncludeInactive), params.Limit)
	if err != nil {
		return nil, fmt.Errorf("list location roles: %w", err)
	}
	defer rows.Close()

	var roles []domain.LocationRole
	for rows.Next() {
		var role domain.LocationRole
		var isActive int
		if err := rows.Scan(&role.ID, &role.Name, &isActive, &role.SortOrder); err != nil {
			return nil, fmt.Errorf("scan location role: %w", err)
		}
		role.IsActive = isActive == 1
		roles = append(roles, role)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate location roles: %w", err)
	}
	return roles, nil
}

func (r *LocationRepository) ListBuildings(ctx context.Context, params ListBuildingsParams) ([]domain.Building, error) {
	if params.Limit <= 0 {
		params.Limit = 100
	}
	if params.Limit > 10000 {
		params.Limit = 10000
	}

	rows, err := r.db.QueryContext(ctx, `
SELECT id, name, is_active
FROM buildings
WHERE (? = 1 OR is_active = 1)
ORDER BY is_active DESC, name, id
LIMIT ?;
`, boolInt(params.IncludeInactive), params.Limit)
	if err != nil {
		return nil, fmt.Errorf("list buildings: %w", err)
	}
	defer rows.Close()

	var buildings []domain.Building
	for rows.Next() {
		var building domain.Building
		var isActive int
		if err := rows.Scan(&building.ID, &building.Name, &isActive); err != nil {
			return nil, fmt.Errorf("scan building: %w", err)
		}
		building.IsActive = isActive == 1
		buildings = append(buildings, building)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate buildings: %w", err)
	}
	return buildings, nil
}

func (r *LocationRepository) ListBuildingFloors(ctx context.Context, params ListBuildingFloorsParams) ([]domain.BuildingFloor, error) {
	if params.Limit <= 0 {
		params.Limit = 100
	}
	if params.Limit > 10000 {
		params.Limit = 10000
	}

	rows, err := r.db.QueryContext(ctx, `
SELECT bf.id, bf.building_id, b.name, bf.label, bf.is_active, bf.sort_order
FROM building_floors bf
JOIN buildings b ON b.id = bf.building_id
WHERE (? = 0 OR bf.building_id = ?)
  AND (? = 1 OR bf.is_active = 1)
ORDER BY b.name, bf.sort_order, bf.label, bf.id
LIMIT ?;
`, params.BuildingID, params.BuildingID, boolInt(params.IncludeInactive), params.Limit)
	if err != nil {
		return nil, fmt.Errorf("list building floors: %w", err)
	}
	defer rows.Close()

	var floors []domain.BuildingFloor
	for rows.Next() {
		var floor domain.BuildingFloor
		var isActive int
		if err := rows.Scan(&floor.ID, &floor.BuildingID, &floor.Building, &floor.Label, &isActive, &floor.SortOrder); err != nil {
			return nil, fmt.Errorf("scan building floor: %w", err)
		}
		floor.IsActive = isActive == 1
		floors = append(floors, floor)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate building floors: %w", err)
	}
	return floors, nil
}

func (r *LocationRepository) CreateBuilding(ctx context.Context, name string) (domain.Building, error) {
	name = strings.TrimSpace(name)
	if _, err := r.db.ExecContext(ctx, `
INSERT INTO buildings (name, is_active)
VALUES (?, 1)
ON CONFLICT(name) DO UPDATE SET is_active = 1;
`, name); err != nil {
		return domain.Building{}, fmt.Errorf("create building: %w", err)
	}
	return r.getBuildingByName(ctx, name)
}

func (r *LocationRepository) UpdateBuilding(ctx context.Context, id int64, name string) (domain.Building, error) {
	name = strings.TrimSpace(name)
	result, err := r.db.ExecContext(ctx, `
UPDATE buildings
SET name = ?
WHERE id = ?;
`, name, id)
	if err != nil {
		return domain.Building{}, fmt.Errorf("update building %d: %w", id, err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return domain.Building{}, fmt.Errorf("read affected building rows: %w", err)
	}
	if affected == 0 {
		return domain.Building{}, sql.ErrNoRows
	}
	return r.getBuildingByName(ctx, name)
}

func (r *LocationRepository) DeleteBuilding(ctx context.Context, id int64) error {
	result, err := r.db.ExecContext(ctx, "DELETE FROM buildings WHERE id = ?;", id)
	if err != nil {
		return fmt.Errorf("delete building %d: %w", id, err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("read affected building rows: %w", err)
	}
	if affected == 0 {
		return sql.ErrNoRows
	}
	return nil
}

func (r *LocationRepository) CreateBuildingFloor(ctx context.Context, buildingID int64, label string) (domain.BuildingFloor, error) {
	label = strings.TrimSpace(label)
	if _, err := r.db.ExecContext(ctx, `
INSERT INTO building_floors (building_id, label, is_active)
VALUES (?, ?, 1)
ON CONFLICT(building_id, label) DO UPDATE SET is_active = 1;
`, buildingID, label); err != nil {
		return domain.BuildingFloor{}, fmt.Errorf("create building floor: %w", err)
	}
	return r.getBuildingFloorByLabel(ctx, buildingID, label)
}

func (r *LocationRepository) DeactivateBuilding(ctx context.Context, id int64) error {
	return r.setBuildingActive(ctx, id, false)
}

func (r *LocationRepository) ActivateBuilding(ctx context.Context, id int64) error {
	return r.setBuildingActive(ctx, id, true)
}

func (r *LocationRepository) setBuildingActive(ctx context.Context, id int64, active bool) error {
	result, err := r.db.ExecContext(ctx, `
UPDATE buildings
SET is_active = ?
WHERE id = ?;
`, boolInt(active), id)
	if err != nil {
		return fmt.Errorf("set building active %d: %w", id, err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("read affected building rows: %w", err)
	}
	if affected == 0 {
		return sql.ErrNoRows
	}
	return nil
}

func (r *LocationRepository) DeactivateBuildingFloor(ctx context.Context, id int64) error {
	return r.setBuildingFloorActive(ctx, id, false)
}

func (r *LocationRepository) ActivateBuildingFloor(ctx context.Context, id int64) error {
	return r.setBuildingFloorActive(ctx, id, true)
}

func (r *LocationRepository) setBuildingFloorActive(ctx context.Context, id int64, active bool) error {
	result, err := r.db.ExecContext(ctx, `
UPDATE building_floors
SET is_active = ?
WHERE id = ?;
`, boolInt(active), id)
	if err != nil {
		return fmt.Errorf("set building floor active %d: %w", id, err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("read affected building floor rows: %w", err)
	}
	if affected == 0 {
		return sql.ErrNoRows
	}
	return nil
}

func (r *LocationRepository) getBuildingByName(ctx context.Context, name string) (domain.Building, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT id, name, is_active
FROM buildings
WHERE name = ?;
`, name)
	var building domain.Building
	var isActive int
	if err := row.Scan(&building.ID, &building.Name, &isActive); err != nil {
		return domain.Building{}, fmt.Errorf("read building: %w", err)
	}
	building.IsActive = isActive == 1
	return building, nil
}

func (r *LocationRepository) getBuildingFloorByLabel(ctx context.Context, buildingID int64, label string) (domain.BuildingFloor, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT bf.id, bf.building_id, b.name, bf.label, bf.is_active, bf.sort_order
FROM building_floors bf
JOIN buildings b ON b.id = bf.building_id
WHERE bf.building_id = ? AND bf.label = ?;
`, buildingID, label)
	var floor domain.BuildingFloor
	var isActive int
	if err := row.Scan(&floor.ID, &floor.BuildingID, &floor.Building, &floor.Label, &isActive, &floor.SortOrder); err != nil {
		return domain.BuildingFloor{}, fmt.Errorf("read building floor: %w", err)
	}
	floor.IsActive = isActive == 1
	return floor, nil
}

func (r *LocationRepository) CreateRole(ctx context.Context, name string) (domain.LocationRole, error) {
	name = strings.TrimSpace(name)
	if _, err := r.db.ExecContext(ctx, `
INSERT INTO location_roles (name, is_active)
VALUES (?, 1)
ON CONFLICT(name) DO UPDATE SET is_active = 1;
`, name); err != nil {
		return domain.LocationRole{}, fmt.Errorf("create location role: %w", err)
	}
	return r.getRoleByName(ctx, name)
}

func (r *LocationRepository) UpdateRole(ctx context.Context, id int64, name string) (domain.LocationRole, error) {
	name = strings.TrimSpace(name)
	result, err := r.db.ExecContext(ctx, `
UPDATE location_roles
SET name = ?
WHERE id = ?;
`, name, id)
	if err != nil {
		return domain.LocationRole{}, fmt.Errorf("update location role %d: %w", id, err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return domain.LocationRole{}, fmt.Errorf("read affected location role rows: %w", err)
	}
	if affected == 0 {
		return domain.LocationRole{}, sql.ErrNoRows
	}
	return r.getRoleByName(ctx, name)
}

func (r *LocationRepository) DeactivateRole(ctx context.Context, id int64) error {
	return r.setRoleActive(ctx, id, false)
}

func (r *LocationRepository) ActivateRole(ctx context.Context, id int64) error {
	return r.setRoleActive(ctx, id, true)
}

func (r *LocationRepository) DeleteRole(ctx context.Context, id int64) error {
	return database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		if _, err := tx.ExecContext(ctx, "DELETE FROM location_role_assignments WHERE role_id = ?;", id); err != nil {
			return fmt.Errorf("delete location role assignments %d: %w", id, err)
		}
		result, err := tx.ExecContext(ctx, "DELETE FROM location_roles WHERE id = ?;", id)
		if err != nil {
			return fmt.Errorf("delete location role %d: %w", id, err)
		}
		affected, err := result.RowsAffected()
		if err != nil {
			return fmt.Errorf("read affected location role rows: %w", err)
		}
		if affected == 0 {
			return sql.ErrNoRows
		}
		return nil
	})
}

func (r *LocationRepository) setRoleActive(ctx context.Context, id int64, active bool) error {
	result, err := r.db.ExecContext(ctx, `
UPDATE location_roles
SET is_active = ?
WHERE id = ?;
`, boolInt(active), id)
	if err != nil {
		return fmt.Errorf("set location role active %d: %w", id, err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("read affected location role rows: %w", err)
	}
	if affected == 0 {
		return sql.ErrNoRows
	}
	return nil
}

func (r *LocationRepository) getRoleByName(ctx context.Context, name string) (domain.LocationRole, error) {
	row := r.db.QueryRowContext(ctx, `
SELECT id, name, is_active, sort_order
FROM location_roles
WHERE name = ?;
`, name)
	var role domain.LocationRole
	var isActive int
	if err := row.Scan(&role.ID, &role.Name, &isActive, &role.SortOrder); err != nil {
		return domain.LocationRole{}, fmt.Errorf("read location role: %w", err)
	}
	role.IsActive = isActive == 1
	return role, nil
}

func ensureBuilding(ctx context.Context, tx *sql.Tx, name string) (sql.NullInt64, error) {
	if name == "" {
		return sql.NullInt64{}, nil
	}
	if _, err := tx.ExecContext(ctx, `
INSERT INTO buildings (name, is_active)
VALUES (?, 1)
ON CONFLICT(name) DO UPDATE SET is_active = 1;
`, name); err != nil {
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
	for _, role := range params.Roles {
		addLocationRole(roleSet, role)
	}
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
	for _, role := range params.Roles {
		addLocationRole(roleSet, role)
	}
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
	sort.Strings(roles)
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
