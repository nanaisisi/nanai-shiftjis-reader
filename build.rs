fn main() {
    if cfg!(target_os = "windows") {
        windows_reactor_setup::as_self_contained();
    }
}
