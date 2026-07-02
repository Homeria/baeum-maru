package config

import (
	"crypto/rand"
	"encoding/base64"
)

func Default() Config {
	return Config{
		App: AppConfig{
			DisplayName: "배움마루",
			EnglishName: "Baeum-Maru",
			Mode:        "portable",
		},
		Server: ServerConfig{
			Host: "0.0.0.0",
			Port: 18080,
		},
		Database: DatabaseConfig{
			Path: "./data/center.db",
		},
		Backup: BackupConfig{
			Path:     "./backups",
			KeepDays: 30,
		},
		Export: ExportConfig{
			Path: "./exports",
		},
		Logging: LoggingConfig{
			Path:  "./logs/app.log",
			Level: "info",
		},
		UI: UIConfig{
			OpenBrowserOnStart: true,
		},
		Auth: AuthConfig{
			Disabled:             false,
			AdminPassword:        "admin",
			SessionSecret:        randomSessionSecret(),
			SessionMaxAgeMinutes: 720,
		},
	}
}

func randomSessionSecret() string {
	data := make([]byte, 32)
	if _, err := rand.Read(data); err != nil {
		return "change-this-session-secret"
	}
	return base64.RawURLEncoding.EncodeToString(data)
}
