// Cargo.toml dependencies:
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// bson = "2.0"
// chrono = "0.4"
// uuid = "1.0"

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use bson::{self, doc};
use serde_json::json;
use uuid::Uuid;

type StrOrList = Option<Vec<String>>;

#[derive(Serialize, Deserialize)]
struct LogIndex {
    uid: String,
    key: String,
    doc: String,
    time: String,
    date: String,
    vec_model: String,
    description: Option<String>,
}

pub struct Log {
    #[allow(dead_code)]
    db: String,
    db_path: PathBuf,
    doc: String,
    doc_format: String,
    doc_entry: usize,
}

impl Log {
    pub fn new(db: &str, db_path: Option<&str>) -> Self {
        let mut log = Log {
            db: db.to_string(),
            db_path: db_path.map(PathBuf::from).unwrap_or_else(|| {
                let mut path = dirs::home_dir().unwrap();
                path.push(format!("{}.ox-db", db));
                path
            }),
            doc: String::new(),
            doc_format: "bson".to_string(),
            doc_entry: 0,
        };
        log.set_db();
        log.set_doc(None);
        log
    }

    fn set_db(&mut self) {
        fs::create_dir_all(&self.db_path).expect("Failed to create database directory");
    }

    fn set_doc(&mut self, doc: Option<&str>) {
        self.doc = doc.map_or_else(
            || format!("log-[{}]", Local::now().format("%d_%m_%Y")),
            |d| d.to_string(),
        );

        let doc_path = self.db_path.join(&self.doc);
        fs::create_dir_all(&doc_path).expect("Failed to create document directory");

        let file_content = self.load_data(&format!("{}.index", self.doc));
        if let Some(index) = file_content.get("ox-db_init") {
            if let Some(entry) = index.get("doc_entry") {
                self.doc_entry = entry.as_u64().unwrap() as usize;
            }
        }
    }

    pub fn push(
        &mut self,
        data: StrOrList,
        embeddings: StrOrList,
        data_story: StrOrList,
        key: Option<String>,
        doc: Option<String>,
    ) -> String {
        let doc_name = doc.unwrap_or_else(|| self.doc.clone());
        let uid = self.gen_uid(key.as_deref());

        if data.is_none() {
            panic!("ox-db: no prompt is given");
        }

        let embeddings = embeddings.or(data.clone());

        let index_data = LogIndex {
            uid: uid.clone(),
            key: key.unwrap_or_else(|| "key".to_string()),
            doc: doc_name.clone(),
            time: Local::now().format("%I:%M:%S_%p").to_string(),
            date: Local::now().format("%d_%m_%Y").to_string(),
            vec_model: "vec.model".to_string(),
            description: data_story.map(|v| v.join(", ")),
        };

        self._push(&uid, &json!(index_data), &format!("{}.index", doc_name));
        self._push(&uid, &json!(data), &doc_name);
        self._push(&uid, &json!(embeddings), &format!("{}.ox-vec", doc_name));

        uid
    }

    pub fn pull(
        &self,
        uid: Option<StrOrList>,
        key: Option<&str>,
        time: Option<&str>,
        date: Option<&str>,
        doc: Option<&str>,
        docfile: Option<&str>,
    ) -> Vec<serde_json::Value> {
        let doc_name = doc.unwrap_or_else(|| &self.doc);
        let docfile_name = docfile.unwrap_or_else(|| doc_name);
        let all_none = [key, time, date].iter().all(|x| x.is_none()) && uid.as_ref().map_or(true, |v| v.is_none());

        let mut log_entries = Vec::new();

        if all_none {
            let content = self.load_data(docfile_name);
            for (uid, data) in content.as_object().unwrap() {
                if uid == "ox-db_init" {
                    continue;
                }
                log_entries.push(json!({ "uid": uid, "data": data }));
            }
            return log_entries;
        }

        if let Some(uids) = uid {
            let content = self.load_data(docfile_name);
            for uid in uids.unwrap_or_default().iter() {
                if let Some(data) = content.get(uid) {
                    log_entries.push(json!({ "uid": uid, "data": data }));
                }
            }
            return log_entries;
        }

        if key.is_some() || time.is_some() || date.is_some() {
            let uids = self.search_uid(doc_name, key, time, date);
            let data = self.pull(Some(Some(uids)), None, None, None, Some(doc_name), Some(docfile_name));
            log_entries.extend(data);
            return log_entries;
        }

        log_entries
    }

