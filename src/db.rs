// Keaton Clark
// 04/03/23
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::io::{stdin, stdout, Write};
use std::process;
use serde::{Serialize, Deserialize};
use rand::{Rng, distributions::Alphanumeric};

#[derive(Parser)]
#[grammar = "sql.pest"]
struct SQLParser;

#[derive(Serialize, Deserialize)]
pub struct DBMS {
    databases: HashMap<String, DataBase>,
    curr_db: Option<String>,
    path: Option<String>,
    id: String,
}

/// Data Base management system
/// Holds a HashMap of databases, path to save to, and current 'USE'ed database
impl DBMS {
    const PROMPT: &str = " > ";

    /// Creates a new DBMS.
    /// path can be none or filesystem path to serde_json encoded DBMS
    pub fn new(path: Option<String>) -> Self {
        match path {
            None => {
                Self {
                    databases: HashMap::new(),
                    curr_db: None,
                    path,
                    id: rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(7)
                        .map(char::from)
                        .collect(),
                }
            }
            Some(unwrapped_path) => {
                match std::fs::read_to_string(unwrapped_path.as_str()) {
                    Ok(file) => {
                        let mut dbms : Self = serde_json::from_str(file.as_str()).unwrap();
                        dbms.id = rand::thread_rng()
                                .sample_iter(&Alphanumeric)
                                .take(7)
                                .map(char::from)
                                .collect();
                        dbms
                    }
                    Err(_) => {
                        Self {
                            databases: HashMap::new(),
                            curr_db: None,
                            path: Some(unwrapped_path),
                            id: rand::thread_rng()
                                .sample_iter(&Alphanumeric)
                                .take(7)
                                .map(char::from)
                                .collect(),
                        }
                    }
                }
            }
        }
    }

    /// Opens an interactive prompt and parses and runs data fed to it
    pub fn interactive(&mut self) {
        let mut line = String::new();
        loop {
            stdout().write(DBMS::PROMPT.as_bytes()).unwrap();
            stdout().flush().unwrap();
            stdin().read_line(&mut line).unwrap();
            self.load();
            match SQLParser::parse(Rule::SQL, &line) {
                Ok(k) => {
                    for command in k {
                        match self.run(command) {
                            Ok(k) => {
                                match k {
                                    Some(s) => println!("{}", s),
                                    None => ()
                                }
                            }
                            Err(e) => println!("{e}")
                        }
                    }
                },
                Err(e) => println!("Error parsing\n{}", e)
            }
            if !std::path::Path::new(&format!("{}.lock", self.path.as_ref().unwrap())).exists() {
                self.save();
            }
            line.clear();
        }
    }
    
    /// Runs sql from a file.sql located at path
//    pub fn sql_from_file(&mut self, path: &str) {
//        match fs::read_to_string(path) {
//            Err(e) => println!("Error reading from {}\n{}", path,  e),
//            Ok(k) => {
//                match SQLParser::parse(Rule::SQL, &k) {
//                    Ok(k) => {
//                        for command in k {
//                            match self.run(command) {
//                                Ok(k) => {
//                                    match k {
//                                        Some(s) => {
//                                            println!("{}", s);
//                                            self.save();
//                                        },
//                                        None => ()
//                                    }
//                                }
//                                Err(e) => println!("{e}")
//                            }
//                        }
//                    },
//                    Err(e) => println!("Error parsing {}\n{}", path, e)
//                }
//            }
//        }
//    }

