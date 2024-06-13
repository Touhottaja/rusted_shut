use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
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

#[derive(Debug)]
struct Password {
    username: String,
    site: String,
    password: String,
}

fn ensure_password_database_exists() -> Result<PathBuf,
                                        Box<dyn std::error::Error>> {
    // Checks if the database already exists under the user's home folder.
    // If not, creates the database (named .rusted_shut.db).
    let username = env::var("USER")
        .map_err(|_| "Unable to identify current user")?;
    let path = format!("/home/{}/{}", username, DATABASE_FILENAME);
    let db_path = PathBuf::from(&path);

    if db_path.exists() {
        return Ok(db_path);
    }

    // Create the database
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&db_path)
        .map_err(|err| format!("Failed to create database: {}", err))?;

    let conn = Connection::open(&db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS passwords (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            username    BLOB,
            password    BLOB,
            site        BLOB
        )",
        (),
    ).map_err(|err| format!("Failed to create database table: {}", err))?;

    println!("Password database initialized successfully at {}", path);

    Ok(db_path)
}

fn handle_enter_new_password(db_path: PathBuf) -> Result<(),
                                                  Box<dyn std::error::Error>> {
    let username = match read_input("Enter the username: ") {
        Ok(username) => username,
        Err(err) => {
            return Err(Box::new(err));
        }
    };

    let password = match read_input("Enter the password: ") {
        Ok(password) => password,
        Err(err) => {
            return Err(Box::new(err));
        }
    };

    let site_or_note = match read_input("Enter a site or a note: ") {
        Ok(username) => username,
        Err(err) => {
            return Err(Box::new(err));
        }
    };

    // TODO: Encrypt details before inserting into database

    let conn = Connection::open(&db_path)?;
    conn.execute(
        "INSERT INTO passwords (username, password, site) VALUES (?1, ?2, ?3)",
        &[&username, &password, &site_or_note],
    ).map_err(|err| format!("Failed to insert password: {}", err))?;

    Ok(())
}

fn handle_list_passwords(db_path: PathBuf) -> Result<(),
                                              Box<dyn std::error::Error>> {
    // Lists the passwords stored in the database
    let conn = Connection::open(&db_path)?;
    let mut statement = conn.prepare("SELECT * FROM passwords")?;

    let password_iter = statement.query_map([], |row| {
        Ok(Password {
            // row.get(0) returns the column id
            username: row.get(1)?,
            password: row.get(2)?,
            site: row.get(3)?,
        })
    })?;

    for result in password_iter {
        // TODO: Decrypt the read values
        match result {
            Ok(result) => {
                println!("{:?}:{:?}         {:?}",
                         result.username,result.password, result.site);
            },
            Err(err) => {
                eprintln!("Error iterating results: {}", err);
            }
        }
    }

    Ok(())
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

fn read_input(prompt: &str) -> Result<String, io::Error> {
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush()?;

    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            Ok(input.trim().to_string())
        }
        Err(err) => Err(err),
    }
}

fn main() -> io::Result<()> {
    // Main loop of the application. Prints a greeting message and calls other
    // functions based on users input.
    println!("Welcome to RustedShut - a CLI based password manager");

    let db_path = match ensure_password_database_exists() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    let mut menu_selection;
    loop {
        menu_selection = print_menu_options_and_get_input();
        if menu_selection != MenuOption::Invalid {
            break;
        }
    }

    match menu_selection {
        MenuOption::ListPasswords => {
            if let Err(err) = handle_list_passwords(db_path) {
                eprintln!("Error fetching passwords: {}", err);
            }
        }
        MenuOption::EnterNewPassword => {
            if let Err(err) = handle_enter_new_password(db_path) {
                eprintln!("Error inserting a new entry: {}", err);
            }
        }
        MenuOption::Exit => {
            process::exit(0);
        },
        _ => {
            // This case shouldn't be reached due to the loop logic
            eprintln!("Invalid input");
        },
    }

    Ok(())
}

