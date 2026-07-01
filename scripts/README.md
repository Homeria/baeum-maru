# Scripts

## Windows portable package

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\package_windows.ps1 -Version 0.1.0
```

Output:

```text
dist/
└─ BaeumMaru_Portable_0.1.0/
   ├─ baeum-maru.exe
   ├─ config.json
   ├─ README_FIRST_RUN.txt
   ├─ data/
   ├─ backups/
   ├─ exports/
   ├─ imports/
   └─ logs/
```

The script also creates `BaeumMaru_Portable_<version>.zip` unless `-SkipArchive` is passed.

