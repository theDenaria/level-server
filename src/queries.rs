pub fn get_object_by_id_sql(version: String) -> String {
    format!("SELECT * FROM objects_v{} WHERE id = $1", version.as_str())
}

pub fn get_objects_sql(version: String) -> String {
    format!("SELECT * FROM objects_v{}", version.as_str())
}

pub fn delete_all_sql(version: String) -> String {
    format!("DELETE FROM objects_v{}", version.as_str())
}

pub fn get_first_id_sql(version: String) -> String {
    format!("SELECT MIN(id) FROM objects_v{}", version.as_str())
}

pub fn get_row_count_sql(version: String) -> String {
    format!(
        "SELECT COUNT(*) AS row_count FROM objects_v{}",
        version.as_str()
    )
}

pub fn set_object_sql(version: String) -> String {
    format!("INSERT INTO objects_v{} (object_type, position, rotation, scale, collider) VALUES ($1, $2, $3, $4, $5)", version.as_str())
}

pub fn create_table_sql(version: String) -> String {
    format!(
        r#"CREATE TABLE IF NOT EXISTS objects_v{} (
        id SERIAL PRIMARY KEY,
        object_type VARCHAR(255) NOT NULL,
        position VARCHAR(255) NOT NULL,
        scale VARCHAR(255) NOT NULL,
        rotation VARCHAR(255) NOT NULL,
        collider TEXT NOT NULL
        )"#,
        version.as_str()
    )
}
