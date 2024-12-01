use rusqlite::{Connection, Result};
use std::io;
use chrono::{Local,TimeZone};

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
            let mut question = String::new();
            let mut input_days = String::new();
            let mut poll_duration = String::new();

            println!("\nWrite your question below: ");
            io::stdin()
                .read_line(&mut question)
                .expect("Failed to read poll Question");

            println!("\n7 days or 30 days until expiration?");
                io::stdin()
                    .read_line(&mut input_days)
                    .expect("Failed to read days");
                

            let _ = poll::create_poll(conn, question.to_string(), poll_duration.to_string());
            let _ = menu(conn);
            break;
        } else if answer == "2" {
            let polls = poll::get_polls(conn)?;
            let mut choice = String::new();
            let mut vote = String::new();
            let mut answer = String::new();
            let mut comment = String::new();

            if polls.len() == 0 {
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
    
            let poll_id = polls[choice - 1].id;

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

            let _ = vote::create_vote(conn, poll_id, &vote, comment);

            let _ = menu(conn);
            break;
        } else if answer == "3" {
            let _ = poll::edit_poll(conn);
            break;
        } else if answer == "4" {
            let votes = vote::get_votes(conn)?;

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

            let _ = vote::edit_vote(conn, current_vote, selected_vote, new_choice, new_comment);

            let _ = menu(conn);

            break;
        } else if answer == "5" {
            let _ = poll::delete_poll(conn);
            break;
        } else if answer == "6" {
            let votes = vote::get_votes(conn)?;

            if votes.len() == 0 {
                println!("\nThere are no votes to show.");
                let _ = menu(conn);
                break;
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
    
            println!("\nAre you sure you want to delete the vote: {} - '{}'? (y/n)", votes[choice - 1].choice, votes[choice - 1].question);
    
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