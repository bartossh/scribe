pub const SQL_COMMANDS: [&str; 5] = [
    r#"
    CREATE TABLE IF NOT EXISTS logs (
      id INTEGER PRIMARY KEY NOT NULL,
      timestamp INTEGER NOT NULL,
      data BLOB NOT NULL
    );"#,
    r#"CREATE INDEX timestamp_index ON logs (timestamp);"#,
    r#"
    CREATE TABLE IF NOT EXISTS serializer (
      id INTEGER PRIMARY KEY NOT NULL,
      word TEXT NOT NULL UNIQUE,
      num INTEGER NOT NULL UNIQUE
    );"#,
    r#"CREATE INDEX word_index ON serializer (word);"#,
    r#"CREATE INDEX num_index ON serializer (num);"#,
];
