use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use ignore::WalkBuilder;

const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone)]
pub struct IndexedFile {
    pub path: String,
    pub is_dir: bool,
    pub mtime: i64,
}

pub struct FileIndex {
    conn: Connection,
    root: PathBuf,
}

impl FileIndex {
    /// Open or create an index for a directory.
    /// Index is stored in .moss/index.sqlite
    pub fn open(root: &Path) -> rusqlite::Result<Self> {
        let moss_dir = root.join(".moss");
        std::fs::create_dir_all(&moss_dir).ok();

        let db_path = moss_dir.join("index.sqlite");
        let conn = Connection::open(&db_path)?;

        // Initialize schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                is_dir INTEGER NOT NULL,
                mtime INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_files_name ON files(path);
            "
        )?;

        // Check schema version
        let version: i64 = conn
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if version != SCHEMA_VERSION {
            // Reset on schema change
            conn.execute("DELETE FROM files", [])?;
            conn.execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
                params![SCHEMA_VERSION.to_string()],
            )?;
        }

        Ok(Self {
            conn,
            root: root.to_path_buf(),
        })
    }

    /// Check if index needs refresh based on .moss directory mtime
    pub fn needs_refresh(&self) -> bool {
        // Check if index is empty
        let file_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap_or(0);
        if file_count == 0 {
            return true;
        }

        let last_indexed: i64 = self
            .conn
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM meta WHERE key = 'last_indexed'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // If never indexed, refresh
        if last_indexed == 0 {
            return true;
        }

        // Check if any common directories have changed
        // This is a heuristic - check src/, lib/, etc.
        // Note: "." changes too often, skip it
        for dir in &["src", "lib", "crates"] {
            let path = self.root.join(dir);
            if path.exists() {
                if let Ok(meta) = path.metadata() {
                    if let Ok(mtime) = meta.modified() {
                        let mtime_secs = mtime
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        if mtime_secs > last_indexed {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Refresh the index by walking the filesystem
    pub fn refresh(&mut self) -> rusqlite::Result<usize> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        // Start transaction for batch insert
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM files", [])?;

        let mut count = 0;
        for entry in walker.flatten() {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(&self.root) {
                let rel_str = rel.to_string_lossy().to_string();
                if rel_str.is_empty() {
                    continue;
                }

                let is_dir = path.is_dir();
                let mtime = path
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                tx.execute(
                    "INSERT INTO files (path, is_dir, mtime) VALUES (?1, ?2, ?3)",
                    params![rel_str, is_dir as i64, mtime],
                )?;
                count += 1;
            }
        }

        // Update last indexed time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        tx.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('last_indexed', ?1)",
            params![now.to_string()],
        )?;

        tx.commit()?;
        Ok(count)
    }

    /// Get all files from the index
    pub fn all_files(&self) -> rusqlite::Result<Vec<IndexedFile>> {
        let mut stmt = self.conn.prepare("SELECT path, is_dir, mtime FROM files")?;
        let files = stmt
            .query_map([], |row| {
                Ok(IndexedFile {
                    path: row.get(0)?,
                    is_dir: row.get::<_, i64>(1)? != 0,
                    mtime: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(files)
    }

    /// Search files by exact name match
    pub fn find_by_name(&self, name: &str) -> rusqlite::Result<Vec<IndexedFile>> {
        let pattern = format!("%/{}", name);
        let mut stmt = self.conn.prepare(
            "SELECT path, is_dir, mtime FROM files WHERE path LIKE ?1 OR path = ?2"
        )?;
        let files = stmt
            .query_map(params![pattern, name], |row| {
                Ok(IndexedFile {
                    path: row.get(0)?,
                    is_dir: row.get::<_, i64>(1)? != 0,
                    mtime: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(files)
    }

    /// Search files by stem (filename without extension)
    pub fn find_by_stem(&self, stem: &str) -> rusqlite::Result<Vec<IndexedFile>> {
        let pattern = format!("%/{}%", stem);
        let mut stmt = self.conn.prepare(
            "SELECT path, is_dir, mtime FROM files WHERE path LIKE ?1"
        )?;
        let files = stmt
            .query_map(params![pattern], |row| {
                Ok(IndexedFile {
                    path: row.get(0)?,
                    is_dir: row.get::<_, i64>(1)? != 0,
                    mtime: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(files)
    }

    /// Count indexed files
    pub fn count(&self) -> rusqlite::Result<usize> {
        self.conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_index_creation() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/moss")).unwrap();
        fs::write(dir.path().join("src/moss/cli.py"), "").unwrap();
        fs::write(dir.path().join("src/moss/dwim.py"), "").unwrap();

        let mut index = FileIndex::open(dir.path()).unwrap();
        assert!(index.needs_refresh());

        let count = index.refresh().unwrap();
        assert!(count >= 2);

        // Should find files by name
        let matches = index.find_by_name("cli.py").unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].path.ends_with("cli.py"));
    }

    #[test]
    fn test_find_by_stem() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/test.py"), "").unwrap();
        fs::write(dir.path().join("src/test.rs"), "").unwrap();

        let mut index = FileIndex::open(dir.path()).unwrap();
        index.refresh().unwrap();

        let matches = index.find_by_stem("test").unwrap();
        assert_eq!(matches.len(), 2);
    }
}
