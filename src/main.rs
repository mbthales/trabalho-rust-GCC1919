use rusqlite::{Connection, Result, types::ToSqlOutput, ToSql, types::FromSqlError, types::ValueRef, types::FromSql};
use std::{io, fmt};
use chrono::{Local,TimeZone};
use uuid::Uuid;

#[derive(Debug)]
enum VoteChoice {
    Yes,
    No
}

//Create only to add to the poll struct
#[derive(Debug)]
struct Vote {
    id: Uuid,
    choice:  VoteChoice,
    comment: String,
    voting_power: i16,
    create_date: i64,
    poll_id: Uuid,
}

#[derive(Debug)]
struct VotePoll {
    id: Uuid,
    choice: String,
    question: String,
    poll_id: Uuid,
}
#[derive(Debug, PartialEq)]
struct Poll {
    id: Uuid, //Could have used a sequential one but I find it easier
    question: String,
    poll_duration: PollDuration,
    create_date: i64,
    expiration_date: i64,
    positive_votes: i16,
    negative_votes: i16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PollDuration {
    OneWeek = 7,
    OneMonth = 30,
}

impl FromSql for PollDuration {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_i64() {
            Ok(7) => Ok(PollDuration::OneWeek),
            Ok(30) => Ok(PollDuration::OneMonth),
            Ok(_) | Err(_) => Err(FromSqlError::Other("Invalid poll duration".into())),
        }
    }
}

impl ToSql for PollDuration {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as i8))
    }
}

impl fmt::Display for PollDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PollDuration::OneWeek => write!(f, "7"),
            PollDuration::OneMonth => write!(f, "30"),
        }
    }
}


impl Vote {
    fn get_poll_votes(conn: &Connection) -> Result<Vec<VotePoll>>{
        let mut stmt = conn.prepare("SELECT Vote.id as id, Vote.choice as choice, Poll.question as question, Poll.id as poll_id FROM Vote JOIN Poll ON Vote.poll_id = Poll.id")?;

        let vote_iter = stmt.query_map([], |row| {
            Ok(VotePoll {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
                choice: row.get(1)?,
                question: row.get(2)?,
                poll_id: Uuid::parse_str(row.get::<_, String>(3)?.as_str()).unwrap(),
            })
        })?;
    
        let mut votes = Vec::new();

        for vote in vote_iter {
            votes.push(vote?);
        }
    
        Ok(votes)
    }

    fn vote_in_poll (conn: &Connection, poll_id: Uuid) -> Result<()> {
        let mut vote = String::new();
        let mut answer = String::new();
        let mut comment = String::new();
    
        println!("\nYou vote? (y/n)");    
        println!("Digit 'y' for yes and 'n' for no");

        io::stdin()
            .read_line(&mut vote)
            .expect("Error");
    
        println!("\nYou want to add a comment? (y/n)");
        io::stdin()
            .read_line(&mut answer)
            .expect("Error");
    
        if answer.trim() == "y" {
            println!("\nWrite your comment:");
            io::stdin()
                .read_line(&mut comment)
                .expect("Error");
        }

        let vote = Vote {
            id: Uuid::new_v4(),
            choice: match vote.trim() {
                "y" => VoteChoice::Yes,
                "n" => VoteChoice::No,
                _ => panic!("Invalid Vote"),
            },
            comment: comment.trim().to_string(),
            voting_power: 1,
            create_date: Local::now().timestamp(),
            poll_id,
        };

        println!("\nYour vote was registered successfully!");

        match vote.choice {
            VoteChoice::Yes => {
                conn.execute(
                    "UPDATE Poll SET positive_votes = positive_votes + 1 WHERE id = ?1",
                    &[&vote.poll_id.to_string()],
                )?;
            }
            VoteChoice::No => {
                conn.execute(
                    "UPDATE Poll SET negative_votes = negative_votes + 1 WHERE id = ?1",
                    &[&vote.poll_id.to_string()],
                )?;
            }
        }

        let choice = match &vote.choice {
            VoteChoice::Yes => "y".to_string(),
            VoteChoice::No => "n".to_string(),
        };

        conn.execute(
            "INSERT INTO Vote (id, choice, comment, voting_power, create_date, poll_id) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &vote.id.to_string(),
                &choice,
                &vote.comment,
                &vote.voting_power.to_string(),
                &vote.create_date.to_string(),
                &vote.poll_id.to_string(),
            ]
        )?;

