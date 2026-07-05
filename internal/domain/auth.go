package domain

const (
	UserRoleStaff          = "staff"
	UserRoleTemporaryStaff = "temporary_staff"
	UserRoleViewer         = "viewer"

	UserStatusActive   = "active"
	UserStatusExpired  = "expired"
	UserStatusDisabled = "disabled"

	AccessCodeStatusActive  = "active"
	AccessCodeStatusExpired = "expired"
	AccessCodeStatusRevoked = "revoked"
)

type User struct {
	ID          int64
	Username    string
	DisplayName string
	Affiliation string
	ContactNote string
	Role        string
	Status      string
	ExpiresAt   string
	LastLoginAt string
	CreatedAt   string
	UpdatedAt   string
}

type AccessCode struct {
	ID          int64
	UserID      int64
	CodeHash    string
	DisplayCode string
	Label       string
	Status      string
	IssuedAt    string
	ExpiresAt   string
	RevokedAt   string
	LastUsedAt  string
	Note        string
	CreatedAt   string
	UpdatedAt   string
}

type AccessCodePrincipal struct {
	User       User
	AccessCode AccessCode
}
