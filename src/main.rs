use rusqlite::{Connection, Result};
use std::io;
use chrono::{Local,TimeZone};
use uuid::Uuid;

mod vote;
mod poll;
mod tests;

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
        println!("2 - Vote on a Poll");
        println!("3 - Edit a Poll");
        println!("4 - Edit a Vote");
        println!("5 - Delete a Poll");
        println!("6 - Delete a Vote");
        println!("7 - View Results");
        println!("8 - View Votes");
        println!("9 - Exit");

        let mut answer = String::new();

        io::stdin()
            .read_line(&mut answer)
            .expect("Error");

        let answer = answer.trim();

        if answer == "1" {
            let mut question = String::new();
            let mut input_days = String::new();
            let poll_duration;

            loop{
                println!("\nWrite your question below:");
                io::stdin()
                    .read_line(&mut question)
                    .expect("\nFailed to read poll Question");
    
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


            loop {
    
                println!("\n7 days or 30 days until expiration?");
                io::stdin()
                    .read_line(&mut input_days)
                    .expect("Failed to read days");

                let input_days_trimmed = input_days.trim();
                
                poll_duration = match input_days_trimmed.parse::<i8>() {
                    Ok(7) => {
                        Some(poll::PollDuration::OneWeek)
                    },
                    Ok(30) => {
                        Some(poll::PollDuration::OneMonth) 
                    }
                    _ => {
                        println!("\nInvalid input. Please enter 7 or 30 Days.");
                        input_days.clear();
                        continue;
                    }
                };   
                break;
            }      

            let _ = poll::create_poll(conn, question.to_string(), poll_duration.expect("to be 7 or 30").to_string());
            let _ = menu(conn);
            break;
        } else if answer == "2" {
            let polls = poll::get_polls(conn)?;

            let mut choice = String::new();
            let mut vote = String::new();
            let mut answer = String::new();
            let mut comment = String::new();

            if polls.is_empty() {
                println!("\nThere are no polls to vote.");
                let _ = menu(conn);
                return Ok(());
            }
    
            println!("\nChoose one of the following polls:");
        
            for (i, poll) in polls.iter().enumerate() {
                println!("{}. {}", i + 1, poll.question);
            }
        
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
    
            let poll = &polls[choice - 1];

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

            let _ = vote::create_vote(conn, poll.clone(), &vote, comment);

            let _ = menu(conn);

            break;
        } else if answer == "3" {
            let mut new_question = String::new();
            let mut  input_days = String::new();
            let _create_date: i64;
            let _expiration_date: i64;
            let mut choice1 = String::new();
            let mut choice2 = String::new();

            let mut stmt = conn.prepare("SELECT id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes FROM Poll")?;
            let poll_iter = stmt.query_map([], |row| {
                Ok(poll::Poll {
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
    
            if polls.is_empty() {
                println!("\nThere are no polls to Edit.");
                let _ = menu(conn);
                return Ok(());
            }
            
            loop{
                println!("\nChose one poll to edit:");
    
                for (i, poll) in polls.iter().enumerate() {
                    println!("{} - {}", i + 1, poll.question);
                }
                
                
                io::stdin().read_line(&mut choice1).expect("Failed to read the choice");
    
                let _choice1: usize = match choice1.trim().parse() {
                    Ok(num) if num > 0 && num <= polls.len() => num,
                    _ => {
                        println!("\nInvalid input. Please enter a valid number.");
                        choice1.clear();
                        continue;
                    }
                };
                break;
            }

            let choice1: usize = choice1.trim().parse().unwrap();

            let _selected_poll = &polls[choice1 - 1];

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
                println!("\nDo You want to set a new poll duration? (y/n)");
                io::stdin()
                    .read_line(&mut choice2)
                    .expect("Error");
    
                if choice2.trim() == "n" {
                    break;
                } else if choice2.trim() == "y"{ 

                    loop {
    
                        println!("\n7 days or 30 days until expiration?");
                        io::stdin()
                            .read_line(&mut input_days)
                            .expect("Failed to read days");
    
                        let input_days_trimmed = input_days.trim();
                        
                        match input_days_trimmed.parse::<i8>() {
                            Ok(7) => {
                                Some(poll::PollDuration::OneWeek)
                            },
                            Ok(30) => {
                                Some(poll::PollDuration::OneMonth) 
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
                    choice2.clear();
                }
            }
            let _ = poll::edit_poll(conn, choice1.to_string(), choice2.to_string(), new_question.to_string(), input_days.to_string());
            let _ = menu(conn);
            break;
        } else if answer == "4" {
            let votes = vote::get_votes(conn)?;

            if votes.is_empty() {
                println!("\nThere are no votes to edit.");
                let _ = menu(conn);
                return Ok(());
            }
    
            println!("\nChoose one of the following votes to edit:");
    
            for (i, vote) in votes.iter().enumerate() {
                println!("{}. Vote: {} | Question: {} | Date: {}", i + 1, vote.choice, vote.poll_question, Local.timestamp_opt(vote.create_date, 0).unwrap().format("%d-%m-%Y %H:%M:%S"));
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

            let _ = vote::edit_vote(conn, current_vote, selected_vote, new_choice, new_comment);

            let _ = menu(conn);

            break;
        } else if answer == "5" {
            let mut choice = String::new();
            let mut confirmation = String::new();
    
            let mut stmt = conn.prepare("SELECT id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes FROM Poll")?;
            let poll_iter = stmt.query_map([], |row| {
                Ok(poll::Poll {
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
            
            if polls.is_empty() {
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
                    let _ = poll::delete_poll(conn, choice.to_string(), confirmation.to_string());
                } else {
                    panic!("Error When Deleting the Poll")
                }
            } else{
                println!("\nCanceling operation")
            }

            let _ = menu(conn);
            break;
        } else if answer == "6" {
            let votes = vote::get_votes(conn)?;

            if votes.is_empty() {
                println!("\nThere are no votes to show.");
                let _ = menu(conn);
                break;
            }

            println!("\nChoose one of the following votes to delete:");

            for (i, vote) in votes.iter().enumerate() {
                println!("{}. Vote: {} | Question: {} | Date: {}", i + 1, vote.choice, vote.poll_question, Local.timestamp_opt(vote.create_date, 0).unwrap().format("%d-%m-%Y %H:%M:%S"));
            }

            let mut choice = String::new();
            let mut confirmation = String::new();
                
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
    
            println!("\nAre you sure you want to delete the vote: {} - '{}'? (y/n)", votes[choice - 1].choice, votes[choice - 1].poll_question);
    
            io::stdin()
                .read_line(&mut confirmation)
                .expect("Error");
    
            if confirmation.trim() == "y" {
                if let Some(_index) = votes.iter().position(|poll| poll.id == votes[choice - 1].id) {

                    vote::delete_vote(conn, selected_vote)?;
        
                } else {
                    panic!("Error when deleting the vote")
                }
            } else{
                println!("\nCanceling operation")
            }

            let _ = menu(conn);

            break;
        } else if answer == "7" {
            let polls = poll::get_polls(conn)?;

            if polls.is_empty() {
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

            break;
        } else if answer == "8" {
            let votes = vote::get_votes(conn)?;

            if votes.is_empty() {
                println!("\nThere are no votes to show.");
                let _ = menu(conn);
                break;
            }

            for vote in votes {
                
                let create_date = Local.timestamp_opt(vote.create_date, 0).unwrap();

                println!("\nQuestion: {} \nChoice: {} \nComment: {} \nCreate Date: {}", vote.poll_question, vote.choice, vote.comment, create_date.format("%d-%m-%Y %H:%M:%S"));
            }

            let _ = menu(conn);

            break;
        } 
        else if answer == "9" {
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