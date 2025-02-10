use rusqlite::{params, Connection, Result};

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
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct Set {
    pub id: u32,
    pub workout_exercise_id: u32,
    pub set_number: u32,
    pub weight: f64,
    pub reps: u32,
    pub rpe: Option<f64>,
    pub notes: Option<String>,
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

    pub fn add_workout(
        &self,
        user_id: u32,
        name: Option<&str>,
        exercises: Vec<(u32, Vec<(f64, u32, Option<f64>, Option<&str>)>)>,
        notes: Option<&str>,
    ) -> Result<u32> {
        let start_time = chrono::Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO workouts (user_id, name, start_time, notes) VALUES (?1, ?2, ?3, ?4)",
            params![user_id, name, start_time, notes],
        )?;
        let workout_id = self.conn.last_insert_rowid() as u32;

        for (exercise_id, sets) in exercises {
            self.conn.execute(
                "INSERT INTO workout_exercises (workout_id, exercise_id) VALUES (?1, ?2)",
                params![workout_id, exercise_id],
            )?;
            let workout_exercise_id = self.conn.last_insert_rowid() as u32;

            for (set_number, (weight, reps, rpe, notes)) in sets.into_iter().enumerate() {
                self.conn.execute(
                    "INSERT INTO sets (workout_exercise_id, set_number, weight, reps, rpe, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![workout_exercise_id, set_number as u32 + 1, weight, reps, rpe, notes],
                )?;
            }
        }

        Ok(workout_id)
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
}
