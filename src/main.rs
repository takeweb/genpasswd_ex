mod db;
mod password;

use clap::{ArgGroup, Parser, Subcommand};
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Secure Password Generator with History",
    after_help = "Examples:
  genpasswd_ex save <SERVICE> -u <USERNAME> [-l <LENGTH>] [--symbols]
  genpasswd_ex register <SERVICE> '<PASSWORD>' -u <USERNAME>
  genpasswd_ex history <SERVICE>
  genpasswd_ex update <ID> [-u <USERNAME>] [-p '<PASSWORD>']

Run 'genpasswd_ex <COMMAND> -h' for command-specific options."
)]
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
    /// Generate a password and save to history (-u <USERNAME>)
    Save {
        /// Service name
        service: String,
        /// Username for the service (defaults to the latest one in history)
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
    /// Register an existing password to history (-u <USERNAME>)
    Register {
        /// Service name
        service: String,
        /// Password to register
        password: String,
        /// Username for the service (defaults to the latest one in history)
        #[arg(short, long, default_value = "")]
        username: String,
    },
    /// Update username and/or password of a history entry (-u <USERNAME> / -p <PASSWORD>)
    #[command(group(
        ArgGroup::new("fields").required(true).multiple(true).args(["username", "password"])
    ))]
    Update {
        /// Entry ID (shown in `history`)
        id: i64,
        /// New username
        #[arg(short, long)]
        username: Option<String>,
        /// New password
        #[arg(short, long)]
        password: Option<String>,
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

/// ユーザ名が省略された場合、同じサービスの履歴にある最新のユーザ名を使う
fn resolve_username(db: &db::Db, service: &str, username: String) -> String {
    if !username.is_empty() {
        return username;
    }
    db.latest_username(service)
        .expect("Failed to read history")
        .unwrap_or_default()
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
            let username = resolve_username(&db, &service, username);
            db.save(&service, &username, &pwd).expect("Failed to save password");
            if username.is_empty() {
                eprintln!("Saved to history for service \"{}\".", service);
            } else {
                eprintln!("Saved to history for service \"{}\" (user: {}).", service, username);
            }
        }

        Some(Command::Register { service, password, username }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let username = resolve_username(&db, &service, username);
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

        Some(Command::Update { id, username, password }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let n = db
                .update_entry(id, username.as_deref(), password.as_deref())
                .expect("Failed to update history");
            if n == 0 {
                eprintln!("Error: no history entry with ID {}.", id);
                std::process::exit(1);
            }
            println!("Updated history entry ID {}.", id);
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
