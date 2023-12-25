fn main() {
    println!("Hello, world!");
    let input = "s";
    let res = convert_u32(input);
    // if let Some(value) = res {
    //     println!("{value}");

    // }
    // else {
    //     eprintln!("can not parse to u32");
    // }
    // if res.is_some() {
    //     let tmp = res.unwrap();
    //     println!("{tmp}");

    // }
    // else {
    //     eprintln!("can not parse to u32");
    // }

    let numbers = vec![1, 2, 3, 4, 5];
    let mut iter = numbers.iter();
    while let Some(next) = iter.next() {
        if next == &3 {
            println!("value here:{}", next);
            iter.next();
        } else {
            println!("value:{}", next);
            iter.next();
        }
    }
    println!("--------------");
    let numbers = vec![1, 2, 3, 4, 5];
    let mut iter = numbers.iter().peekable();
    while let Some(&next) = iter.peek() {
        if next == &3 {
            println!("value here:{}", next);
            iter.next();
        } else {
            println!("value:{}", next);
            iter.next();
        }
    }

    // có 1 sự khác biệt 
    // iter() -> consume 
    // peek() -> xem giá trị trước và ko consum 
    

}

// Trait
// định nghĩa interface( đặc tính ) chung
// đặc tính -> function hay method
pub trait People {
    fn height(&self) -> String;
    fn weight(&self) -> String;
}

struct Peter {}

impl People for Peter {
    fn height(&self) -> String {
        "1m70".to_string()
    }

    fn weight(&self) -> String {
        "70kg".to_string()
    }
}

struct Alice {}

impl People for Alice {
    fn height(&self) -> String {
        "1m60".to_string()
    }

    fn weight(&self) -> String {
        "50kg".to_string()
    }
}

// generic type -> kiểu dữ liệu chung
// struct Point {
//     x: f32,
//     y: f32,
// }

// struct Point2 {
//     x: f64,
//     y: f64,
// }
// placeholder -> f32 hoặc f64
struct Point<T> {
    x: T,
    y: T,
}

// if let Some -> kết hợp giữa việc if x.is_some() -> unwrap()
// if let Ok ->
// if let Err ->
// function convert từ string sang u32
fn convert_u32(input: &str) -> Option<u32> {
    let res = input.parse::<u32>().ok();
    res
}
