use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use axum::{routing::get, Json, Router};
use lazy_static::lazy_static;
use log::info;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Status {
    OnCall,
    Available,
    Break,
    LoggedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Department {
    Sales,
    Renewal,
    Audit,
    Developer,
    Hr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    department: Department,
    status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Call {
    id: i32,
    details: String,
    department: Department,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssignedCall {
    user_id: i32,
    call_id: i32,
}

lazy_static! {
    static ref USER_DATA: Arc<RwLock<Vec<User>>> = Arc::new(RwLock::new(create_users()));
    static ref CALL_DATA: Arc<RwLock<Vec<Call>>> = Arc::new(RwLock::new(Vec::new()));
    static ref ASSIGNED_CALL_DATA: Arc<RwLock<Vec<AssignedCall>>> =
        Arc::new(RwLock::new(Vec::new()));
}

fn create_users() -> Vec<User> {
    vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            department: Department::Sales,
            status: Status::Available,
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            department: Department::Renewal,
            status: Status::Available,
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
            department: Department::Audit,
            status: Status::Available,
        },
        User {
            id: 4,
            name: "David".to_string(),
            department: Department::Developer,
            status: Status::Available,
        },
        User {
            id: 5,
            name: "Eve".to_string(),
            department: Department::Hr,
            status: Status::Available,
        },
    ]
}

fn random_id() -> i32 {
    rng().random_range(1..9999)
}

async fn get_users() -> Json<Vec<User>> {
    let users = USER_DATA.read().unwrap();
    Json(users.clone())
}

async fn get_calls() -> Json<Vec<Call>> {
    let calls = CALL_DATA.read().unwrap();
    Json(calls.clone())
}

async fn get_assigned_calls() -> Json<Vec<AssignedCall>> {
    let assigned_calls = ASSIGNED_CALL_DATA.read().unwrap();
    Json(assigned_calls.clone())
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let create_server = tokio::spawn(async {
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/get-users", get(get_users))
            .route("/get-calls", get(get_calls))
            .route("/get-assigned-calls", get(get_assigned_calls));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    let create_call = thread::spawn(|| loop {
        let call_id = random_id();
        let new_call = Call {
            id: call_id,
            details: format!("Call details for ID {}", call_id),
            department: match call_id % 5 {
                0 => Department::Sales,
                1 => Department::Renewal,
                2 => Department::Audit,
                3 => Department::Developer,
                _ => Department::Hr,
            },
        };

        {
            let mut calls = CALL_DATA.write().unwrap();
            calls.push(new_call.clone());
        }

        info!("New call generated: {:?}", new_call);
        thread::sleep(Duration::from_secs(2));
    });

    let assign_call = thread::spawn(|| loop {
        {
            let mut calls = CALL_DATA.write().unwrap();
            let mut users = USER_DATA.write().unwrap();
            let mut assigned_calls = ASSIGNED_CALL_DATA.write().unwrap();

            while let Some(call) = calls.pop() {
                if let Some(user) = users
                    .iter_mut()
                    .find(|u| u.department == call.department && u.status == Status::Available)
                {
                    user.status = Status::OnCall;
                    let assignment = AssignedCall {
                        user_id: user.id,
                        call_id: call.id,
                    };
                    assigned_calls.push(assignment);

                    info!("Assigned Call {} to User {}", call.id, user.name)
                }
            }
        }

        thread::sleep(Duration::from_secs(2));
    });

    let reset_status = thread::spawn(|| loop {
        {
            let mut users = USER_DATA.write().unwrap();
            let assigned_calls = ASSIGNED_CALL_DATA.read().unwrap();

            // Find users who are currently on a call
            let assigned_user_ids: Vec<i32> = assigned_calls.iter().map(|ac| ac.user_id).collect();

            for user in users.iter_mut() {
                if assigned_user_ids.contains(&user.id) {
                    // Simulate call completion by making the user available again
                    user.status = Status::Available;
                    info!("User {} is now available again", user.name)
                }
            }
        }

        thread::sleep(Duration::from_secs(10)); // Reset users every 5 seconds
    });

    create_server.await.unwrap();
    create_call.join().unwrap();
    assign_call.join().unwrap();
    reset_status.join().unwrap();
}
