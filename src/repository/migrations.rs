pub const COMMANDS: [&str; 2] = [
    r#"
    CREATE TABLE IF NOT EXISTS logs (
      id INTEGER PRIMARY KEY NOT NULL,
      timestamp INTEGER NOT NULL,
      data BLOB NOT NULL
    );"#,
    r#"CREATE INDEX timestamp_index ON logs (timestamp);"#,
];
