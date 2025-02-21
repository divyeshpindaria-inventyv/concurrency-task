use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use axum::{routing::get, Json, Router};
use lazy_static::lazy_static;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            status: Status::OnCall,
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
            department: Department::Audit,
            status: Status::LoggedOut,
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
    let create_server = tokio::spawn(async {
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/get-users", get(get_users))
            .route("/get-calls", get(get_calls))
            .route("/get-assigned-calls", get(get_assigned_calls));

        // run our app with hyper, listening globally on port 3000
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

        println!("New call generated: {:?}", new_call);
        thread::sleep(Duration::from_secs(2));
    });

    let assign_call = thread::spawn(|| loop {
        {
            let mut calls = CALL_DATA.write().unwrap();
            let users = USER_DATA.read().unwrap();
            let mut assigned_calls = ASSIGNED_CALL_DATA.write().unwrap();

            if let Some(call) = calls.first().cloned() {
                if let Some(user) = users.iter().find(|u| u.department == call.department) {
                    if let Status::Available = user.status {
                        let assignment = AssignedCall {
                            user_id: user.id,
                            call_id: call.id,
                        };
                        assigned_calls.push(assignment.clone());

                        calls.retain(|c| c.id != call.id);

                        println!("Assigned Call {} to User {}", call.id, user.name);
                    }
                } else {
                    calls.remove(0);
                    calls.push(call.clone());

                    println!("No available user for Call {}. Moved to the end.", call.id);
                }
            }
        }

        thread::sleep(Duration::from_secs(2));
    });

    create_server.await.unwrap();
    create_call.join().unwrap();
    assign_call.join().unwrap();
}
