use rusqlite::{Connection, Result, types::ToSqlOutput, ToSql, types::FromSqlError, types::ValueRef, types::FromSql};
use std::{io, fmt};
use chrono::Local;
use uuid::Uuid;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct Poll {
   pub id: Uuid, //Could have used a sequential one but I find it easier
   pub question: String,
   pub poll_duration: PollDuration,
   pub create_date: i64,
   pub expiration_date: i64,
   pub positive_votes: i16,
   pub negative_votes: i16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PollDuration {
    OneWeek = 7,
    OneMonth = 30,
}

#[derive(Debug)]
pub struct ValidationError {
   pub details: String,
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

impl PartialEq for ValidationError {
   fn eq(&self, other: &Self) -> bool {
        self.details == other.details
   }
}

impl Error for ValidationError {}

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


pub fn get_polls(conn: &Connection) -> Result<Vec<Poll>> {
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
pub fn create_poll(conn: &Connection, question: String, input_days: String) -> Result<Poll, Box<dyn Error>>  {
   let create_date: i64;
   let expiration_date: i64;


            
            
            
   if question.trim().chars().count() > 0 {
      if question.chars().count() <= 150{
            
      } else{
            println!("\nQuestion is too long. Question only can have up to 150 chars.");
            return Err(Box::new(ValidationError::new(
               "Question is too long. Question only can have up to 150 chars.",
            )));
      }
   } else{
      println!("\nQuestion can't be empty");
      return Err(Box::new(ValidationError::new(
            "Question can't be empty.",
      )));
   }
   
   

   let input_days_trimmed = input_days.trim();
            
   let poll_duration = match input_days_trimmed.parse::<i8>() {
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
      Ok(_) | Err(_) => {
            return Err(Box::new(ValidationError::new(
               "Invalid input_days value. Must be 7 or 30.",
            )));
      }
   };

   

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
   );

   println!("\nPoll Created!");

   Ok(poll)
}

pub fn edit_poll(conn: &Connection) -> Result<()>  {
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
         // let _ = menu(conn);
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

   // let _ = menu(conn);

   Ok(())
}

pub fn delete_poll(conn: &Connection) -> Result<()> {
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
         // let _ = menu(conn);
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

   // let _ = menu(conn);

   Ok(())
}