        Ok(())
    }

    fn choose_poll_to_vote(conn: &Connection) -> Result<()> {
        let polls = Poll::get_polls(conn)?;

        if polls.len() == 0 {
            println!("\nThere are no polls to vote.");
            let _ = menu(conn);
            return Ok(());
        }

        println!("\nChoose one of the following polls:");
    
    
        for (i, poll) in polls.iter().enumerate() {
            println!("{}. {}", i + 1, poll.question);
        }
    
        let mut choice = String::new();
    
        loop{
            io::stdin()
            .read_line(&mut choice)
            .expect("\nFailed to read line");
        
            let _: usize = match choice.trim().parse() {
                Ok(num) if num > 0 && num <= polls.len() => num,
                _ => {
                    println!("\nInvalid input. Please enter a number.");
                    choice.clear();
                    continue;
                }
            };
            break;
        }
        let choice: usize = choice.trim().parse().unwrap();

        let poll_id = polls[choice - 1].id;
    
        let _ = Vote::vote_in_poll(conn, poll_id);

        menu(conn)?;
        
        Ok(())
    }

    fn edit_vote(conn: &Connection) -> Result<()> {
        let votes = Vote::get_poll_votes(conn)?;

        if votes.len() == 0 {
            println!("\nThere are no votes to edit.");
            let _ = menu(conn);
            return Ok(());
        }

        println!("\nChoose one of the following votes to edit:");

        for (i, vote) in votes.iter().enumerate() {
            println!("{}. {} - {}", i + 1, vote.choice, vote.question);
        }

        let mut choice = String::new();

        loop{
            io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        
            let _: usize = match choice.trim().parse() {
                Ok(num) if num > 0 && num <= votes.len() => num,
                _ => {
                    println!("\nInvalid input. Please enter a number.");
                    choice.clear();
                    continue;
                }
            };
            break;
        }

        let choice: usize = choice.trim().parse().unwrap();

        let selected_vote = &votes[choice - 1];

        let mut new_choice = String::new();
        let mut new_comment = String::new();

        loop{
            println!("\nYou vote? (y/n)");    
            println!("Digit 'y' for yes and 'n' for no");

            io::stdin()
                .read_line(&mut new_choice)
                .expect("Error");
        
            println!("\nYou want to add a comment? (y/n)");
            io::stdin()
                .read_line(&mut new_comment)
                .expect("Error");
        
            if new_comment.trim() == "y" {
                println!("\nWrite your comment:");
                io::stdin()
                    .read_line(&mut new_comment)
                    .expect("Error");
            }

            if new_choice.trim() == "y" || new_choice.trim() == "n" {
                break;
            } else{
                println!("\nInvalid input. Please enter 'y' or 'n'.");
            }
        }

        let current_vote = &votes[choice - 1];

        if current_vote.choice.trim() == "y" && new_choice.trim() == "n" {
            conn.execute(
                "UPDATE Poll SET positive_votes = positive_votes - 1 WHERE id = ?1",
                &[
                    &selected_vote.poll_id.to_string(),
                ],
            )?;

            conn.execute(
                "UPDATE Poll SET negative_votes = negative_votes + 1 WHERE id = ?1",
                &[
                    &selected_vote.poll_id.to_string(),
                ],
            )?;
        } else if current_vote.choice.trim() == "n" && new_choice.trim() == "y" {
            conn.execute(
                "UPDATE Poll SET negative_votes = negative_votes - 1 WHERE id = ?1",
                &[
                    &selected_vote.poll_id.to_string(),
                ],
            )?;

            conn.execute(
                "UPDATE Poll SET positive_votes = positive_votes + 1 WHERE id = ?1",
                &[
                    &selected_vote.poll_id.to_string(),
                ],
            )?;
        }

        conn.execute(
            "UPDATE Vote SET choice = ?1, comment = ?2 WHERE id = ?3",
            [new_choice.trim(), new_comment.trim(), selected_vote.id.to_string().as_str()],
        )?;

        println!("\nYour vote was edited successfully!");

        let _ = menu(conn);

        Ok(())
    }

    fn delete_poll_vote(conn: &Connection) -> Result<()> {
        let votes = Vote::get_poll_votes(conn)?;
        let mut confirmation = String::new();

        if votes.len() == 0 {
            println!("\nThere are no votes to delete.");
            let _ = menu(conn);
            return Ok(());
        }

        println!("\nChoose one of the following votes to delete:");

        for (i, vote) in votes.iter().enumerate() {
            println!("{}. {} - {}", i + 1, vote.choice, vote.question);
        }

        let mut choice = String::new();

        loop{
            io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        
            let _: usize = match choice.trim().parse() {
                Ok(num) if num > 0 && num <= votes.len() => num,
                _ => {
                    println!("\nInvalid input. Please enter a number.");
                    choice.clear();
                    continue;
                }
            };
            break;
        }

        let choice: usize = choice.trim().parse().unwrap();

        let selected_vote = &votes[choice - 1];

        println!("\nAre you sure you want to delete the vote: {} - '{}'? (y/n)", votes[choice - 1].choice, votes[choice - 1].question);

        io::stdin()
            .read_line(&mut confirmation)
            .expect("Error");

        if confirmation.trim() == "y" {
            if let Some(_index) = votes.iter().position(|poll| poll.id == votes[choice - 1].id) {
                conn.execute(
                    "DELETE FROM Poll WHERE id = ?1",
                    &[votes[choice - 1].id.to_string().as_str()],
                )?;

                conn.execute(
                    "DELETE FROM Vote WHERE id = ?1",
                    &[selected_vote.id.to_string().as_str()],
                )?;
        
                if selected_vote.choice == "y" {
                    conn.execute(
                        "UPDATE Poll SET positive_votes = positive_votes - 1 WHERE id = ?1",
                        &[selected_vote.poll_id.to_string().as_str()],
                    )?;
                } else {
                    conn.execute(
                        "UPDATE Poll SET negative_votes = negative_votes - 1 WHERE id = ?1",
                        &[selected_vote.poll_id.to_string().as_str()],
                    )?;
                }
                println!("\nYour vote was removed successfully!");
        
            } else {
                panic!("Error When Deleting the Poll")
            }
        } else{
            println!("\nCanceling operation")
        }

        let _ = menu(conn);

        Ok(())
    }   
}


