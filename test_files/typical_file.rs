use anyhow::{Context, Result};
use crossterm::{terminal::{Clear, ClearType}, style::{Print}, cursor};

fn main() -> Result<()> {
    let super_variable = 12341234664;
    let another_super_variable = 235131362412341;

    super_variable.checked_add(another_super_variable).content("Failed to add variables together")?;
    print_result(&super_variable);
}

fn print_result(result: &i32){
    println!("The result is {}", result);
}
