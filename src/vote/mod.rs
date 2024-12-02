
use rusqlite::{Connection, Result};
use chrono::Local;
use uuid::Uuid;
use std::fmt;
use std::error::Error;

use crate::poll::Poll;

#[derive(Debug, PartialEq, Clone)]
pub enum VoteChoice {
    Yes,
    No
}

#[derive(Debug, Clone)]
pub struct Vote {
    pub id: Uuid,
    pub choice:  VoteChoice,
    pub comment: String,
    pub voting_power: i16,
    pub create_date: i64,
    pub poll_id: Uuid,
    pub poll_question: String,
}
#[derive(Debug)]
pub struct ValidationError {
   pub details: String,
}

impl PartialEq for ValidationError {
    fn eq(&self, other: &Self) -> bool {
         self.details == other.details
    }
 }

impl ValidationError {
   pub fn new(msg: &str) -> ValidationError {
        ValidationError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ValidationError {}

impl fmt::Display for VoteChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         match self {
             VoteChoice::Yes => write!(f, "Yes"),
             VoteChoice::No => write!(f, "No"),
         }
     }
 }

pub fn get_votes(conn: &Connection) -> Result<Vec<Vote>>{
    let mut stmt = conn.prepare("SELECT Vote.id as id, choice, comment, voting_power, Vote.create_date as create_date, poll_id, question FROM Vote JOIN Poll ON Vote.poll_id = Poll.id")?;

    let vote_iter = stmt.query_map([], |row| {
        Ok(Vote {
            id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
            choice: match row.get::<_, String>(1)?.as_str().trim() {
                "y" => VoteChoice::Yes,
                "n" => VoteChoice::No,
                _ => panic!("Invalid Vote"),
            },
            comment: row.get(2)?,
            voting_power: row.get(3)?,
            create_date: row.get(4)?,
            poll_id: Uuid::parse_str(row.get::<_, String>(5)?.as_str()).unwrap(),
            poll_question: row.get(6)?,
        })
    })?;

    let mut votes = Vec::new();

    for vote in vote_iter {
        votes.push(vote?);
    }

    Ok(votes)
}

pub fn create_vote (conn: &Connection, poll: Poll, vote: &str, comment: String) -> Result<Vote, Box<dyn Error>>{
    let vote = Vote {
        id: Uuid::new_v4(),
        choice: match vote.trim() {
            "y" => VoteChoice::Yes,
            "n" => VoteChoice::No,
            _ => {
                println!("Invalid Vote");
                return Err(Box::new(ValidationError::new(
                    "Invalid Vote."
                )));
            }
        },
        comment: comment.trim().to_string(),
        voting_power: 1,
        create_date: Local::now().timestamp(),
        poll_id: poll.id,
        poll_question: poll.question,
    };

    if comment.len() > 100 {
        return Err(Box::new(ValidationError::new(
            "Comment is too long. Comment only can have up to 100 chars.",
         )));
    }

    let choice = match &vote.choice {
        VoteChoice::Yes => "y".to_string(),
        VoteChoice::No => "n".to_string(),
    };

    conn.execute(
        "INSERT INTO Vote (id, choice, comment, voting_power, create_date, poll_id) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            &vote.id.to_string(),
            &choice,
            &vote.comment,
            &vote.voting_power.to_string(),
            &vote.create_date.to_string(),
            &vote.poll_id.to_string(),
        ]
    )?;

    match vote.choice {
        VoteChoice::Yes => {
            conn.execute(
                "UPDATE Poll SET positive_votes = positive_votes + 1 WHERE id = ?1",
                [&vote.poll_id.to_string()],
            )?;
        }
        VoteChoice::No => {
            conn.execute(
                "UPDATE Poll SET negative_votes = negative_votes + 1 WHERE id = ?1",
                [&vote.poll_id.to_string()],
            )?;
        }
    }

    println!("\nYour vote was registered successfully!");

    Ok(vote)
}

pub fn edit_vote(
    conn: &Connection,
    current_vote: &Vote,
    selected_vote: &Vote,
    new_choice: String,
    new_comment: String
) -> Result<Vote, Box<dyn Error>> {
    if new_comment.len() > 100 {
        return Err(Box::new(ValidationError::new(
            "Comment is too long. Comment only can have up to 100 chars.",
         )));
    }

    if match  current_vote.choice {
        VoteChoice::Yes => "y",
        VoteChoice::No => "n",
    } == "y" && new_choice.trim() == "n" {
        conn.execute(
            "UPDATE Poll SET positive_votes = positive_votes - 1 WHERE id = ?1",
            [&selected_vote.poll_id.to_string()],
        )?;

        conn.execute(
            "UPDATE Poll SET negative_votes = negative_votes + 1 WHERE id = ?1",
            [&selected_vote.poll_id.to_string()],
        )?;
    } else if match  current_vote.choice {
        VoteChoice::Yes => "y",
        VoteChoice::No => "n",
    } == "n" && new_choice.trim() == "y" {
        conn.execute(
            "UPDATE Poll SET negative_votes = negative_votes - 1 WHERE id = ?1",
            [&selected_vote.poll_id.to_string()],
        )?;

        conn.execute(
            "UPDATE Poll SET positive_votes = positive_votes + 1 WHERE id = ?1",
            [&selected_vote.poll_id.to_string()],
        )?;
    }

    if new_comment.trim() == "" {
        conn.execute(
            "UPDATE Vote SET choice = ?1 WHERE id = ?2",
            [new_choice.trim(), selected_vote.id.to_string().as_str()],
        )?;
    } else {
        conn.execute(
            "UPDATE Vote SET choice = ?1, comment = ?2 WHERE id = ?3",
            [new_choice.trim(), new_comment.trim(), selected_vote.id.to_string().as_str()],
        )?;
    }

    println!("\nYour vote was edited successfully!");

    Ok(selected_vote.clone())
}

pub fn delete_vote(
    conn: &Connection,
    selected_vote: &Vote
) -> Result<Vote> {
    conn.execute(
        "DELETE FROM Vote WHERE id = ?1",
        [selected_vote.id.to_string().as_str()],
    )?;

    if match selected_vote.choice {
        VoteChoice::Yes => "y",
        VoteChoice::No => "n",
    } == "y" {
        conn.execute(
            "UPDATE Poll SET positive_votes = positive_votes - 1 WHERE id = ?1",
            [selected_vote.poll_id.to_string().as_str()],
        )?;
    } else {
        conn.execute(
            "UPDATE Poll SET negative_votes = negative_votes - 1 WHERE id = ?1",
            [selected_vote.poll_id.to_string().as_str()],
        )?;
    }

    println!("\nYour vote was removed successfully!");
    
    Ok(selected_vote.clone())
}