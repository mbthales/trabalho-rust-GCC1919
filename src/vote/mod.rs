
use rusqlite::{Connection, Result};
use chrono::Local;
use uuid::Uuid;

#[derive(Debug)]
pub enum VoteChoice {
    Yes,
    No
}

//Create only to add to the poll struct
#[derive(Debug)]
pub struct Vote {
    pub id: Uuid,
    pub choice:  VoteChoice,
    pub comment: String,
    pub voting_power: i16,
    pub create_date: i64,
    pub poll_id: Uuid,
}

#[derive(Debug)]
pub struct VotePoll {
    pub id: Uuid,
    pub choice: String,
    pub question: String,
    pub poll_id: Uuid,
}

pub fn get_votes(conn: &Connection) -> Result<Vec<VotePoll>>{
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

pub fn create_vote (conn: &Connection, poll_id: Uuid, vote: &String, comment: String) -> Result<()> {
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

    println!("\nYour vote was registered successfully!");

    Ok(())
}

pub fn edit_vote(conn: &Connection, current_vote: &VotePoll, selected_vote: &VotePoll, new_choice: String, new_comment: String) -> Result<()> {
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

    Ok(())
}

pub fn delete_vote(conn: &Connection, selected_vote: &VotePoll) -> Result<()> {
    conn.execute(
        "DELETE FROM Vote WHERE id = ?1",
        &[selected_vote.id.to_string().as_str()],
    )?;

    if selected_vote.choice.trim() == "y" {
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
    
    Ok(())
}   

