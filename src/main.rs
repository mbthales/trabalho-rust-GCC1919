use csv::Reader;
use csv::WriterBuilder;
use std::fs::OpenOptions;
use chrono::{Local};
use uuid::{Uuid};

//Create only to add to the Pool struct
#[derive(Debug)]
struct Vote {
    choice: String,
}

#[derive(Debug)]
struct Poll {
    id: Uuid, //Could have used a sequential one but I find it easier
    question: String,
    create_date: i64,
    expiration_date: i64,
    votes: Vec<Vote>,
    total_votes: i16,
}

fn main() {
    let mut polls = vec![Poll::create_poll("Do you like Rust?", 7), Poll::create_poll("Do you like Python?", 3)];
    
    println!("Added Polls: {:?}", polls);

    polls[0].edit_poll("Have you done the Hello World example?");
    println!("\nPoll after edit: {:?}", polls);

    // Get id to delete
    let id_to_delete = polls[0].id;
    delete_poll(&mut polls, id_to_delete);

    println!("\nPoll after delete: {:?}", polls);

    
    let file_name = "bd.csv";
    
    csv_writer(file_name);
    csv_reader(file_name);

}

impl Poll {
    // Receive the question and the duration in days
    fn create_poll(question: &str, days_until_expiration: i64) -> Poll {
        let create_date = Local::now().timestamp();
        let expiration_date = create_date + 24*60*60*days_until_expiration;

        Poll {
            id: Uuid::new_v4(),
            question: question.to_string(),
            create_date,
            expiration_date,
            votes: Vec::new(),
            total_votes: 0,
        }
    }

    fn edit_poll(&mut self, new_question: &str) {
        self.question = new_question.to_string();
        //Maybe add edit to date?
    }

}

fn delete_poll(polls: &mut Vec<Poll>, poll_id: Uuid) -> bool {
    //Check if some poll.id is the same as poll_id. If it is equal it returns the index position then remove the poll from the polls list.
    if let Some(index) = polls.iter().position(|poll| poll.id == poll_id) {
        polls.remove(index);
        true
    } else {
        false
    }
}

fn csv_writer(file_name: &str) {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name);

    if file.is_err() {
        println!("Error reading file");
        return;
    }

    let file = file.unwrap();
    let mut writer = WriterBuilder::new().from_writer(file);

    let _ = writer.write_record(&["teste", "teste"]);
}

fn csv_reader(file_name: &str){
    let reader = Reader::from_path(file_name);

    if reader.is_err() {
        println!("Error reading file");
        return;
    }

    let mut file = reader.unwrap();

    for record in file.records() {
        let record = record.unwrap();
        
        println!("{}", record.get(0).unwrap());
    }
}