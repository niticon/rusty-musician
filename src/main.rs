fn main() {
    use std::io::{stdin,stdout,Write};
    let mut user_input = String::new();
    print!("Please enter the name of an artist: ");
    let _=stdout().flush();
    stdin().read_line(&mut user_input).expect("Invalid artist name");
    if let Some('\n')=user_input.chars().next_back() {
	user_input.pop();
    }
    if let Some('\r')=user_input.chars().next_back() {
	user_input.pop();
    }
    println!("You typed: {}",user_input);
    println!("Cooking up something by: {}",user_input);
}
