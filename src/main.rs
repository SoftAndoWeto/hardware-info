fn main() {
    let os_info = hardware_requiem::get_os_info();
    println!("OS info: {os_info:#?}");
    let hw_info = hardware_requiem::hw::get_hw_info();
    println!("hw info: {hw_info:#?}");
}