    /// runs already parsed commands
    /// it is just a big brancing switch-case
    fn run(&mut self, command: Pair<Rule>) -> Result<Option<String>, String>{
        match command.as_rule() {
            Rule::create => {
                let mut it = command.into_inner();
                match it.next().unwrap().as_rule() {
                    Rule::database => {
                        let name = it.next().unwrap().as_str();
                        if self.databases.contains_key(name) {
                            Err(format!("!Failed to create database {} because it already exists.", name))
                        } else {
                            self.databases.insert(String::from(name), DataBase::new());
                            Ok(Some(format!("Database {} created.", name)))
                        }
                    },
                    Rule::table => {
                        match &self.curr_db {
                            Some(db) => {
                                match self.databases.get_mut(db) {
                                    None => Err(format!("!Database {} was deleted", db)),
                                    Some(db) => {
                                        db.create(it)
                                    }
                                }
                            },
                            None => Err(format!("!No database supplied"))
                        }
                    },
                    _ => Err(format!("An uknown parsing error happened on line: {}", line!()))
                }
            },
            Rule::insert => {
                match &self.curr_db {
                    Some(db) => {
                        match self.databases.get_mut(db) {
                            None => Err(format!("!Database {} was deleted", db)),
                            Some(db) => {
                                db.insert(command.into_inner())
                            }
                        }
                    },
                    None => Err(format!("!No database supplied"))
                }

            }
            Rule::drop => {
                let mut it = command.into_inner();
                match it.next().unwrap().as_rule() {
                    Rule::database => {
                        let name = it.next().unwrap().as_str();
                        if self.databases.contains_key(name) {
                            self.databases.remove(&String::from(name));
                            Ok(Some(format!("Database {} deleted.", name)))
                        } else {
                            Err(format!("!Failed to delete database {} because it does not exist.", name))
                        }
                    },
                    Rule::table => {
                        match &self.curr_db {
                            Some(db) => {
                                match self.databases.get_mut(db.as_str()) {
                                    Some(db) => db.drop(it),
                                    None => Err(format!("!Database {} was deleted", db.as_str()))
                                }
                            },
                            None => Err(format!("!No database supplied."))
                        }
                    }
                    _ => Err(format!("An uknown parsing error happened on line: {}", line!()))
                }
            },
            Rule::_use => {
                let mut it = command.into_inner();
                let name = it.next().unwrap().as_str();
                if self.databases.contains_key(name) {
                    self.curr_db = Some(String::from(name));
                    Ok(Some(format!("Using database {}.", name)))
                } else {
                    Err(format!("!Cannot use database {} as it does not exist", name))
                }
            },
            Rule::exit => {
                process::exit(0);
            },
            Rule::semicolon => {
                Ok(None)  
            },
            Rule::select => {
                match &self.curr_db {
                    Some(db) => {
                        if self.databases.contains_key(db.as_str()) {
                            match self.databases.get(db).unwrap().select(command.into_inner()) {
                                Ok(k) => Ok(Some(k.unwrap())),
                                Err(e) => Err(e)
                            }
                        } else {
                            Err(format!("!Database {} was deleted", db.as_str()))
                        }
                    },
                    None => Err(format!("!No database supplied"))
                }
            },
            Rule::alter => {
                let mut it = command.into_inner();
                match it.next().unwrap().as_rule() {
                    Rule::table => {
                        match &self.curr_db {
                            Some(db) => {
                                match self.databases.get_mut(db) {
                                    Some(db) => {
                                        db.alter(it)
                                    },
                                    None => Err(format!("!Database {} was deleted.", db.as_str()))
                                }
                            },
                            None => Err(format!("!No database supplied."))
                        }
                    },
                    _ => Err(format!("An uknown parsing error happened"))
                }
            },
            Rule::update => {
                match &self.curr_db {
                    Some(db) => {
                        match self.databases.get_mut(db) {
                            None => Err(format!("!Database {} was deleted", db)),
                            Some(db) => {
                                db.update(command.into_inner(), self.id.as_str())
                            }
                        }
                    },
                    None => Err(format!("!No database supplied"))
                }
            },
            Rule::delete => {
                match &self.curr_db {
                    Some(db) => {
                        match self.databases.get_mut(db) {
                            None => Err(format!("!Database {} was deleted", db)),
                            Some(db) => {
                                db.delete(command.into_inner())
                            }
                        }
                    },
                    None => Err(format!("!No database supplied"))
                }
            },
            Rule::EOI => {
                Ok(None)
            },
            Rule::begin => {
                if !std::path::Path::new(&format!("{}.lock", self.path.as_ref().unwrap())).exists() {
                    std::fs::File::create(&format!("{}.lock", self.path.as_ref().unwrap())).unwrap().write(self.id.as_bytes()).unwrap();
                }
                Ok(Some(format!("Transaction starts.")))
            }
            Rule::commit => {
                if std::path::Path::new(&format!("{}.lock", self.path.as_ref().unwrap())).exists() {
                    if std::fs::read_to_string(&format!("{}.lock", self.path.as_ref().unwrap())).unwrap() == self.id {
                        std::fs::remove_file(&format!("{}.lock", self.path.as_ref().unwrap())).unwrap();
                        Ok(Some(format!("Transaction Committed")))
                    } else {
                        Err(format!("Transaction Aborted."))
                    }
                } else {
                    Err(format!("Transaction Aborted."))
                }
            }
            _ => Err(format!("Command \"{}\" was parsed but could not be ran", command.as_str()))
        }
    }

