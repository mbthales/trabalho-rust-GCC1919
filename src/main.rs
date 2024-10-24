use csv::Reader;
use csv::WriterBuilder;
use std::fs::OpenOptions;

fn main() {
    let file_name = "bd.csv";
    
    csv_writer(file_name);
    csv_reader(file_name);
}

fn csv_writer(file_name: &str) {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name);

    if file.is_err() {
        println!("Error reading file");
        return;
    }

    let file = file.unwrap();
    let mut writer = WriterBuilder::new().from_writer(file);

    let _ = writer.write_record(&["teste", "teste"]);
}

fn csv_reader(file_name: &str){
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