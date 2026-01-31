use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct BackupManager {
    backup_dir: PathBuf,
}

#[allow(dead_code)]
impl BackupManager {
    pub fn new(backup_dir: PathBuf) -> Result<Self, std::io::Error> {
        fs::create_dir_all(&backup_dir)?;
        Ok(Self { backup_dir })
    }

    pub fn default() -> Result<Self, std::io::Error> {
        let backup_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dough")
            .join("backups");
        Self::new(backup_dir)
    }

    /// Create a backup of the database
    pub fn create_backup(&self, db_path: &Path) -> Result<PathBuf, std::io::Error> {
        if !db_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Database file not found",
            ));
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("dough_backup_{}.db", timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        fs::copy(db_path, &backup_path)?;

        // Also copy WAL and SHM files if they exist (for SQLite)
        let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
        let shm_path = PathBuf::from(format!("{}-shm", db_path.display()));

        if wal_path.exists() {
            let wal_backup = PathBuf::from(format!("{}-wal", backup_path.display()));
            fs::copy(wal_path, wal_backup).ok();
        }

        if shm_path.exists() {
            let shm_backup = PathBuf::from(format!("{}-shm", backup_path.display()));
            fs::copy(shm_path, shm_backup).ok();
        }

        Ok(backup_path)
    }

    /// List all backups
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>, std::io::Error> {
        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("db") {
                let metadata = entry.metadata()?;
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Skip WAL and SHM files
                if name.ends_with("-wal") || name.ends_with("-shm") {
                    continue;
                }

                backups.push(BackupInfo {
                    name,
                    path,
                    size: metadata.len(),
                    created: metadata.modified()?.into(),
                });
            }
        }

        backups.sort_by(|a, b| b.created.cmp(&a.created));
        Ok(backups)
    }

    /// Restore from backup
    pub fn restore_backup(&self, backup_path: &Path, db_path: &Path) -> Result<(), std::io::Error> {
        if !backup_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Backup file not found",
            ));
        }

        // Create a safety backup of current database before restoring
        if db_path.exists() {
            let safety_backup = self.backup_dir.join(format!(
                "pre_restore_safety_{}.db",
                Utc::now().format("%Y%m%d_%H%M%S")
            ));
            fs::copy(db_path, safety_backup)?;
        }

        // Restore the backup
        fs::copy(backup_path, db_path)?;

        // Also restore WAL and SHM files if they exist
        let wal_backup = PathBuf::from(format!("{}-wal", backup_path.display()));
        let shm_backup = PathBuf::from(format!("{}-shm", backup_path.display()));

        if wal_backup.exists() {
            let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
            fs::copy(wal_backup, wal_path).ok();
        }

        if shm_backup.exists() {
            let shm_path = PathBuf::from(format!("{}-shm", db_path.display()));
            fs::copy(shm_backup, shm_path).ok();
        }

        Ok(())
    }

    /// Delete old backups, keeping the most recent N backups
    pub fn cleanup_old_backups(&self, keep_count: usize) -> Result<usize, std::io::Error> {
        let mut backups = self.list_backups()?;

        if backups.len() <= keep_count {
            return Ok(0);
        }

        let mut deleted_count = 0;
        backups.drain(keep_count..).for_each(|backup| {
            if fs::remove_file(&backup.path).is_ok() {
                deleted_count += 1;

                // Also remove associated WAL and SHM files
                let wal_path = PathBuf::from(format!("{}-wal", backup.path.display()));
                let shm_path = PathBuf::from(format!("{}-shm", backup.path.display()));
                fs::remove_file(wal_path).ok();
                fs::remove_file(shm_path).ok();
            }
        });

        Ok(deleted_count)
    }

    /// Export backup to a different location
    pub fn export_backup(
        &self,
        backup_name: &str,
        destination: &Path,
    ) -> Result<(), std::io::Error> {
        let backup_path = self.backup_dir.join(backup_name);

        if !backup_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Backup not found",
            ));
        }

        fs::copy(backup_path, destination)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BackupInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub created: chrono::DateTime<Utc>,
}

#[allow(dead_code)]
impl BackupInfo {
    pub fn size_mb(&self) -> f64 {
        self.size as f64 / (1024.0 * 1024.0)
    }
}
