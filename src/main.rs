mod file_process;
mod ui;
use windows::ApplicationModel::Package;

fn main() {
    match Package::Current() {
        Ok(package) => match package.Id() {
            Ok(id) => match id.FamilyName() {
                Ok(name) => println!("Package Family Name: {}", name),
                Err(e) => println!("Error getting family name: {}", e),
            },
            Err(e) => println!("Error getting package ID: {}", e),
        },
        Err(_) => println!("Not packaged"),
    }

    let decoded_text = file_process::file_process();
    ui::ui(decoded_text);
}
