mod notifier;
//#[cfg(windows)]
//fn main() -> windows_service::Result<()> {
//    //ping_service::run()
//}

//#[cfg(not(windows))]
fn main() {
    notifier::run();
}
