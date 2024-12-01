#[cfg(test)]
mod tests {
    use rusqlite::{Connection, Result};
    use uuid::Uuid;
    use chrono::Local;

    use crate::create_tables;
    use crate::poll;
    use crate::poll::Poll;
    use crate::poll::PollDuration;
    use crate::poll::ValidationError;

    #[test]
    fn test_get_polls() -> Result<()> {
        println!("Starting Test");
        let conn = Connection::open_in_memory()?;
    
        create_tables(&conn)?;
    
        let vote_id_2 = Uuid::new_v4().to_string();
    
        let now = Local::now().timestamp();

        let poll1 = Poll {
            id: Uuid::new_v4(),
            question: "Do you like Rust?".trim().to_string(),
            poll_duration: PollDuration::OneWeek,
            create_date: now,
            expiration_date : now + 24*60*60*7,
            positive_votes: 0,
            negative_votes: 0,
        };

        let poll2 = Poll {
            id: Uuid::new_v4(),
            question: "Do you like Python?".trim().to_string(),
            poll_duration: PollDuration::OneMonth,
            create_date: now,
            expiration_date : now + 24*60*60*30,
            positive_votes: 0,
            negative_votes: 0,
        };

        let expected_polls = vec![&poll1, &poll2];
    
        conn.execute(
            "INSERT INTO Poll (id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &poll1.id.to_string(),
                &poll1.question,
                &poll1.poll_duration.to_string(),
                &poll1.create_date.to_string(),
                &poll1.expiration_date.to_string(),
                &poll1.positive_votes.to_string(),
                &poll1.negative_votes.to_string(),
            ],
        )?;

        conn.execute(
            "INSERT INTO Poll (id, question, poll_duration, create_date, expiration_date, positive_votes, negative_votes ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &poll2.id.to_string(),
                &poll2.question,
                &poll2.poll_duration.to_string(),
                &poll2.create_date.to_string(),
                &poll2.expiration_date.to_string(),
                &poll2.positive_votes.to_string(),
                &poll2.negative_votes.to_string(),
            ],
        )?;

        println!("Insert okay");

        let polls_output = poll::get_polls(&conn);

        match polls_output {
            Ok(polls) => {
                for (i, response) in polls.iter().enumerate() {
                    assert_eq!(&response, &expected_polls[i], "Output was different than expected.");
                }
            }
            Err(err) => {
                panic!("Failed to retrieve polls: {:?}", err);
            }
        }
        Ok(())
    }

    #[test]
    fn test_create_poll_30days_duration() -> Result<()> {
        println!("Starting Test");
        let conn = Connection::open_in_memory()?;
    
        create_tables(&conn)?;
    
        let now = Local::now().timestamp();

        let poll = Poll {
            id: Uuid::new_v4(),
            question: "Do You like Rust?".trim().to_string(),
            poll_duration: PollDuration::OneMonth,
            create_date: now,
            expiration_date : now + 24*60*60*30,
            positive_votes: 0,
            negative_votes: 0,
        };
        
        let poll_output = poll::create_poll(&conn, "Do You like Rust?".to_string(), "30".to_string());

        println!("{:?}", poll);
        println!("{:?}", poll_output);

        match poll_output {
            Ok(poll_generated) => {
                assert_eq!(poll.question, poll_generated.question);
                assert_eq!(poll.poll_duration, poll_generated.poll_duration);
                assert_eq!(poll.create_date, poll_generated.create_date);
                assert_eq!(poll.expiration_date, poll_generated.expiration_date);
                assert_eq!(poll.positive_votes, poll_generated.positive_votes);
                assert_eq!(poll.negative_votes, poll_generated.negative_votes);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        Ok(())
    }

    #[test]
    fn test_create_poll_7days_duration() -> Result<()> {
        println!("Starting Test");
        let conn = Connection::open_in_memory()?;
    
        create_tables(&conn)?;
    
        let now = Local::now().timestamp();

        let poll = Poll {
            id: Uuid::new_v4(),
            question: "Do You like Rust?".trim().to_string(),
            poll_duration: PollDuration::OneWeek,
            create_date: now,
            expiration_date : now + 24*60*60*7,
            positive_votes: 0,
            negative_votes: 0,
        };
        
        let poll_output = poll::create_poll(&conn, "Do You like Rust?".to_string(), "7".to_string());

        println!("{:?}", poll);
        println!("{:?}", poll_output);

        match poll_output {
            Ok(poll_generated) => {
                assert_eq!(poll.question, poll_generated.question);
                assert_eq!(poll.poll_duration, poll_generated.poll_duration);
                assert_eq!(poll.create_date, poll_generated.create_date);
                assert_eq!(poll.expiration_date, poll_generated.expiration_date);
                assert_eq!(poll.positive_votes, poll_generated.positive_votes);
                assert_eq!(poll.negative_votes, poll_generated.negative_votes);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        Ok(())
    }

    #[test]
    fn test_create_poll_too_long_question() -> Result<()> {
        println!("Starting Test");
        let conn = Connection::open_in_memory()?;
    
        create_tables(&conn)?;
        
        let poll_output = poll::create_poll(&conn, "A".repeat(151).to_string(), "7".to_string());

        let expected_error =  ValidationError::new(
            "Question is too long. Question only can have up to 150 chars.",
        );

        match poll_output {
            Err(err) => {
                println!("Output error: {:?}", err);

                // Tente converter o erro para ValidationError
                if let Some(validation_error) = err.downcast_ref::<ValidationError>() {
                    assert_eq!(validation_error, &expected_error, "Different Error Messages");
                    Ok(())
                } else {
                    panic!("Expected ValidationError Type");
                }
            }
            Ok(_) => panic!("Expected Error."),
        }       
    }

    #[test]
    fn test_create_poll_invalid_poll_duration() -> Result<()> {
        println!("Starting Test");
        let conn = Connection::open_in_memory()?;
    
        create_tables(&conn)?;
        
        let poll_output = poll::create_poll(&conn, "Do You like Rust?".to_string(), "5".to_string());

        let expected_error =  ValidationError::new(
            "Invalid input_days value. Must be 7 or 30.",
        );

        match poll_output {
            Err(err) => {
                println!("Output error: {:?}", err);

                // Tente converter o erro para ValidationError
                if let Some(validation_error) = err.downcast_ref::<ValidationError>() {
                    assert_eq!(validation_error, &expected_error, "Different Error Messages");
                    Ok(())
                } else {
                    panic!("Expected ValidationError Type");
                }
            }
            Ok(_) => panic!("Expected Error."),
        }       
    }

    #[test]
    fn test_create_poll_empty_question() -> Result<()> {
        println!("Starting Test");
        let conn = Connection::open_in_memory()?;
    
        create_tables(&conn)?;
    
        
        let poll_output = poll::create_poll(&conn, "".to_string(), "5".to_string());

        let expected_error =  ValidationError::new(
            "Question can't be empty.",
        );

        match poll_output {
            Err(err) => {
                println!("Output error: {:?}", err);

                // Tente converter o erro para ValidationError
                if let Some(validation_error) = err.downcast_ref::<poll::ValidationError>() {
                    assert_eq!(validation_error, &expected_error, "Different Error Messages");
                    Ok(())
                } else {
                    panic!("Expected ValidationError Type");
                }
            }
            Ok(_) => panic!("Expected Error."),
        }       
    }
}