impl Poll {
    fn get_polls(conn: &Connection) -> Result<Vec<Poll>> {
        let mut stmt = conn.prepare("SELECT id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes FROM Poll")?;
        let poll_iter = stmt.query_map([], |row| {
            Ok(Poll {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
                question: row.get(1)?,
                poll_duration: row.get(2)?,
                create_date: row.get(3)?,
                expiration_date: row.get(4)?,
                positive_votes: row.get(5)?,
                negative_votes: row.get(6)?,
            })
        })?;
        let mut polls = Vec::new();
    
        for poll in poll_iter {
            polls.push(poll?);
        }
    
        Ok(polls)
    }

    // Receive the question and the duration in days
    fn create_poll(conn: &Connection) -> Result<()>  {
        let mut question = String::new();
        let mut input_days = String::new();
        let poll_duration: Option<PollDuration>;
        let create_date: i64;
        let expiration_date: i64;
        
        loop{
            println!("\nWrite your question below: ");
            io::stdin()
                .read_line(&mut question)
                .expect("Failed to read poll Question");

            if question.trim().chars().count() > 0 {
                if question.chars().count() <= 150{
                    break;
                } else{
                    println!("\nQuestion is too long. Question only can have up to 150 chars.");
                    question.clear()
                }
            } else{
                println!("\nQuestion can't be empty");
                question.clear()
            }
        }

        loop{
            println!("\n7 days or 30 days until expiration?");
            io::stdin()
                .read_line(&mut input_days)
                .expect("Failed to read days");
            
            let input_days_trimmed = input_days.trim();
            
            poll_duration = match input_days_trimmed.parse::<i8>() {
                Ok(7) =>  {
                    create_date = Local::now().timestamp();
                    expiration_date = create_date + 24*60*60*7;
                    Some(PollDuration::OneWeek)
                },
                Ok(30) =>  {
                    create_date = Local::now().timestamp();
                    expiration_date = create_date + 24*60*60*30;
                    Some(PollDuration::OneMonth)
                }
                _ => {
                    println!("\nInvalid input. Please enter 7 or 30.");
                    input_days.clear();
                    continue;
                }
            };
            break;
        }

        let poll = Poll {
            id: Uuid::new_v4(),
            question: question.trim().to_string(),
            poll_duration: poll_duration.expect("Poll duration can't be empty"),
            create_date,
            expiration_date,
            positive_votes: 0,
            negative_votes: 0,
        };

        conn.execute(
            "INSERT INTO Poll (id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &poll.id.to_string(),
                &poll.question,
                &poll.poll_duration.to_string(),
                &poll.create_date.to_string(),
                &poll.expiration_date.to_string(),
                &poll.positive_votes.to_string(),
                &poll.negative_votes.to_string(),
            ],
        )?;

