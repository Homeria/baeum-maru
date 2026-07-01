package config

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
	}
}
