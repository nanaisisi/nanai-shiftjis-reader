#[cfg(target_os = "windows")]
fn main() {
    windows_reactor_setup::as_self_contained();
}
