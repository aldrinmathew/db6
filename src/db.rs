use std::{fs, path::Path};

use crate::cli::Cli;

pub struct DB {
    path: String,
}

impl DB {
    pub fn create(cl: &mut Cli, name: String, password: String) -> Result<DB, String> {
        let db_dir = Path::new(cl.root.as_str()).join(&name);
        match fs::create_dir(db_dir.to_string_lossy().to_string()) {
            Ok(_) => {
                todo!();
            }
            Err(err) => {
                return Err(format!(
                    "Error while creating the directory {} for the database {}. The error is {}",
                    db_dir.to_string_lossy(),
                    name,
                    err
                ));
            }
        }
    }
}
