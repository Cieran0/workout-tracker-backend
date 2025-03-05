use std::ptr::null;

use rusqlite::{params, Connection, Result, Error};

use crate::wt_types::{self, C_Set, C_Sets};


#[derive(Debug)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug)]
pub struct Exercise {
    pub id: u32,
    pub user_id: u32,
    pub name: String,
    pub description: Option<String>,
    pub muscle_group: Option<String>,
}

#[derive(Debug)]
pub struct Workout {
    pub id: u32,
    pub user_id: u32,
    pub name: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct WorkoutExercise {
    pub id: u32,
    pub workout_id: u32,
    pub exercise_id: u32,
}

#[derive(Debug)]
pub struct Set {
    pub id: u32,
    pub workout_exercise_id: u32,
    pub set_number: u32,
    pub weight: f64,
    pub reps: u32,
}

pub struct DatabaseHandler {
    pub conn: Connection,
}

impl DatabaseHandler {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(DatabaseHandler { conn })
    }

    pub fn register_user(&self, username: &str, password_hash: &str) -> Result<u32> {
        self.conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            params![username, password_hash],
        )?;
        let user_id = self.conn.last_insert_rowid() as u32;
        Ok(user_id)
    }

    pub fn delete_user(&self, user_id: u32) -> Result<()> {
        self.conn.execute("DELETE FROM users WHERE id = ?1", params![user_id])?;
        Ok(())
    }

    pub fn add_exercise_to_user(
        &self,
        user_id: u32,
        name: &str,
        description: Option<&str>,
        muscle_group: Option<&str>,
    ) -> Result<u32> {
        self.conn.execute(
            "INSERT INTO user_exercises (user_id, name, description, muscle_group) VALUES (?1, ?2, ?3, ?4)",
            params![user_id, name, description, muscle_group],
        )?;
        let exercise_id = self.conn.last_insert_rowid() as u32;
        Ok(exercise_id)
    }


    pub fn get_user_exercises(&self, user_id: u32) -> Result<Vec<Exercise>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, name, description, muscle_group FROM user_exercises WHERE user_id = ?1",
        )?;
        let exercise_iter = stmt.query_map(params![user_id], |row| {
            Ok(Exercise {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                muscle_group: row.get(4)?,
            })
        })?;

        let mut exercises = Vec::new();
        for exercise in exercise_iter {
            exercises.push(exercise?);
        }
        Ok(exercises)
    }

    pub fn is_valid_user(&self, user_id: u32) -> Result<u32> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM users WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![user_id])?;

        if let Some(row) = rows.next()? {
            Ok(user_id)
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }

    pub fn new_workout(&self, user_id: u32) -> Result<Workout> {
        println!("HERE 3");

        self.conn.execute(
            "INSERT INTO workouts (user_id) VALUES (?1)",
            params![user_id],
        )?;
        let workout_id = self.conn.last_insert_rowid() as u32;
        
        Ok(Workout {
            id: workout_id,
            user_id,
            name: None,
            start_time: "".to_string(),
            end_time: None,
            notes: None
        })
    }

    pub fn save_sets(&self, user_id: u32, sets: Vec<C_Set>) -> Result<u32> {
        println!("HERE 1");

        let db_sets: Vec<Set> = sets.iter().map(|x| -> Set {
            let db_set = Set {
                id: 0,
                workout_exercise_id: x.workout_exercise_id,
                set_number: x.set_number,
                weight: x.weight,
                reps: x.reps,
            };
            db_set
        }).collect();



        let workout = self.new_workout(user_id)?;
        println!("HERE 4");


        for db_set in &db_sets {
            let e = self.conn.execute(
                "INSERT INTO sets (workout_exercise_id, set_number, weight, reps) VALUES (?1, ?2, ?3, ?4)",
                params![
                    db_set.workout_exercise_id,
                    db_set.set_number,
                    db_set.weight,
                    db_set.reps,
                ],
            );

            match e {
                Ok(_) => {},
                Err(err) => {println!("Error: {}", err)},
            }
        }

        println!("HERE 2");

        Ok(db_sets.len().try_into().unwrap())
    }
}
