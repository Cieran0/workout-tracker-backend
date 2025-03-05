use std::{
    collections::HashMap, ffi::CString, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, sync::Arc, vec
};

mod database_handler;
use database_handler::DatabaseHandler;
mod wt_types;
use wt_types::*;

#[link(name = "exercise")] unsafe extern "C" {
    unsafe fn exercises_to_json(exercises: *const Exercise, num: usize, out_buffer: *mut u8);
}

extern "C" {
    fn json_to_exercise(json_string: *const i8, user_id: u32) -> C_Sets;  // `*const i8` corresponds to `const char*` in C
    fn free_sets(sets: C_Sets);  // To free memory in C
}

fn send_json_to_c(json_contents: String, user_id: u32) -> Option<Vec<C_Set>> {
    let c_json_contents = CString::new(json_contents).unwrap();

    unsafe {

        let sets = json_to_exercise(c_json_contents.as_ptr(), user_id);
        
        if sets.count > 0 {
            println!("Received sets count: {}", sets.count);

            let sets_vec = std::slice::from_raw_parts(sets.content, sets.count.try_into().unwrap());

            for set in sets_vec  {
                println!("Exercise_ID: {}, Set_index: {}, Weight: {}, Reps: {}", set.workout_exercise_id, set.set_number, set.weight, set.reps);
            }

            let safe_sets: Vec<C_Set> = Vec::from(sets_vec);

            free_sets(sets);

            return Some(safe_sets);
        } else {
            println!("Failed to parse JSON or no sets found.");
        }

        None
    }
}

fn get_json(exercises: &[Exercise]) -> String {
    unsafe {
        let mut out_buffer: Vec<u8> = vec![0u8; 8192]; // Ensure buffer is large enough

        if exercises.is_empty() {
            return "[]".to_string();
        }
        
        exercises_to_json(exercises.as_ptr(), exercises.len(), out_buffer.as_mut_ptr());
        
        if let Some(pos) = out_buffer.iter().position(|&c| c == 0) {
            out_buffer.truncate(pos);
        }

        String::from_utf8(out_buffer).unwrap_or_else(|_| "Failed to parse JSON".to_string())
    }
}


fn convert_db_exercise(db_ex: &database_handler::Exercise) -> Exercise {
    Exercise {
        id: db_ex.id,
        name: string_to_cstring_64(&db_ex.name),
        body_part: string_to_cstring_16(db_ex.muscle_group.as_deref().unwrap_or("")),
    }
}

fn main() {
    let db_handler = Arc::new(DatabaseHandler::new("workout_tracker.db").expect("Failed to open database"));
    let listener = TcpListener::bind("0.0.0.0:25561").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let db_handler = Arc::clone(&db_handler);
        handle_connection(stream, &db_handler);
    }
}

fn handle_connection(mut stream: TcpStream, db_handler: &DatabaseHandler) {
    println!("New connection");

    let mut buf_reader = BufReader::new(&stream);
    let mut request_content = Vec::new();
    let mut content_length: usize = 0;

    for line in buf_reader.by_ref().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            break;
        }

        if let Some(len) = line.strip_prefix("Content-Length: ") {
            content_length = len.parse::<usize>().unwrap_or(0);
        }

        request_content.push(line);
    }

    let mut body = String::new();
    if content_length > 0 {
        let mut body_reader = buf_reader.take(content_length as u64);
        body_reader.read_to_string(&mut body).unwrap();
    }

    // println!("Headers:\n{}", request_content.join("\n"));
    // println!("Body:\n{}", body);

    let request_line = request_content[0].clone();

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        let response = build_response("HTTP/1.1 400 BAD REQUEST", "400 BAD REQUEST", "text/html");
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    let path_with_query = parts[1];
    let path_parts: Vec<&str> = path_with_query.splitn(2, '?').collect();
    let path = path_parts[0];
    let query_string = if path_parts.len() > 1 { path_parts[1] } else { "" };

    let query_params: HashMap<_, _> = query_string
        .split('&')
        .filter_map(|s: &str| {
            let mut kv: std::str::SplitN<'_, char> = s.splitn(2, '=');
            Some((kv.next()?.to_string(), kv.next().unwrap_or("").to_string()))
        })
        .collect();


    let (status_line, contents, content_type) = if path == "/exercises" {
        match query_params.get("userid").and_then(|s| s.parse::<u32>().ok()) {
            Some(userid) => match db_handler.get_user_exercises(userid) {
                Ok(db_exercises) => {
                    let exercises: Vec<Exercise> = db_exercises.iter().map(convert_db_exercise).collect();

                    let json_contents = get_json(&exercises);

                    ("HTTP/1.1 200 OK", json_contents, "application/json")
                }
                Err(_) => (
                    "HTTP/1.1 500 INTERNAL SERVER ERROR",
                    "<html><body>500 INTERNAL SERVER ERROR</body></html>".to_string(),
                    "text/html",
                ),
            },
            None => (
                "HTTP/1.1 400 BAD REQUEST",
                "<html><body>400 BAD REQUEST: Invalid or missing userid</body></html>".to_string(),
                "text/html",
            ),
        }
    } 
    else if path == "/workout" {
        match query_params.get("userid").and_then(|s| s.parse::<u32>().ok()) {
            Some(userid) => { 
                match db_handler.is_valid_user(userid) {
                    Ok(user_id) => {
                        println!("Request: {}", body);
                        let json_contents = format!(r#"{{ "user_id": {}, "success": true }}"#, user_id);
                        let json_body = body.trim();
                        let got_sets =  send_json_to_c(json_body.to_string(), user_id);
                        match got_sets {
                            None => (),
                            Some(sets) => {
                                db_handler.save_sets(userid, sets);
                                ()
                            }
                        }
                        ("HTTP/1.1 200 OK", json_contents, "application/json")

                    }
                    Err(e) => 
                    (
                        "HTTP/1.1 400 BAD REQUEST", format!(r#"{{ "error": "{}", "success": false }}"#, e.to_string()), "application/json"
                    )
                }
            }
            None => (
                "HTTP/1.1 400 BAD REQUEST",
                format!(r#"{{ "error": "{}", "success": false }}"#, "Invalid or missing userid"),
                "application/json",
            )
        }
    }    
    else {
        (
            "HTTP/1.1 404 NOT FOUND",
            "<html><body>404 NOT FOUND</body></html>".to_string(),
            "text/html",
        )
    };

    let response = build_response(status_line, &contents, content_type);
    stream.write_all(response.as_bytes()).unwrap();
}

fn build_response(status: &str, body: &str, content_type: &str) -> String {
    format!(
        "{status}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn string_to_cstring_64(s: &str) -> [u8; 64] {
    let mut arr = [0u8; 64];
    let bytes = s.as_bytes();
    arr[..bytes.len().min(64)].copy_from_slice(&bytes[..bytes.len().min(64)]);
    arr
}

fn string_to_cstring_16(s: &str) -> [u8; 16] {
    let mut arr = [0u8; 16];
    let bytes = s.as_bytes();
    arr[..bytes.len().min(16)].copy_from_slice(&bytes[..bytes.len().min(16)]);
    arr
}
