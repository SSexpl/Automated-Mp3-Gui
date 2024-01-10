// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Window, Runtime};
// use std::collections::HashMap;
// use id3::{Tag, TagLike};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::fs::{read_dir,OpenOptions, File, write};
use std::io::{Read, Write};
use std::io;
use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result};

mod types;
mod db;
mod json;
mod threading;

// Main Func

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![start_scrape_process, get_settings_data, save_settings, close_splashscreen, initialize_db, check_directory, long_job])
    .setup(|app| {
      let main_window = app.get_window("main").unwrap();
      let splashscreen_window = app.get_window("splashscreen").unwrap();

      json::init();
      db::init();

      // we perform the initialization code on a new task so the app doesn't freeze
      tauri::async_runtime::spawn(async move {
        // initialize your app here instead of sleeping :)
        println!("Initializing...");
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("Done initializing.");

        // After it's done, close the splashscreen and display the main window
        splashscreen_window.close().unwrap();
        main_window.show().unwrap();
      });
      
      Ok(())
    })
    .plugin(tauri_plugin_store::Builder::default().build())
    .run(tauri::generate_context!())
    .expect("failed to launch app");
}

// SplashScreen & Init Functions

#[tauri::command]
async fn close_splashscreen(window: Window) {
  // Close splashscreen
  window.get_window("splashscreen").expect("no window labeled 'splashscreen' found").close().unwrap();
  // Show main window
  window.get_window("main").expect("no window labeled 'main' found").show().unwrap();
}

// Save Settings

#[tauri::command]
fn save_settings(data: types::Settings) -> Result<(), ()> {
    let j = serde_json::to_string(&data);
    println!("{:?}", &j);
    let mut f = OpenOptions::new().write(true).truncate(true).open(json::get_settings_path()).expect("Unable to create file");
    f.write_all(j.unwrap().as_bytes()).expect("Unable to write data");
    Ok(())
}

#[tauri::command]
async fn start_scrape_process<R: Runtime>(window: tauri::Window<R>) -> Result<u32, ()> {
    let endpoints = vec![
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=1",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=2",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=3",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=4",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=5",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=6",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=7",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=8",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=9",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=10",
        "https://blog-app-service-gag1.onrender.com/api/post/posts?pageNo=0&limit=11",
    ];

    let num_workers = 4;

    let start_time = std::time::Instant::now();
    threading::threaded_execution(window, endpoints.clone(), num_workers);
    let elapsed_time = start_time.elapsed();

    println!("Threaded Execution Time: {:?}", elapsed_time);
    Ok(elapsed_time.as_secs().try_into().unwrap())
    // let start_time = std::time::Instant::now();
    // threading::non_threaded_execution(endpoints.clone());
    // let elapsed_time2 = start_time.elapsed();
    // window.emit("progress", 2).unwrap();
    // println!("Non-Threaded Execution Time: {:?}", elapsed_time2);
}

#[tauri::command]
fn check_directory(var: String) -> Result<bool, bool> {
    // println!("Started Directory Check @{}", var.clone());
    let paths = fs::read_dir(var.clone()).unwrap();
    for path in paths {
        let file_name = path.as_ref().unwrap().file_name();
        if file_name.clone().to_str().unwrap().ends_with(".mp3") {
            return Ok(true)
        }   
    }
    Ok(false)    
}

#[tauri::command(rename_all = "snake_case")]
async fn initialize_db<R: Runtime>(window: tauri::Window<R>, path_var: String) -> Result<u32, ()> {
// async fn initialize_db(path_var: String) -> Result<(), ()> {
    println!("Started Build");
    // window.emit("db_init_state", true).unwrap();
    let _ = db_populate(path_var.clone()).await;
    let num_paths: u32 = read_dir(path_var).unwrap().count().try_into().unwrap();    
    std::thread::sleep(std::time::Duration::from_secs(2));
    // window.emit("db_init_state", false).unwrap();
    window.emit("db_init_paths", num_paths).unwrap();
    Ok(num_paths)
}

async fn db_populate(path_var: String) -> Result<()> {
    let paths = fs::read_dir(path_var.clone()).unwrap();
    let conn = Connection::open("./.userData/Mp3data.db")?;
    println!("From DB populate");
    for path in paths {
        // let tag = Tag::read_from_path(path.as_ref())?;
        let file_name = path.as_ref().unwrap().file_name();
        let path_value = path.as_ref().unwrap().path();
        
        //Dont send into DB until API request has been made, then send ALL data. After that, send result to frontend - if successfull - begin preview screen, else show error

        let _ = conn.execute("INSERT INTO mp3_table_data (
            file_name, 
            path, 
            title, 
            artist, 
            album, 
            year, 
            track, 
            genre,
            comment, 
            album_artist, 
            composer, 
            discno, 
            successfulFieldCalls,
            successfulMechanismCalls,
            successfulQueries,
            totalFieldCalls,
            totalMechanismCalls,
            totalSuccessfulQueries,
            album_art
        ) VALUES (
            ?1,
            ?2,
            NULL,
            NULL,
            NULL,
            0,
            0,
            NULL,
            NULL,
            NULL,
            NULL,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            NULL
        )",(file_name.into_string().unwrap(), path_value.to_str().unwrap()));
    }

    println!("Process Completed @: {}", path_var);
    Ok(())
}

// Old Tauri Functions

#[tauri::command]
fn get_settings_data() -> types::Settings {

    let mut file = File::open(json::get_settings_path()).expect("Unable to open");

    // Read the file content into a String
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Unable to Read");

    // Deserialize the JSON content into your struct
    let parsed_json: types::Settings = serde_json::from_str(&content).expect("JSON was not well-formatted");
    parsed_json
}

#[tauri::command]
async fn long_job<R: Runtime>(window: tauri::Window<R>) {
    println!("Hello from BE");
    for i in 0..101 {
        // println!("{}", i.clone());
        window.emit("progress", i).unwrap();
        // window.emit("confirmation", i).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}