    /// saves serde_json encoded data to self.path
    fn save(&mut self) {
        match &self.path {
            Some(path) => {
                let mut f = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open(path).unwrap();
                let ser = serde_json::to_string(&self).unwrap();
                f.write_all(ser.as_bytes()).unwrap();
                f.flush().unwrap();
            },
            None => ()
        }
    }
    fn load(&mut self) {
        match &self.path {
            Some(path) => {
                if std::path::Path::new(path).exists() {
                    if std::path::Path::new(&format!("{}.lock", path)).exists() {
                        if std::fs::read_to_string(&format!("{}.lock", self.path.as_ref().unwrap())).unwrap() != self.id {
                            let s = std::fs::read_to_string(path).unwrap();
                            let ser : DBMS = serde_json::from_str(s.as_str()).unwrap();
                            self.databases = ser.databases;
                        }
                    } else {
                        let s = std::fs::read_to_string(path).unwrap();
                        let ser : DBMS = serde_json::from_str(s.as_str()).unwrap();
                        self.databases = ser.databases;
                    }
                }
            }
            None => (),
        }
    }
}

/// DataBase that holds a hashmap of tables
#[derive(Serialize,Deserialize)]
struct DataBase {
    tables: HashMap<String, Table>,
}

impl DataBase {

    /// Creates a new empty database
    fn new() -> Self {
        Self {
            tables: HashMap::new()
        }
    }

    /// Updates a table
    fn update(&mut self, mut list: Pairs<Rule>, id: &str) -> Result<Option<String>, String> {
        let table_name = list.next().unwrap().as_str();
        match self.tables.get_mut(table_name) {
            Some(table) => {
                table.update(list, id)
            },
            None => Err(format!("!Failed to insert into table {} as it does not exist.", table_name))
        }
    }
    /// Deletes part of a table
    fn delete(&mut self, mut list: Pairs<Rule>) -> Result<Option<String>, String> {
        let table_name = list.next().unwrap().as_str();
        match self.tables.get_mut(table_name) {
            Some(table) => {
                table.delete(list)
            },
            None => Err(format!("!Failed to insert into table {} as it does not exist.", table_name))
        }
    }

    /// Drops a table from this database
    fn drop(&mut self, mut list: Pairs<Rule>) -> Result<Option<String>, String> {
        let table_name = list.next().unwrap().as_str();
        if self.tables.contains_key(table_name) {
            self.tables.remove(table_name);
            Ok(Some(format!("Table {} deleted.", table_name)))
        } else {
            Err(format!("!Failed to delete {} because it does not exist", table_name))
        }
    }
    
    /// Inserts data into table
    fn insert(&mut self, mut list: Pairs<Rule>) -> Result<Option<String>, String> {
        let table_name = list.next().unwrap().as_str();
        match self.tables.get_mut(table_name) {
            Some(table) => {
                table.insert(list.next().unwrap())
            },
            None => Err(format!("!Failed to insert into table {} as it does not exist.", table_name))
        }
    }

    /// Alters a table in the database
    fn alter(&mut self, mut list: Pairs<Rule>) -> Result<Option<String>, String> {
        let table_name = list.next().unwrap().as_str();
        match self.tables.get_mut(table_name) {
            Some(table) => {
                match table.alter(list) {
                    Ok(_) => Ok(Some(format!("Table {} modified", table_name))),
                    Err(_) => Err(format!("Table {} not modified", table_name))
                }
            }
            None => Err(format!("!Failed to modify table {} as it does not exist", table_name))
        }
    }

    /// Selects a table in the database
    fn select(&self, mut list: Pairs<Rule>) -> Result<Option<String>, String> {
        let select_args = list.next().unwrap();
        let table_name = list.nth(1).unwrap().as_str();
        match self.tables.get(table_name) {
            Some(table) => {
                table.select(select_args)
            },
            None => Err(format!("!Failed to query {} as it does not exist", table_name))
        }
    }

