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
    //votes: Vec<Vote>,
    total_votes: i16,
}

impl Vote {
    fn vote_in_pool (conn: &Connection, poll_id: Uuid) -> Result<()> {
        let mut vote = String::new();
        let mut answer = String::new();
        let mut comment = String::new();
    
        println!("You vote? (y/n)");    
        println!("Digit 'y' for yes and 'n' for no");

        io::stdin()
            .read_line(&mut vote)
            .expect("Error");
    
        println!("You want to add a comment? (y/n)");
        io::stdin()
            .read_line(&mut answer)
            .expect("Error");
    
        if answer.trim() == "y" {
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
            poll_id: poll_id,
        };

        println!("{:?}", vote);

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

    fn choose_pool_to_vote(conn: &Connection) -> Result<()> {
        println!("Hello!");
        println!("Choose one of the following pools:");
    
        let mut stmt = conn.prepare("SELECT id, question, create_date, expiration_date, total_votes FROM Poll")?;
        let poll_iter = stmt.query_map([], |row| {
            Ok(Poll {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
                question: row.get(1)?,
                create_date: row.get(2)?,
                expiration_date: row.get(3)?,
                total_votes: row.get(4)?,
            })
        })?;
        let mut polls = Vec::new();
    
        for poll in poll_iter {
            polls.push(poll?);
        }

    
        for (i, poll) in polls.iter().enumerate() {
            println!("{}. {}", i + 1, poll.question);
        }
    
        let mut choice = String::new();
    
        loop{
            io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        
            let choice: usize = match choice.trim().parse() {
                Ok(num) if num > 0 && num <= polls.len() => num,
                _ => {
                    println!("Invalid input. Please enter a number.");
                    choice.clear();
                    continue;
                }
            };
            break;
        }
        let choice: usize = choice.trim().parse().unwrap();

        let poll_id = polls[choice - 1].id;
    
        Vote::vote_in_pool(conn, poll_id);
        Ok(())
    }
}


impl Poll {
    // Receive the question and the duration in days
    fn create_poll(conn: &Connection) -> Result<()>  {
        let mut question = String::new();
        let mut input_days = String::new();
        
        loop{
            println!("Pool Question: ");
            io::stdin()
                .read_line(&mut question)
                .expect("Failed to read Pool Question");

            if question.trim().chars().count() > 0 {
                if question.chars().count() <= 150{
                    break;
                } else{
                    println!("Question is too long. Question only can have up to 150 chars.");
                    question.clear()
                }
            } else{
                println!("Question can't be empty");
                question.clear()
            }
        }

        loop{
            println!("7 days or 30 days until expiration?");
            io::stdin()
                .read_line(&mut input_days)
                .expect("Failed to read days");
            
                let input_days: u32 = match input_days.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Invalid input. Please enter a number.");
                        input_days.clear();
                        continue;
                    }
                };

                if input_days == 7 || input_days == 30 {
                    break;
                } else{
                    println!("Only can be 7 days or 30 days.");
                }
        }

        let days_until_expiration: i64 = input_days.trim().parse().unwrap();

        let create_date = Local::now().timestamp();
        let expiration_date = create_date + 24*60*60*days_until_expiration;
        let pool = Poll {
            id: Uuid::new_v4(),
            question: question.trim().to_string(),
            create_date,
            expiration_date,
            total_votes: 0,
        };

