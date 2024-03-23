use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: i32,
    name: String,
    completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: i32,
    name: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    tasks: HashMap<i32, Task>,
    users: HashMap<i32, User>,
}
#[allow(dead_code)]
impl Database {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            users: HashMap::new(),
        }
    }

    fn insert (&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }
    fn get(&self, id: &i32) -> Option<&Task> {
        self.tasks.get(&id)
    }

    fn get_all(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    fn delete (&mut self, id: &i32) {
        self.tasks.remove(&id);
    }

    fn update (&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    fn insert_user (&mut self, user: User) {
        self.users.insert(user.id, user);
    }


    fn get_user_by_name(&self, name: &str) -> Option<&User> {
        self.users.values().find(|user| user.name == name)
    }

    fn save_to_file(&self) -> std::io::Result<()> {
        let data: String = serde_json::to_string(&self)?;
        let mut file: fs::File = fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file_content: String = fs::read_to_string("database.json")?;
        let db: Database = serde_json::from_str(&file_content)?;
        Ok(db)
    }

    fn get_all_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    fn delete_user (&mut self, id: &i32) {
        self.users.remove(&id);
    }

    fn update_user (&mut self, user: User) {
        self.users.insert(user.id, user);
    }
    
}

struct AppState {
    db: Mutex <Database>
}

async fn create_task(app_state: web::Data <AppState>, task: web::Json <Task>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    db.insert(task.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn read_task(app_state: web::Data <AppState>, task: web::Path <i32>) -> impl Responder {
    let db = app_state.db.lock().unwrap();
    match db.get(&task.into_inner()) {
        Some(task) => HttpResponse::Ok().json(task),
        None => HttpResponse::NotFound().finish(),
        
    }
}

async fn read_all_task(app_state: web::Data <AppState>) -> impl Responder {
    let db = app_state.db.lock().unwrap();
    let tasks = db.get_all();
    if tasks.is_empty() {
        HttpResponse::NotFound().finish()
    } else {
        HttpResponse::Ok().json(tasks)
    }
}

async fn update_task(app_state: web::Data <AppState>, task: web::Json<Task>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    db.insert(task.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn delete_task(app_state: web::Data <AppState>, task: web::Path <i32>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    db.delete(&task.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn register(app_state: web::Data <AppState>, user: web::Json <User>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    db.insert_user(user.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn login(app_state: web::Data <AppState>, user: web::Json <User>) -> impl Responder {
    let db = app_state.db.lock().unwrap();
    match db.get_user_by_name(&user.name) {
        Some(stored_user)
            if stored_user.password == user.password => {
                HttpResponse::Ok().body("Login successful")
            },
        _ => HttpResponse::Unauthorized().body("Login failed")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new(),
    };

    let data: web::Data<AppState> = web::Data::new(AppState {
        db: Mutex::new(db),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(data.clone())
            .route("/task", web::post().to(create_task))
            .route("/task/{id}", web::get().to(read_task))
            .route("/tasks", web::get().to(read_all_task))
            .route("/task", web::put().to(update_task))
            .route("/task/{id}", web::delete().to(delete_task))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
