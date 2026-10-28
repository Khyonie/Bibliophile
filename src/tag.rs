use rusqlite::{Connection, Error, OptionalExtension};

pub struct Tag {
    pub name: String,
    pub color: String,
}

impl Tag {
    pub fn get_tags(database: &Connection) -> Result<Vec<Tag>, Error> {
        let mut statement = database.prepare("SELECT name, color FROM tags")?;

        let rows: Vec<_> = statement
            .query_map([], |row| {
                Ok(Tag {
                    name: row.get(0)?,
                    color: row.get(1)?,
                })
            })?
            .filter_map(|v| v.ok())
            .collect();

        Ok(rows)
    }

    pub fn new_tag(name: &str, color: &str, database: &Connection) -> Result<Tag, Error> {
        database.execute(r#"INSERT INTO tags (name, color) VALUES (?1, ?2)"#, [name, color])?;

        // TODO Add color validation

        Ok(Tag {
            name: String::from(name),
            color: String::from(color),
        })
    }

    pub fn get_tag_by_name(name: &str, database: &Connection) -> Result<Option<Tag>, Error> {
        let mut statement = database.prepare("SELECT name, color FROM tags WHERE name = ?1")?;
        statement
            .query_row([name], |row| {
                Ok(Tag {
                    name: row.get(0)?,
                    color: row.get(1)?,
                })
            })
            .optional()
    }
}