    /// Creates a table in the database
    fn create(&mut self, mut list: Pairs<Rule>) -> Result<Option<String>, String> {
        let name = list.next().unwrap().as_str();
        match self.tables.get(name) {
            Some(_) => Err(format!("!Failed to create table {} because it already exists.", name)),
            None => {
                self.tables.insert(String::from(name), Table::new(list));
                Ok(Some(format!("Table {} created.", name)))
            }
        }
    }
}

/// Allows the header to have names and metadata and type of each column in a vector of the same type
#[derive(Debug, Serialize, Deserialize)]
enum SQLHeaderDef {
    Char(String, u32),
    Varchar(String, u32),
    Float(String),
    Int(String),
}

#[derive(Debug, Serialize, Deserialize)]
enum SQLColumn {
    Char(Vec<String>),
    Float(Vec<f64>),
    Int(Vec<i64>)
}

/// Currently just holds the header data of the table
#[derive(Serialize, Deserialize)]
struct Table {
    header: Vec<SQLHeaderDef>,
    data: Vec<SQLColumn>,
    len: usize
}

impl Table {
    /// Creates a new table and populates the header
    fn new(mut list: Pairs<Rule>) -> Self {
        let mut header: Vec<SQLHeaderDef> = Vec::new();
        let mut data: Vec<SQLColumn> = Vec::new();
        for element in list.next().unwrap().into_inner() {
            match element.as_rule() {
                Rule::columnDef => {
                    let mut it = element.into_inner();
                    let name = it.next().unwrap().as_str();
                    let column = it.next().unwrap();
                    match column.as_rule() {
                        Rule::char => {
                            data.push(SQLColumn::Char(Vec::new()));
                            header.push(
                                SQLHeaderDef::Char(
                                    String::from(name), 
                                    column.into_inner().nth(1).unwrap().as_str().parse::<u32>().unwrap())
                            )
                        },
                        Rule::varchar => {
                            data.push(SQLColumn::Char(Vec::new()));
                            header.push(
                                SQLHeaderDef::Varchar(
                                    String::from(name), 
                                    column.into_inner().nth(1).unwrap().as_str().parse::<u32>().unwrap())
                            )
                        }
                        Rule::float => {
                            data.push(SQLColumn::Float(Vec::new()));
                            header.push(
                                SQLHeaderDef::Float(
                                    String::from(name)
                                )
                            )
                        }
                        Rule::int => {
                            data.push(SQLColumn::Int(Vec::new()));
                            header.push(
                                SQLHeaderDef::Int(
                                    String::from(name)
                                )
                            )
                        }
                        _ => (),
                    };
                }
                _ => ()
            }
        }
        Self {
            header,
            data,
            len: 0,
        }
    }
    /// Updates entries in the table
    fn update(&mut self, mut list: Pairs<Rule>, id: &str) -> Result<Option<String>, String> {
        if !std::path::Path::new(&format!("{}.lock", "dbms")).exists() || std::fs::read_to_string(&format!("{}.lock", "dbms")).unwrap() == id {
            let set_name = list.nth(0).unwrap().as_str();
            let mut _set_idx = 0;
            let set_val = list.nth(0).unwrap().as_str().trim().parse::<i64>().unwrap();
            let mut list = list.nth(0).unwrap().into_inner();
            let where_name = list.nth(0).unwrap().as_str();
            let mut _where_idx = 0;
            let where_val = list.nth(1).unwrap().as_str().trim().parse::<i64>().unwrap();
            //let mut set_val;
            for i in 0..self.header.len() {
                if let SQLHeaderDef::Int(name) = self.header.get_mut(i).unwrap() {
                    if name.as_str() == where_name {
                        _where_idx = i;
                    }
                    if name.as_str() == set_name {
                        _set_idx = i;
                    }
                }
            }
            if let (SQLColumn::Int(where_vals), [SQLColumn::Int(set_vals)]) = self.data.split_first_mut().unwrap() {
                for i in 0..self.len {
                    if where_vals[i] == where_val {
                        set_vals[i] = set_val;
                    }
                }
            }
            Err(String::from("1 record modified"))
        } else {
            Err(format!("Error: Table {} is locked!", "Flights"))
        }
    }
    /// Deletes entries in the table
    fn delete(&mut self, _list: Pairs<Rule>) -> Result<Option<String>, String> {
        Err(String::from(""))
    }
    /// Inserts new values into table
    fn insert(&mut self, list: Pair<Rule>) -> Result<Option<String>, String> {
        let vals = list.into_inner();
        let mut i = 0;
        for val in vals {
            match val.as_rule() {
                Rule::columnVal => {
                    let val = val.into_inner().next().unwrap();
                    match val.as_rule() {
                        Rule::floatVal => {
                            if let SQLColumn::Int(ref mut col) = self.data.get_mut(i).unwrap() {
                                col.push(val.as_str().parse::<i64>().unwrap());
                                i += 1;
                            } else if let SQLColumn::Float(ref mut col) = self.data.get_mut(i).unwrap() {
                                col.push(val.as_str().parse::<f64>().unwrap());
                                i += 1;
                            }
                        },
                        Rule::charVal => {
                            if let SQLColumn::Char(ref mut col) = self.data.get_mut(i).unwrap() {
                                col.push(String::from(val.as_str()));
                                i += 1;
                            }
                        },
                        Rule::intVal => {
                            if let SQLColumn::Int(ref mut col) = self.data.get_mut(i).unwrap() {
                                col.push(val.as_str().parse::<i64>().unwrap());
                                i += 1;
                            }
                        }
                        _ => ()
                    }
                },
                _ => ()
            }
        }
        self.len += 1;
        Ok(Some(format!("1 new record inserted")))
    }
    /// Selects what is needed from the table
    fn select(&self, _list: Pair<Rule>) -> Result<Option<String>, String> {
        let mut out = String::new();
        for column in &self.header {
            match column {
                SQLHeaderDef::Int(name) => {
                    out.push_str(format!("{} int | ", name).as_str());
                },
                SQLHeaderDef::Char(name, size) => {
                    out.push_str(format!("{} char({}) | ", name, size).as_str());
                },
                SQLHeaderDef::Float(name) => {
                    out.push_str(format!("{} float | ", name).as_str());
                },
                SQLHeaderDef::Varchar(name, size) => {
                    out.push_str(format!("{} varchar({}) | ", name, size).as_str());
                },
            }
        }
        for i in 0..self.len {
            out.pop();
            out.pop();
            out.push_str("\n");
            for j in 0..self.data.len() {
                match self.data.get(j).unwrap() {
                    SQLColumn::Int(val) => out.push_str(format!("{} | ", val.get(i).unwrap()).as_str()),
                    SQLColumn::Char(val) => out.push_str(format!("{} | ", val.get(i).unwrap()).as_str()),
                    SQLColumn::Float(val) => out.push_str(format!("{} | ", val.get(i).unwrap()).as_str()),
                }
            }
        }
        out.pop();
        out.pop();
        Ok(Some(out))
    }
    
