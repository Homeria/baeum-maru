package domain

type BackupFile struct {
	FileName  string
	Path      string
	SizeBytes int64
	CreatedAt string
}

type BackupStatus struct {
	Latest      *BackupFile
	TotalCount  int
	TotalBytes  int64
	KeepDays    int
	RetentionOn bool
}

type BackupCleanup struct {
	DeletedCount int
	DeletedFiles []string
}

type RestorePlan struct {
	FileName string
	Path     string
}
