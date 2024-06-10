use std::env;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use std::process;

extern crate rusqlite;
use rusqlite::{Connection};

static DATABASE_FILENAME: &str = ".rusted_shut.db";

#[derive(Debug, PartialEq)]
enum MenuOption {
    ListPasswords = 1,
    EnterNewPassword = 2,
    Exit = 3,
    Invalid = -1
}

fn ensure_password_database_exists() -> Result<(), Box<dyn std::error::Error>> {
    // Checks if the database already exists under the user's home folder.
    // If not, creates the database (named .rusted_shut.db).
    let username = env::var("USER")
        .map_err(|_| "Unable to identify current user")?;
    let path = format!("/home/{}/{}", username, DATABASE_FILENAME);
    let db_path = PathBuf::from(&path);

    if db_path.exists() {
        return Ok(());
    }

    // Create the database
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("Failed to create database: {}", e))?;

    let conn = Connection::open(&path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS passwords (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            username    BLOB,
            password    BLOB,
            site        BLOB
        )",
        (),
    ).map_err(|e| format!("Failed to create database table: {}", e))?;

    println!("Password database initialized successfully at {}", path);

    Ok(())
}

fn handle_enter_new_password() {
    println!("EnterNewPassword selected");
    // TODO: Implement a function to enter a new password
}

fn handle_list_passwords() {
    println!("ListPasswords selected");
    // TODO: Implement a function to list passwords
}

fn print_menu_options_and_get_input() -> MenuOption {
    // Prints the menu options and takes input from the user. Returns the user
    // input as MenuOption.
    let menu_string = r#"
        Select an option:
        1. List passwords
        2. Enter a new password
        3. Exit
    "#;
    println!("{}", menu_string);

    let mut menu_selection = String::new();
    io::stdin().read_line(&mut menu_selection).expect("Failed to read line");

    // Convert the input to an i32
    match menu_selection.trim().parse::<i32>() {
        Ok(1) => MenuOption::ListPasswords,
        Ok(2) => MenuOption::EnterNewPassword,
        Ok(3) => MenuOption::Exit,
        _ => {
            println!("Invalid input, please select a valid option.");
            MenuOption::Invalid
        }
    }
}

fn main() -> io::Result<()> {
    // Main loop of the application. Prints a greeting message and calls other
    // functions based on users input.
    println!("Welcome to RustedShut - a CLI based password manager");

    if let Err(_) = ensure_password_database_exists() {
        process::exit(1);
    }

    let mut menu_selection;
    loop {
        menu_selection = print_menu_options_and_get_input();
        if menu_selection != MenuOption::Invalid {
            break;
        }
    }

    match menu_selection {
        MenuOption::ListPasswords => handle_list_passwords(),
        MenuOption::EnterNewPassword => handle_enter_new_password(),
        MenuOption::Exit => {
            process::exit(0);
        },
        _ => {
            // This case shouldn't be reached due to the loop logic
            println!("Invalid input");
        },
    }

    Ok(())
}

