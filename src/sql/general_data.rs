use rusqlite::{params, Connection, Result, Error};
use std::cell::RefCell;
use std::string::ToString;


thread_local!
{
    pub static CONNECTION: RefCell<Connection> = {
        let conn = Connection::open(
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("general-data.db")
        ).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Plugins (
                id INTEGER PRIMARY KEY,
                dev_name TEXT NOT NULL UNIQUE,
                will_be_used BOOLEAN NOT NULL,
                dependencies TEXT NOT NULL
            );",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Projects (
                id INTEGER PRIMARY KEY,
                dev_name TEXT NOT NULL,
                dir TEXT NOT NULL,
                UNIQUE (dev_name, dir)
            );",
            [],
        ).unwrap();
        RefCell::new(conn)
    };
}


#[derive(Debug, Clone)]
pub struct Plugin
{
    pub id: i32,
    pub dev_name: String,
    pub will_be_used: bool,
    pub dependencies: Vec<String>,
}
impl Plugin
{
    pub fn select_all() -> Result<Box<Vec<Result<Self, Error>>>, Error>
    {
        return CONNECTION.with(
            |data| Ok(
                Box::new(
                    (*data).borrow().prepare(
                        "SELECT * FROM Plugins",
                    )?.query_map(
                        [],
                        |row| Ok(
                            Self {
                                id: row.get::<_, i32>(0)?,
                                dev_name: row.get::<_, String>(1)?,
                                will_be_used: row.get::<_, bool>(2)?,
                                dependencies: row.get::<_, String>(3)?
                                    .split_whitespace().map(|s| s.to_string()).collect(),
                            },
                        ),
                    )?.collect(),
                ),
            ),
        );
    }
    pub fn get(dev_name: impl ToString) -> Result<Self, Error>
    {
        return CONNECTION.with(
            |data| (*data).borrow().query_row(
                "SELECT *
                FROM Plugins
                WHERE dev_name = ?1;",
                params![dev_name.to_string()],
                |row| Ok(
                    Self {
                        id: row.get::<_, i32>(0)?,
                        dev_name: row.get::<_, String>(1)?,
                        will_be_used: row.get::<_, bool>(2)?,
                        dependencies: row.get::<_, String>(3)?
                            .split_whitespace().map(|s| s.to_string()).collect(),
                    },
                ),
            ),
        );
    }
    pub fn set_will_be_used(&mut self, value: bool) -> Result<(), Error>
    {
        CONNECTION.with(
            |data| (*data).borrow().execute(
                "UPDATE Plugins SET will_be_used = ?1 WHERE id = ?2;",
                params![value, self.id],
            ),
        )?;
        self.will_be_used = value;
        return Ok(());
    }
    pub fn set_will_be_used_from_dev_name(
        dev_name: impl ToString,
        value: bool,
    ) -> Result<(), Error>
    {
        CONNECTION.with(
            |data| (*data).borrow().execute(
                "UPDATE Plugins SET will_be_used = ?1 WHERE dev_name = ?2;",
                params![value, dev_name.to_string()],
            ),
        )?;
        return Ok(());
    }
}


#[derive(Debug, Clone)]
pub struct Project
{
    pub id: i32,
    pub dev_name: String,
    pub dir: String,
}
impl Project
{
    pub fn select_all() -> Result<Box<Vec<Result<Self, Error>>>, Error>
    {
        return CONNECTION.with(
            |data| Ok(
                Box::new(
                    (*data).borrow().prepare(
                        "SELECT id, dev_name, dir FROM Projects",
                    )?.query_map(
                        [],
                        |row| Ok(
                            Self {
                                id: row.get::<_, i32>(0)?,
                                dev_name: row.get::<_, String>(1)?,
                                dir: row.get::<_, String>(2)?,
                            },
                        ),
                    )?.collect(),
                ),
            ),
        );
    }
    pub fn remove_from_id(id: i32) -> Result<(), Error> {
        CONNECTION.with(
            |data| (*data).borrow().execute(
                "DELETE FROM Projects WHERE id = ?1",
                params![id],
            ),
        )?;
        return Ok(());
    }
    pub fn remove(self) -> Result<(), Error> {
        return Self::remove_from_id(self.id);
    }
    pub fn get(id: i32) -> Result<Self, Error>
    {
        return CONNECTION.with(
            |data| (*data).borrow().query_row(
                "SELECT id, dev_name, dir
                FROM Projects
                WHERE id = ?1;",
                params![id],
                |row| Ok(
                    Self {
                        id: row.get::<_, i32>(0)?,
                        dev_name: row.get::<_, String>(1)?,
                        dir: row.get::<_, String>(2)?,
                    },
                ),
            ),
        );
    }
    pub fn get_from_path(
        dir: impl ToString,
        dev_name: impl ToString,
    ) -> Result<Self, Error>
    {
        return CONNECTION.with(
            |data| (*data).borrow().query_row(
                "SELECT id, dev_name, dir
                FROM Projects
                WHERE dir = ?1 AND dev_name = ?2;",
                params![dir.to_string(), dev_name.to_string()],
                |row| Ok(
                    Self {
                        id: row.get::<_, i32>(0)?,
                        dev_name: row.get::<_, String>(1)?,
                        dir: row.get::<_, String>(2)?,
                    },
                ),
            ),
        );
    }
    pub fn insert(&self) -> Result<(), Error>
    {
        CONNECTION.with(
            |data| (*data).borrow().execute(
                "INSERT INTO Projects (id, dev_name, dir) VALUES (?1, ?2, ?3)",
                params![&self.id, &self.dev_name, &self.dir],
            ),
        )?;
        return Ok(());
    }
    pub fn get_max_id() -> Result<i32, Error>
    {
        return CONNECTION.with(
            |data| (*data).borrow().query_row(
                "SELECT id
                FROM Projects
                WHERE id = (SELECT MAX(id) FROM Projects);",
                [],
                |row| Ok(row.get::<_, i32>(0)?),
            ),
        );
    }
    pub fn rename(&mut self, value: String) -> Result<(), Error>
    {
        CONNECTION.with(
            |data| (*data).borrow().execute(
                "UPDATE Projects SET dev_name = ?1 WHERE id = ?2;",
                params![&value, self.id],
            ),
        )?;
        self.dev_name = value;
        return Ok(());
    }
    pub fn replace(&mut self, value: String) -> Result<(), Error>
    {
        CONNECTION.with(
            |data| (*data).borrow().execute(
                "UPDATE Projects SET dir = ?1 WHERE id = ?2;",
                params![value.clone(), self.id],
            ),
        )?;
        self.dir = value;
        return Ok(());
    }
}
