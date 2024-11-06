use rusqlite::{Connection, Result};
use std::io;
use chrono::Local;
use uuid::Uuid;

//Create only to add to the Pool struct
#[derive(Debug)]
struct Vote {
    id: Uuid,
    choice: String,
    comment: String,
    voting_power: i16,
    create_date: i64,
    poll_id: Uuid,
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

impl Vote {
    fn vote_in_pool (conn: &Connection) -> Result<()> {
        let mut vote = String::new();
        let mut answer = String::new();
        let mut comment = String::new();
    
        println!("You vote? (y/n)");    
        println!("Digit 'y' for yes and 'n' for no");

        io::stdin()
            .read_line(&mut vote)
            .expect("Error");
    
        println!("You want to add a comment? (s/n)");
        io::stdin()
            .read_line(&mut answer)
            .expect("Error");
    
        if answer.trim() == "s" {
            println!("Write your comment:");
            io::stdin()
                .read_line(&mut comment)
                .expect("Error");
        }

        let vote = Vote {
            id: Uuid::new_v4(),
            choice: vote.trim().to_string(),
            comment: comment.trim().to_string(),
            voting_power: 1,
            create_date: Local::now().timestamp(),
            poll_id: Uuid::new_v4(),
        };

        conn.execute(
            "INSERT INTO Vote (id, choice, comment, voting_power, create_date, poll_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &vote.id.to_string(),
                &vote.choice,
                &vote.comment,
                &vote.voting_power.to_string(),
                &vote.create_date.to_string(),
                &vote.poll_id.to_string(),
            ],
        )?;
    
        Ok(())
    }

    fn choose_pool_to_vote(conn: &Connection) {
        println!("Hello!");
        println!("Choose one of the following pools:");
    
        let pools = vec![
            "Pool 1",
            "Pool 2",
            "Pool 3",
            "Pool 4",
            "Pool 5",
        ];
    
        for (i, question) in pools.iter().enumerate() {
            println!("{}. {}", i + 1, question);
        }
    
        let mut choose = String::new();
    
        io::stdin()
        .read_line(&mut choose)
        .expect("Failed to read line");
    
        let choose: usize = match choose.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input. Please enter a number.");
                return;
            }
        };
    
        Vote::vote_in_pool(conn);
    }
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
    
    fn delete_poll(polls: &mut Vec<Poll>, poll_id: Uuid) -> bool {
        //Check if some poll.id is the same as poll_id. If it is equal it returns the index position then remove the poll from the polls list.
        if let Some(index) = polls.iter().position(|poll| poll.id == poll_id) {
            polls.remove(index);
            true
        } else {
            false
        }
    }
}

fn create_tables(conn: &Connection) -> Result<()> {

    
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS Poll (
             id TEXT PRIMARY KEY,
             question TEXT NOT NULL,
             create_date DATE NOT NULL,
             expiration_date DATE NOT NULL,
             total_votes INTEGER NOT NULL
             )",
             (),
            )?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS Vote (
             id TEXT PRIMARY KEY,
             choice TEXT NOT NULL,
             comment TEXT,
             voting_power INTEGER NOT NULL,
             create_date DATE NOT NULL,
             pool_id TEXT NOT NULL REFERENCES Pool(id)
         )",
        (),
    )?;

    Ok(())
}
fn main() -> Result<()> {
    let conn = Connection::open("database.db")?;
    create_tables(&conn)?;
    Vote::choose_pool_to_vote(&conn);
    Ok(())
}