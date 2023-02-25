use std::{
    env::args,
    error::Error,
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Note {
    content: String,
    id: String,
}

#[derive(Debug)]
enum Action {
    List,
    Get { id: String },
    Add { content: String },
    Patch { id: String, content: String },
    Delete { id: String },
}

#[derive(Debug)]
struct Args {
    action: Action,
    file: File,
}

fn read_notes(file: &File) -> Result<Vec<Note>, Box<dyn Error>> {
    Ok(serde_json::from_reader(file)?)
}

fn write_notes(notes: &Vec<Note>, file: &mut File) -> Result<(), Box<dyn Error>> {
    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;
    file.write_all(serde_json::to_string(&notes)?.as_bytes())?;
    Ok(())
}

fn format_note(Note { content, id }: &Note) -> String {
    format!("{id} -> {content}")
}

fn generate_id() -> String {
    const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let mut rng = rand::thread_rng();

    let id = rng
        .gen::<[u8; 8]>()
        .iter()
        .map(|byte| {
            let idx = (byte % CHARSET.len() as u8) as usize;
            CHARSET[idx] as char
        })
        .collect();
    id
}

fn parse_args() -> Result<Args, Box<dyn Error>> {
    let args: Vec<String> = args().collect();

    let file_path = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => return Err("file path must be provided".into()),
    };

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(file_path)?;

    if let Ok(metadata) = file.metadata() {
        if metadata.len() <= 0 {
            file.write_all(b"[]")?;
            file.seek(SeekFrom::Start(0))?;
        }
    };

    let action = match args
        .get(2)
        .ok_or_else(|| "action must be provided")?
        .as_str()
    {
        "list" => Action::List,
        "get" => Action::Get {
            id: args
                .get(3)
                .ok_or_else(|| "id must be provided")?
                .to_string(),
        },
        "add" => Action::Add {
            content: args
                .get(3)
                .ok_or_else(|| "content must be provided")?
                .to_string(),
        },
        "patch" => Action::Patch {
            id: args
                .get(3)
                .ok_or_else(|| "id must be provided")?
                .to_string(),
            content: args
                .get(4)
                .ok_or_else(|| "content must be provided")?
                .to_string(),
        },
        "delete" => Action::Delete {
            id: args
                .get(3)
                .ok_or_else(|| "id must be provided")?
                .to_string(),
        },
        _ => return Err("unknown action".into()),
    };

    Ok(Args { file, action })
}

fn main() -> Result<(), Box<dyn Error>> {
    let Args { action, mut file } = parse_args()?;

    match action {
        Action::List => {
            let notes = read_notes(&file)?;

            if notes.len() > 0 {
                for note in notes {
                    println!("{}", format_note(&note));
                }
            } else {
                println!("No notes found");
            }
        }

        Action::Get { id } => {
            let notes = read_notes(&file)?;

            let note = notes
                .iter()
                .find(|note| note.id == id)
                .ok_or_else(|| "note not found")?;

            println!("{}", format_note(&note));
        }

        Action::Add { content } => {
            let mut notes = read_notes(&file)?;

            let note = Note {
                id: generate_id(),
                content,
            };

            notes.push(note.clone());

            write_notes(&notes, &mut file)?;

            println!("{}", format_note(&note));
        }

        Action::Patch { id, content } => {
            let mut notes = read_notes(&file)?;

            if let Some(note) = notes.iter_mut().find(|note| note.id == id) {
                note.content = content;
                println!("{}", format_note(&note));
            } else {
                return Err("note not found".into());
            }

            write_notes(&notes, &mut file)?;
        }

        Action::Delete { id } => {
            let mut notes = read_notes(&file)?;

            let initial_len = notes.len();
            notes.retain(|note| note.id != id);

            if notes.len() == initial_len {
                return Err("note not found".into());
            }

            write_notes(&notes, &mut file)?;
        }
    }

    Ok(())
}
