use dirs;
use std::path::Path;

pub enum CliCommand {
    Help,
    New(String, Option<String>, bool),
    Run,
}

pub struct Cli {
    pub root: String,
    pub port: u16,
    pub command: CliCommand,
}

impl Cli {
    pub fn new() -> Result<Cli, String> {
        let args: Vec<String> = std::env::args().collect();
        let mut root: Option<String> = None;
        let mut port: Option<u16> = None;
        let mut cmd: CliCommand;
        if args.len() == 1 {
            return Ok(Cli {
                root: match dirs::home_dir() {
                    Some(dir) => dir.join(".db6").to_string_lossy().to_string(),
                    None => {
                        return Err("Could not determine the home directory, where the default database directory resides in".to_string());
                    }
                },
                port: 6100,
                command: CliCommand::Help,
            });
        }
        match args[1].as_str() {
            "new" => {
                if args.len() == 2 {
                    return Err(
                        "Expected the name of the database to create after the 'new' command"
                            .to_string(),
                    );
                }
                let name = args[2].clone();
                for it in name.as_bytes() {
                    if !it.is_ascii_alphanumeric() && *it != b'_' {
                        return Err(
                            "Only alphanumeric characters or _ are allowed for the name of the database. Found invalid character "
                                .to_string() + &(*it as char).to_string(),
                        );
                    }
                }
                cmd = CliCommand::New(name, None, false);
            }
            "run" => {
                cmd = CliCommand::Run;
            }
            "help" => {
                cmd = CliCommand::Help;
            }
            val => {
                return Err("Invalid command ".to_string() + val + " provided");
            }
        }
        let mut ind = 2;
        while ind < args.len() {
            if args[ind] == "--root" {
                if ind + 1 >= args.len() {
                    return Err("Expected a path to be provided after '--root'".to_string());
                }
                root = Some(args[ind + 1].clone());
                ind += 1;
            } else if args[ind].starts_with("--root=") {
                root = Some(args[ind]["--root=".len()..].to_string());
            } else if args[ind] == "--port" {
                if ind + 1 >= args.len() {
                    return Err("Expected a port number to be provided after '--port'".to_string());
                }
                port = Some(match args[ind + 1].parse::<u16>() {
                    Ok(val) => val,
                    Err(err) => {
                        return Err(
                            "Error while parsing the port number: ".to_string() + &err.to_string()
                        );
                    }
                });
                ind += 1;
            } else if args[ind].starts_with("--port=") {
                port = Some(
                    match args[ind]["--port=".len()..].to_string().parse::<u16>() {
                        Ok(val) => val,
                        Err(err) => {
                            return Err("Error while parsing the port number: ".to_string()
                                + &err.to_string());
                        }
                    },
                );
            } else if args[ind] == "--password" {
                if ind + 1 >= args.len() {
                    return Err("Expected a password to be provided after '--password'".to_string());
                }
                match &mut cmd {
                    CliCommand::New(_, password, insecure) => {
                        if *insecure {
                            return Err("The '--insecure' flag was provided previously, but found the '--password' as well. \
                            These are conflicting configurations. '--insecure' is used to skip the requirement of a password.".to_string());
                        }
                        *password = Some(args[ind + 1].clone());
                    }
                    _ => {
                        return Err("The '--password' argument is only supported for the 'new' command, for creating a new database".to_string());
                    }
                }
                ind += 1;
            } else if args[ind].starts_with("--password=") {
                match &mut cmd {
                    CliCommand::New(_, password, insecure) => {
                        if *insecure {
                            return Err("The '--insecure' flag was provided previously, but found the '--password' as well. \
                            These are conflicting configurations. '--insecure' is used to skip the requirement of a password.".to_string());
                        }
                        *password = Some(args[ind]["--password=".len()..].to_string());
                    }
                    _ => {
                        return Err("The '--password' argument is only supported for the 'new' command, for creating a new database".to_string());
                    }
                }
            } else if args[ind] == "--insecure" {
                match &mut cmd {
                    CliCommand::New(_, password, insecure) => {
                        if password.is_some() {
                            return Err(
                                "The '--password' argument was provided previously, but found the '--insecure' flag as well. \
                                These are conflicting configurations. '--insecure' is used to skip the requirement of a password.".to_string(),
                            );
                        }
                        *insecure = true;
                    }
                    _ => {
                        return Err("The '--insecure' flag is only supported for the 'new' command, for creating a new database".to_string());
                    }
                }
            }
            ind += 1;
        }
        if root.is_some() {
            let root_path = Path::new(root.as_ref().unwrap());
            if !root_path.exists() {
                return Err(
                    "The provided path for the '--root' argument does not exist. Expected an existing directory to be provided".to_string()
                );
            } else if !root_path.is_dir() {
                return Err(
                    "The provided path for the '--root' argument is not a directory".to_string(),
                );
            }
        }
        Ok(Cli {
            root: root.unwrap_or(match dirs::home_dir() {
                Some(dir) => (dir.join(".db6")).to_string_lossy().to_string(),
                None => {
                    return Err("The '--root' argument was not provided to determine the root folder of the database installation. Also could not retrieve the home directory where the default database directory resides".to_string());
                }
            }),
            port: port.unwrap_or(6100),
            command: cmd,
        })
    }

    pub fn help(&self) {
        println!(
            "db6
Default configuration:
    root  =  {}
    port  =  {}

Commands
========
db6 new [name]
    Create a new database in the default root path, or the provided root path if it is available.
    You will be prompted for a password, which is the recommended way to provide the password.
    If you wish to provide the password through command line arguments, use the '--password'
    argument instead. 
    Supported arguments:
        --root     (Optional)
        --password (Optional)
    Supported flags:
        --insecure (Optional)
db6 run
    Start the database runtime from the default root path, or the provided root path if it is
    available. This command should be run once at startup, as a daemon possibly, to start the
    database runtime.
    Supported arguments:
        --root (Optional)
        --port (Optional)
db6 help
    Display this help message

Arguments
=========
 --root     (Optional) Path to the root folder of the database installation. This is the directory
            where all your databases are stored. It is not recommended to provide this unless you
            want to customize it.
            On Unix systems, the default value is '$HOME/.db6', where $HOME is the home directory
            of the current user.
            On Windows, the default value is 'C:/Users/CURRENT_USER/.db6', where
            CURRENT_USER is the username of the current user.
 --port     (Optional) The default port used by the database server is 6100. If you want to
            customize the port for a specific database runtime, then provide this argument. Unless
            you are dealing with multiple database runtimes in multiple root directories, it is
            not recommended to use this argument.
 --password (Optional) The password to be used to encrypt the database to be created. If you wish
            to avoid encryption of the database (which is not recommended), you can provide the
            --insecure flag instead. If this argument and the '--insecure' flag
            are not provided, then the user will be prompted for a password.
                                                                                                   
Flags
=====
 --insecure (Optional) To be used to skip providing a password. This is not recommended unless     
            you know what you are doing.                                                           
",
            self.root, self.port,
        );
    }
}
