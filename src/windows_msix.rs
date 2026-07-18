#[cfg(target_os = "windows")]
use windows::ApplicationModel::Package;

#[cfg(target_os = "windows")]
pub fn check_msix_package() {
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
}
