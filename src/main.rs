mod db;
mod password;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

#[derive(Parser)]
#[command(author, version, about = "Secure Password Generator with History")]
struct Args {
    /// Password length
    #[arg(short, long, default_value_t = 16)]
    length: usize,

    /// Include symbols
    #[arg(long)]
    symbols: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a password and save to history
    Save {
        /// Service name
        service: String,
        /// Username for the service
        #[arg(short, long, default_value = "")]
        username: String,
        /// Password length
        #[arg(short, long, default_value_t = 16)]
        length: usize,
        /// Include symbols
        #[arg(long)]
        symbols: bool,
    },
    /// Show password history for a service
    History {
        /// Service name
        service: String,
    },
    /// List all services with saved passwords
    List,
    /// Register an existing password to history
    Register {
        /// Service name
        service: String,
        /// Password to register
        password: String,
        /// Username for the service
        #[arg(short, long, default_value = "")]
        username: String,
    },
    /// Delete all history for a service
    Delete {
        /// Service name
        service: String,
    },
}

fn pad(s: &str, width: usize) -> String {
    let display_width = s.width();
    let spaces = width.saturating_sub(display_width);
    format!("{}{}", s, " ".repeat(spaces))
}

fn db_path() -> PathBuf {
    if let Ok(path) = std::env::var("GENPASSWD_EX_DB") {
        return PathBuf::from(path);
    }

    let home = dirs::home_dir().expect("Cannot determine home directory");
    let dir = home.join(".local/share/genpasswd_ex");
    std::fs::create_dir_all(&dir).expect("Failed to create data directory");
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("history_dev.db")
    } else {
        dir.join("history.db")
    }
}

fn main() {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    match args.command {
        None => {
            if args.length < 4 {
                eprintln!("Error: password length must be at least 4.");
                std::process::exit(1);
            }
            println!("Generated password: {}", password::generate_password(args.length, args.symbols));
        }

        Some(Command::Save { service, username, length, symbols }) => {
            if length < 4 {
                eprintln!("Error: password length must be at least 4.");
                std::process::exit(1);
            }
            let pwd = password::generate_password(length, symbols);
            println!("Generated password: {}", pwd);

            let db = db::Db::open(&db_path()).expect("Failed to open database");
            db.save(&service, &username, &pwd).expect("Failed to save password");
            if username.is_empty() {
                eprintln!("Saved to history for service \"{}\".", service);
            } else {
                eprintln!("Saved to history for service \"{}\" (user: {}).", service, username);
            }
        }

        Some(Command::Register { service, password, username }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            db.save(&service, &username, &password).expect("Failed to save password");
            if username.is_empty() {
                println!("Registered to history for service \"{}\".", service);
            } else {
                println!("Registered to history for service \"{}\" (user: {}).", service, username);
            }
        }

        Some(Command::History { service }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let entries = db.get_history(&service).expect("Failed to read history");
            if entries.is_empty() {
                println!("No history for service \"{}\".", service);
            } else {
                println!("History for \"{}\":", service);
                println!("{:>4}  {:<20}  {:<20}  {}", "ID", "Username", "Password", "Created At");
                println!("{}", "-".repeat(80));
                for e in &entries {
                    let user = if e.username.is_empty() { "-".to_string() } else { e.username.clone() };
                    println!("{:>4}  {:<20}  {:<20}  {}", e.id, user, e.password, e.created_at);
                }
            }
        }

        Some(Command::List) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let services = db.list_services().expect("Failed to list services");
            if services.is_empty() {
                println!("No services found.");
            } else {
                let col = services.iter()
                    .map(|(svc, _)| svc.width())
                    .max()
                    .unwrap_or(0)
                    .max("Service".len());
                println!("{}  Count", pad("Service", col));
                println!("{}", "-".repeat(col + 8));
                for (svc, cnt) in &services {
                    println!("{}  {:>5}", pad(svc, col), cnt);
                }
            }
        }

        Some(Command::Delete { service }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let n = db.delete_service(&service).expect("Failed to delete history");
            if n == 0 {
                println!("No history found for service \"{}\".", service);
            } else {
                println!("Deleted {} record(s) for service \"{}\".", n, service);
            }
        }
    }
}
