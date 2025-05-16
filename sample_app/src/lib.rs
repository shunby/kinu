#![no_std]

wit_bindgen::generate!({
    world: "sample",
    with: { "wasi:random/random@0.2.5": generate}
});

use wasi::random::random::get_random_u64;
struct Component {}

impl Guest for Component {
    fn run() -> () {
        print("generating a random number...\r\n");
        let ans = get_random_u64() % 100;
        print("generated a number\r\n");
        loop {
            let user_input = input();
            if user_input == ans {
                print("Correct!\r\n");
                break;
            } else if user_input < ans {
                print("Too small\r\n");
            } else {
                print("Too big\r\n")
            }
        }
        print("bye!\r\n");
    }
}

export!(Component);
