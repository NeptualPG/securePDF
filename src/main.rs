struct Task {
    id: u32,
    title: String,
    done: bool
}


fn main() {
    let mut first = Task {
        id: 1,
        title: "Learn Axum".to_string(),
        done: false,
    };

    first.done = true;

    println!("{} is done: {}", first.title, first.done);

    print_title(&first);
    print_title(&first);

    let maybe_title :Option<String>= Some("Buy milk".to_string());

    match  maybe_title {
        Some(t) => println!("We have a title: {}", t),
        None => println!("No title was given!"),
    }

    let parse: Result<u32, _>  = "42".parse::<u32>();

    match parse {
        Ok(n) => println!("Parsed the number: {}", n),
        Err(e) => println!("That wasn't a number! {}", e),
    }

    


}

fn print_title(task: &Task) {
    println!("Title: {}", task.title);
}
