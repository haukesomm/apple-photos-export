use diesel::{Connection, SqliteConnection};

pub fn establish_connection(database_url: &String) -> SqliteConnection {
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}