async fn slow_hello() -> String {
    "hello from an async function".to_string()
}

#[tokio::main]
async fn main (){
    let greeting = slow_hello().await;
    println!("{}", greeting);
}


