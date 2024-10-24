use csv::Reader;
use csv::WriterBuilder;
use std::fs::OpenOptions;
use std::error::Error;

fn main() {
    if let Err(e) = csv_writer() {
        println!("Error writing file: {}", e);
    }

    csv_reader();
}

fn csv_writer() -> Result<(), Box<dyn Error>> {
    let file_name = "bd.csv";
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name)?;

    let mut writer = WriterBuilder::new().from_writer(file);

    writer.write_record(&["pergunta", "votos"])?;
    writer.write_record(&["voce gosta de rust?", "90"])?;
    writer.write_record(&["a", "b"])?;

    Ok(())
}

fn csv_reader(){
    let file_name = "bd.csv";
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