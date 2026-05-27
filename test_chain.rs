fn main() {
    // Test: method chain inside push should work
    let x = std::vec::Vec::<i32>::new()
        .push(
            std::vec::Vec::<i32>::new()
                .push(1)
                .push(2)
                .push(3),
        )
        .push(
            std::vec::Vec::<i32>::new()
                .push(4),
        );
    println!("{:?}", x);
}
