//db.mod.rs
use mysql::prelude::*;
use mysql::{params, Pool};

pub struct DbConnection {
    pool: Pool,
}

impl DbConnection {
    pub fn new(url: &str) -> Result<Self, mysql::Error> {
        let pool = Pool::new(url)?;
        Ok(Self { pool })
    }

    pub fn add_high_score(
        &self,
        name: &str,
        time: f32,
        difficulty: &str,
    ) -> Result<(), mysql::Error> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(
            "INSERT INTO high_scores (name, time, difficulty) VALUES (:name, :time, :difficulty)",
            params! {
                "name" => name,
                "time" => time,
                "difficulty" => difficulty,
            },
        )
    }

    pub fn get_top_10_scores(&self, difficulty: &str) -> Result<Vec<HighScore>, mysql::Error> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_map(
            "SELECT id, name, time, difficulty FROM high_scores WHERE difficulty = :difficulty ORDER BY time ASC LIMIT 10",
            params! {
                "difficulty" => difficulty,
            },
            |(id, name, time, difficulty)| HighScore {
                id,
                name,
                time,
                difficulty,
            },
        )
    }
}

#[derive(Debug)]
pub struct HighScore {
    pub id: i32,
    pub name: String,
    pub time: f32,
    pub difficulty: String,
}