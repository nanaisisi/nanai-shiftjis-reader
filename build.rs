fn main() {
    if cfg!(target_os = "windows") && std::env::var_os("CARGO_FEATURE_WIN_REACTOR_UI").is_some() {
        windows_reactor_setup::as_framework_dependent();
        windows_reactor_setup::as_self_contained();
    }
}