        println!("\nPoll Created!");

        let _ = menu(conn);

        Ok(())
    }

    fn edit_poll(conn: &Connection) -> Result<()>  {
        let mut choice = String::new();
        let mut input_days = String::new();
        let mut new_question = String::new();
        let poll_duration: Option<PollDuration>;
        let create_date;
        let expiration_date;
        
        let mut stmt = conn.prepare("SELECT id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes FROM Poll")?;
        let poll_iter = stmt.query_map([], |row| {
            Ok(Poll {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
                question: row.get(1)?,
                poll_duration: row.get(2)?,
                create_date: row.get(3)?,
                expiration_date: row.get(4)?,
                positive_votes: row.get(5)?,
                negative_votes: row.get(6)?,
            })
        })?;
        let mut polls = Vec::new();
    
        for poll in poll_iter {
            polls.push(poll?);
        }

        if polls.len() == 0 {
            println!("\nThere are no polls to Edit.");
            let _ = menu(conn);
            return Ok(());
        }

        
        loop{
            println!("\nChose one poll to edit:");

            for (i, poll) in polls.iter().enumerate() {
                println!("{} - {}", i + 1, poll.question);
            }
            
            
            io::stdin().read_line(&mut choice).expect("Failed to read the choice");

            let _choice: usize = match choice.trim().parse() {
                Ok(num) if num > 0 && num <= polls.len() => num,
                _ => {
                    println!("\nInvalid input. Please enter a valid number.");
                    choice.clear();
                    continue;
                }
            };
            break;
        }

        let choice: usize = choice.trim().parse().unwrap();

        let selected_poll = &polls[choice - 1];

        loop{
            println!("\nWrite your question below:");
            io::stdin()
                .read_line(&mut new_question)
                .expect("\nFailed to read poll Question");

            if new_question.trim().chars().count() > 0 {
                if new_question.chars().count() <= 150{
                    break;
                } else{
                    println!("\nQuestion is too long. Question only can have up to 150 chars.");
                    new_question.clear()
                }
            } else{
                println!("\nQuestion can't be empty");
                new_question.clear()
            }
        }

        loop{
            let mut new_choice = String::new();
        
            println!("\nDo You want to set a new poll duration? (y/n)");
            io::stdin()
                .read_line(&mut new_choice)
                .expect("Error");

            if new_choice.trim() == "n" {
                poll_duration = Some(selected_poll.poll_duration);
                create_date = selected_poll.create_date;
                expiration_date = selected_poll.expiration_date;
                break;
            } else if new_choice.trim() == "y"{ 
                loop {

                    println!("\n7 days or 30 days until expiration?");
                    io::stdin()
                        .read_line(&mut input_days)
                        .expect("Failed to read days");

                    let input_days_trimmed = input_days.trim();
                    
                    poll_duration = match input_days_trimmed.parse::<i8>() {
                        Ok(7) => {
                            create_date = Local::now().timestamp();
                            expiration_date = create_date + 24*60*60*7;
                            Some(PollDuration::OneWeek)
                        },
                        Ok(30) => {
                            create_date = Local::now().timestamp();
                            expiration_date = create_date + 24*60*60*30;
                            Some(PollDuration::OneMonth) 
                        }
                        _ => {
                            println!("\nInvalid input. Please enter 7 or 30 Days.");
                            input_days.clear();
                            continue;
                        }
                    };   
                    break; 
                }
                break;
            } 
            else{
                println!("\nInvalid input. Please enter 'y' or 'n'.");
            }
        }

        let poll = Poll {
            id: selected_poll.id,
            question: new_question.trim().to_string(),
            poll_duration: poll_duration.expect("Poll duration can't be empty"),
            create_date,
            expiration_date,
            positive_votes: 0,
            negative_votes: 0,
        };


        conn.execute(
            "UPDATE Poll SET question = ?1, poll_duration = ?2, create_date = ?3, expiration_date = ?4  WHERE id = ?5",
            [
                &poll.question,
                &poll.poll_duration.to_string(),
                &poll.create_date.to_string(),
                &poll.expiration_date.to_string(),
                &poll.id.to_string()
            ],
        )?;

        println!("\nPoll {} edited Successfully", choice);

        let _ = menu(conn);

        Ok(())
    }

    fn delete_poll(conn: &Connection) -> Result<()> {
        let mut choice = String::new();
        let mut confirmation = String::new();

        let mut stmt = conn.prepare("SELECT id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes FROM Poll")?;
        let poll_iter = stmt.query_map([], |row| {
            Ok(Poll {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
                question: row.get(1)?,
                poll_duration: row.get(2)?,
                create_date: row.get(3)?,
                expiration_date: row.get(4)?,
                positive_votes: row.get(5)?,
                negative_votes: row.get(6)?,
            })
        })?;
        let mut polls = Vec::new();
    
        for poll in poll_iter {
            polls.push(poll?);
        }
        
        if polls.len() == 0 {
            println!("\nThere are no polls to Delete.");
            let _ = menu(conn);
            return Ok(());
        }

        loop{
            println!("\nChose one poll to delete:");

            for (i, poll) in polls.iter().enumerate() {
                println!("{} - {}", i + 1, poll.question);
            }
            
            
            io::stdin().read_line(&mut choice).expect("Failed to read the choice");

            let _choice: usize = match choice.trim().parse() {
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

        println!("\nAre you sure you want to delete the poll: '{}'? (y/n)", polls[choice - 1].question );
        io::stdin()
            .read_line(&mut confirmation)
            .expect("Error");

        if confirmation.trim() == "y" {
            //Check if some poll.id is the same as poll_id. If it is equal it returns the index position then remove the poll from the polls list.
            if let Some(_index) = polls.iter().position(|poll| poll.id == polls[choice - 1].id) {
                conn.execute(
                    "DELETE FROM Vote WHERE poll_id = ?1",
                    [polls[choice - 1].id.to_string().as_str()],
                )?;

                conn.execute(
                    "DELETE FROM Poll WHERE id = ?1",
                    [polls[choice - 1].id.to_string().as_str()],
                )?;
                println!("\nPoll Removed Successfuly!");
            } else {
                panic!("Error When Deleting the Poll")
            }
        } else{
            println!("\nCanceling operation")
        }

        let _ = menu(conn);

        Ok(())
    }
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS Poll (
             id TEXT PRIMARY KEY,
             question TEXT NOT NULL,
             poll_duration INTEGER NOT NULL,
             create_date DATE NOT NULL,
             expiration_date DATE NOT NULL,
             positive_votes INTEGER NOT NULL,
             negative_votes INTEGER NOT NULL
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

fn menu (conn: &Connection) -> Result<()>{
    loop {
        println!("\nWhat do you want to do?");
        println!("1 - Create a Poll");
        println!("2 - Vote in a Poll");
        println!("3 - Edit a Poll");
        println!("4 - Edit a Vote");
        println!("5 - Delete a Poll");
        println!("6 - Delete a Vote");
        println!("7 - View Results");
        println!("8 - Exit");

        let mut answer = String::new();

        io::stdin()
            .read_line(&mut answer)
            .expect("Error");

        let answer = answer.trim();

        if answer == "1" {
            let _ = Poll::create_poll(conn);
            break;
        } else if answer == "2" {
            let _ = Vote::choose_poll_to_vote(conn);
            break;
        } else if answer == "3" {
            let _ = Poll::edit_poll(conn);
            break;
        } else if answer == "4" {
            let _ = Vote::edit_vote(conn);
            break;
        } else if answer == "5" {
            let _ = Poll::delete_poll(conn);
            break;
        } else if answer == "6" {
            let _ = Vote::delete_poll_vote(conn);
            break;
        } else if answer == "7" {
            let polls = Poll::get_polls(conn)?;

            if polls.len() == 0 {
                println!("\nThere are no polls to show.");
                let _ = menu(conn);
                break;
            }

            for poll in polls {
                
                let create_date = Local.timestamp_opt(poll.create_date, 0).unwrap();
                let expiration_date = Local.timestamp_opt(poll.expiration_date, 0).unwrap();

                println!("\nQuestion: {} \nPositive Votes: {}\nNegative Votes: {} \nCreate Date: {}\nExpiration Date: {} \nTotal Poll Duration: {} Days",
                poll.question, 
                poll.positive_votes,
                poll.negative_votes,
                create_date.format("%d-%m-%Y %H:%M:%S"),
                expiration_date.format("%d-%m-%Y %H:%M:%S"),
                poll.poll_duration);
            }

            let _ = menu(conn);
        } else if answer == "8" {
            println!("\nExiting...");
            break;
        } else {
            println!("\nInvalid input, please try again.");
        }

    }
    Ok(())
}
fn main() -> Result<()> {
    let conn = Connection::open("database.db")?;
    create_tables(&conn)?;

    println!("Hello!");    

    let _ = menu(&conn);

    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, Result};
    use uuid::Uuid;
    use chrono::Local;

    use crate::{create_tables, Vote, PollDuration, Poll, VoteChoice};

    use std::fs;
    use std::path::Path;
    
    #[test]
    fn test_get_polls() -> Result<()> {
        println!("Starting Test");
        let conn = Connection::open_in_memory()?;
    
        create_tables(&conn)?;
    
        // Insert sample data
        let vote_id_2 = Uuid::new_v4().to_string();
    
        let now = Local::now().timestamp();

        let poll = Poll {
            id: Uuid::new_v4(),
            question: "teste question?".trim().to_string(),
            poll_duration: PollDuration::OneMonth,
            create_date: Local::now().timestamp(),
            expiration_date : Local::now().timestamp() + 24*60*60*30,
            positive_votes: 0,
            negative_votes: 0,
        };
    
        conn.execute(
            "INSERT INTO Poll (id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &poll.id.to_string(),
                &poll.question,
                &poll.poll_duration.to_string(),
                &poll.create_date.to_string(),
                &poll.expiration_date.to_string(),
                &poll.positive_votes.to_string(),
                &poll.negative_votes.to_string(),
            ],
        )?;

        println!("Insert okay");
        /*
        let vote1 = Vote{
            id: Uuid::new_v4(),
            choice:  VoteChoice::Yes,
            comment: "Comment about this poll".trim().to_string(),
            voting_power: 5,
            create_date: Local::now().timestamp(),
            poll_id: poll.id,
        };

        
        conn.execute(
            "INSERT INTO Vote (id, choice, comment, voting_power, create_date, poll_id) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &vote1.id.to_string(),
                &"y".trim().to_string(),
                &vote1.comment,
                &vote1.voting_power.to_string(),
                &vote1.create_date.to_string(),
                &vote1.poll_id.to_string(),
            ]
        )?;
        */

        // Obtemos o resultado da função get_polls
        let polls_output = Poll::get_polls(&conn);

        match polls_output {
            Ok(polls) => {
                let response = &polls[0];
                println!("{:?}", response);
                println!("{:?}", poll);
                assert_eq!(response, &poll)
            }
            Err(err) => {
                panic!("Failed to retrieve polls: {:?}", err);
            }
        }
        Ok(())
    }
}