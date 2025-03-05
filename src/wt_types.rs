use std::{
    collections::HashMap, ffi::CString, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, sync::Arc, vec
};

#[repr(C)]
pub struct Exercise {
    pub id: u32,
    pub name: [u8; 64],
    pub body_part: [u8; 16],
}



#[derive(Copy, Clone)]
#[repr(C)]
pub struct C_Set {
    pub workout_exercise_id: u32,
    pub set_number: u32,
    pub weight: f64,
    pub reps: u32,
}

#[repr(C)]
pub struct C_Sets {
    pub content: *mut C_Set,
    pub count: u32,
}