        conn.execute(
            "INSERT INTO Poll (id, question, create_date, expiration_date, total_votes) VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                &pool.id.to_string(),
                &pool.question,
                &pool.create_date.to_string(),
                &pool.expiration_date.to_string(),
                &pool.total_votes.to_string(),
            ],
        )?;

        println!("Poll Created!");
        Ok(())
    }

    fn edit_poll(conn: &Connection) -> Result<()>  {
        let mut choice = String::new();
        let mut new_question = String::new();
        
        let mut stmt = conn.prepare("SELECT id, question, create_date, expiration_date, total_votes FROM Poll")?;
        let poll_iter = stmt.query_map([], |row| {
            Ok(Poll {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
                question: row.get(1)?,
                create_date: row.get(2)?,
                expiration_date: row.get(3)?,
                total_votes: row.get(4)?,
            })
        })?;
        let mut polls = Vec::new();
    
        for poll in poll_iter {
            polls.push(poll?);
        }

        
        loop{
            println!("Chose one poll to edit:");

            for (i, poll) in polls.iter().enumerate() {
                println!("{} - {}", i + 1, poll.question);
            }
            
            
            io::stdin().read_line(&mut choice).expect("Failed to read the choice");

            let choice: usize = match choice.trim().parse() {
                Ok(num) if num > 0 && num <= polls.len() => num,
                _ => {
                    println!("Invalid input. Please enter a valid number.");
                    choice.clear();
                    continue;
                }
            };
            break;
        }

        loop{
            println!("Pool Question: ");
            io::stdin()
                .read_line(&mut new_question)
                .expect("Failed to read Pool Question");

            if new_question.trim().chars().count() > 0 {
                if new_question.chars().count() <= 150{
                    break;
                } else{
                    println!("Question is too long. Question only can have up to 150 chars.");
                    new_question.clear()
                }
            } else{
                println!("Question can't be empty");
                new_question.clear()
            }
        }

        let choice: usize = choice.trim().parse().unwrap();

        let selected_poll = &polls[choice - 1];
        conn.execute(
            "UPDATE Poll SET question = ?1 WHERE id = ?2",
            &[&new_question.trim(), &selected_poll.id.to_string().as_str()],
        )?;
        println!("Poll {} edited Successfully", choice);
        Ok(())
    }

    fn delete_poll(conn: &Connection) -> Result<()> {
        let mut choice = String::new();
        let mut confirmation = String::new();

        let mut stmt = conn.prepare("SELECT id, question, create_date, expiration_date, total_votes FROM Poll")?;
        let poll_iter = stmt.query_map([], |row| {
            Ok(Poll {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
                question: row.get(1)?,
                create_date: row.get(2)?,
                expiration_date: row.get(3)?,
                total_votes: row.get(4)?,
            })
        })?;
        let mut polls = Vec::new();
    
        for poll in poll_iter {
            polls.push(poll?);
        }

        loop{
            println!("Chose one poll to edit:");

            for (i, poll) in polls.iter().enumerate() {
                println!("{} - {}", i + 1, poll.question);
            }
            
            
            io::stdin().read_line(&mut choice).expect("Failed to read the choice");

            let choice: usize = match choice.trim().parse() {
                Ok(num) if num > 0 && num <= polls.len() => num,
                _ => {
                    println!("Invalid input. Please enter a valid number.");
                    choice.clear();
                    continue;
                }
            };
            break;
        }
        let choice: usize = choice.trim().parse().unwrap();

        println!("Are you sure you want to delete the poll: {}? (y/n)", polls[choice - 1].question );
        io::stdin()
            .read_line(&mut confirmation)
            .expect("Error");

        if confirmation.trim() == "y" {
            //Check if some poll.id is the same as poll_id. If it is equal it returns the index position then remove the poll from the polls list.
            if let Some(index) = polls.iter().position(|poll| poll.id == polls[choice - 1].id) {
                conn.execute(
                    "DELETE FROM Poll WHERE id = ?1",
                    &[polls[choice - 1].id.to_string().as_str()],
                )?;
                println!("Poll Removed Successfuly!");
            } else {
                panic!("Error When Deleting the Poll")
            }
        } else{
            println!("Canceling operation")
        }
        Ok(())
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
             poll_id TEXT NOT NULL REFERENCES Poll(id)
         )",
        (),
    )?;

    Ok(())
}
fn main() -> Result<()> {
    let conn = Connection::open("database.db")?;
    create_tables(&conn)?;

    //Poll::create_poll(&conn);
    
    //Poll::edit_poll(&conn);

    //Poll::delete_poll(&conn);

    Vote::choose_pool_to_vote(&conn);

    let mut stmt = conn.prepare("SELECT id, question, create_date, expiration_date, total_votes FROM Poll")?;
    let poll_iter = stmt.query_map([], |row| {
        Ok(Poll {
            id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
            question: row.get(1)?,
            create_date: row.get(2)?,
            expiration_date: row.get(3)?,
            total_votes: row.get(4)?,
        })
    })?;
    let mut polls = Vec::new();

    for poll in poll_iter {
        polls.push(poll?);
    }

    //println!("{:?}", polls);
    
    Ok(())
}