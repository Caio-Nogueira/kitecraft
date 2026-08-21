use wasm_bindgen::JsCast;
use worker::SqlStorage;
use world::ChunkBacking;

pub struct SqliteBacking {
    sql: SqlStorage,
}

impl SqliteBacking {
    pub fn new(sql: SqlStorage) -> Self {
        Self { sql }
    }

    pub fn init_schema(&self) -> worker::Result<()> {
        self.sql
            .exec("CREATE TABLE IF NOT EXISTS kv (k TEXT PRIMARY KEY, v BLOB)", None)?;
        Ok(())
    }

    pub fn get_blob(&self, key: &str) -> Option<Vec<u8>> {
        let mut cursor = self
            .sql
            .exec("SELECT v FROM kv WHERE k = ?1", Some(vec![key.into()]))
            .ok()?;
        let row = std::iter::Iterator::next(&mut cursor)?.ok()?;
        let v = if row.is_instance_of::<js_sys::Array>() {
            row.dyn_into::<js_sys::Array>().ok()?.get(0)
        } else {
            js_sys::Reflect::get(&row, &"v".into()).ok()?
        };
        if v.is_undefined() || v.is_null() {
            return None;
        }
        v.dyn_into::<js_sys::Uint8Array>().ok().map(|u| u.to_vec())
    }

    pub fn set_blob(&self, key: &str, data: &[u8]) {
        let _ = self.sql.exec(
            "INSERT OR REPLACE INTO kv (k, v) VALUES (?1, ?2)",
            Some(vec![key.into(), data.to_vec().into()]),
        );
    }

    pub fn meta_pair(&self, key: &str) -> Option<(i64, i64)> {
        let blob = self.get_blob(key)?;
        if blob.len() < 16 {
            return None;
        }
        Some((
            i64::from_le_bytes(blob[0..8].try_into().ok()?),
            i64::from_le_bytes(blob[8..16].try_into().ok()?),
        ))
    }

    pub fn set_meta_pair(&self, key: &str, a: i64, b: i64) {
        let mut blob = Vec::with_capacity(16);
        blob.extend_from_slice(&a.to_le_bytes());
        blob.extend_from_slice(&b.to_le_bytes());
        self.set_blob(key, &blob);
    }
}

impl ChunkBacking for SqliteBacking {
    fn read(&self, x: i32, z: i32) -> Option<Vec<u8>> {
        self.get_blob(&format!("chunk:{x}:{z}"))
    }

    fn write(&mut self, x: i32, z: i32, data: &[u8]) {
        self.set_blob(&format!("chunk:{x}:{z}"), data);
    }
}
