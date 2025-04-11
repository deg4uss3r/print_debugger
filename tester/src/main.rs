use print_debugger::print_debug;

#[print_debug(----> )]
fn main() {

    //let x = 11;
    //let x = 6;
    let x = 1;
    //let y = true;
    let y = false;

    if x > 10 {
        println!("Greater than 10");
        if y {
            println!("if true");
        } else {
            println!("if false");
        }
    } else if x >= 5 {
        println!("between 5 and 10");
        match y {
            true => println!("if else true"),
            false =>println!("if else false"),
        }
    } else {
        println!("less than 5");
        if y {
            println!("else true");
        } else {
            println!("else false");
        }
    }
}
