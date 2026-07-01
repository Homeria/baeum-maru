package domain

type AuditLog struct {
	ID          int64
	ActorUserID int64
	Action      string
	EntityType  string
	EntityID    int64
	Summary     string
	CreatedAt   string
}