    fn gen_uid(&self, key: Option<&str>) -> String {
        format!(
            "{}-{}-{}",
            self.doc_entry,
            key.unwrap_or("key"),
            Uuid::new_v4().to_string()
        )
    }

    fn load_data(&self, log_file: &str) -> serde_json::Value {
        let log_file_path = self.get_logfile_path(log_file);
        let mut file_content = String::new();
    
        if log_file_path.exists() {
            let mut file = File::open(&log_file_path).expect("Failed to open file");
            file.read_to_string(&mut file_content).expect("Failed to read file");
    
            if file_content.trim().is_empty() {
                // Initialize with default content if the file is empty
                file_content = json!({"ox-db_init": {"doc_entry": 0}}).to_string();
                self.save_data(log_file, &serde_json::from_str(&file_content).unwrap());
            }
        } else {
            // Initialize with default content if the file does not exist
            file_content = json!({"ox-db_init": {"doc_entry": 0}}).to_string();
            self.save_data(log_file, &serde_json::from_str(&file_content).unwrap());
        }
    
        // Parse JSON content
        match serde_json::from_str(&file_content) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {}", log_file, e);
                // Return a default empty JSON value in case of parsing failure
                json!({})
            }
        }
    }
    
    

    fn save_data(&self, log_file: &str, file_content: &serde_json::Value) {
        let log_file_path = self.get_logfile_path(log_file);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(log_file_path)
            .expect("Failed to open file");
    
        if self.doc_format == "bson" {
            let document = bson::to_document(file_content).unwrap();
            let bson_bytes = bson::to_vec(&document).unwrap();
            file.write_all(&bson_bytes).expect("Failed to write BSON to file");
        } else {
            file.write_all(file_content.to_string().as_bytes())
                .expect("Failed to write JSON to file");
        }
    }
    
    
    
    fn search_uid(
        &self,
        doc: &str,
        key: Option<&str>,
        time: Option<&str>,
        date: Option<&str>,
    ) -> Vec<String> {
        let content = self.load_data(&format!("{}.index", doc));
        let mut uids = Vec::new();

        let itime_parts: Vec<&str> = time.map_or(vec![], |t| t.split("_").collect());
        let idate_parts: Vec<&str> = date.map_or(vec![], |d| d.split("_").collect());

        for (uid, data) in content.as_object().unwrap() {
            if uid == "ox-db_init" {
                continue;
            }

            let log_entry: LogIndex = serde_json::from_value(data.clone()).unwrap();
            let time_parts: Vec<&str> = log_entry.time.split("_").collect();
            let date_parts: Vec<&str> = log_entry.date.split("_").collect();

            let mut log_it = false;

            if itime_parts.len() == 2 && time_parts.len() == 2 {
                log_it = itime_parts[0] == time_parts[0] && itime_parts[1] == time_parts[1];
            } else if idate_parts == date_parts {
                log_it = true;
            } else if key == Some(log_entry.key.as_str()) {
                log_it = true;
            }

            if log_it {
                uids.push(uid.clone());
            }
        }

        uids
    }

    fn get_logfile_path(&self, log_file: &str) -> PathBuf {
        let doc_path = self.db_path.join(&self.doc);
        doc_path.join(format!("{}.{}", log_file, self.doc_format))
    }

    fn _push(&mut self, uid: &str, data: &serde_json::Value, log_file: &str) {
        if data.is_null() {
            panic!("ox-db: no prompt is given");
        }

        let mut file_content = self.load_data(log_file);

        file_content[uid] = data.clone();

        if log_file.ends_with(".index") {
            if let Some(entry) = file_content["ox-db_init"].get_mut("doc_entry") {
                *entry = json!(entry.as_u64().unwrap() + 1);
                self.doc_entry = entry.as_u64().unwrap() as usize;
            }
        }

        self.save_data(log_file, &file_content);
        println!("ox-db: logged data: {} \n{}", uid, log_file);
    }
}

fn main() {
    // Initialize Log with example database and no specific path
    let mut log = Log::new("example", None);
    
    // Push a sample data entry
    log.push(Some(vec!["sample data".to_string()]), None, None, None, None);

    // Retrieve entries and print them
    let entries = log.pull(None, None, None, None, None, None);
    println!("Log Entries: {:?}", entries);
}