    /// Alters the table
    fn alter(&mut self, mut list: Pairs<Rule>) -> Result<Option<String>, String> {
        match list.next().unwrap().as_rule() {
            Rule::add => {
                for element in list.next().unwrap().into_inner() {
                    let mut it = element.into_inner();
                    let name = it.next().unwrap().as_str();
                    let column = it.next().unwrap();
                    match column.as_rule() {
                        Rule::char => {
                            self.header.push(
                                SQLHeaderDef::Char(
                                    String::from(name), 
                                    column.into_inner().nth(1).unwrap().as_str().parse::<u32>().unwrap())
                            )
                        },
                        Rule::varchar => {
                            self.header.push(
                                SQLHeaderDef::Varchar(
                                    String::from(name), 
                                    column.into_inner().nth(1).unwrap().as_str().parse::<u32>().unwrap())
                            )
                        }
                        Rule::float => {
                            self.header.push(
                                SQLHeaderDef::Float(
                                    String::from(name)
                                )
                            )
                        }
                        Rule::int => {
                            self.header.push(
                                SQLHeaderDef::Int(
                                    String::from(name)
                                )
                            )
                        }
                        _ => (),
                    };
                }
                Ok(Some(String::from("Table {} modified.")))
            },
            _ => Err(String::from("Table {} not modified"))
        }
    }
}
