mod singleton {
    use std::sync::Mutex;

    #[derive(Debug)]
    pub struct Singleton {
        permit: u32,
    }

    impl Singleton {
        pub fn new() -> Option<Self> {
            let mut num_tickets = TICKETS.lock().unwrap();
            if *num_tickets > 0 {
                *num_tickets -= 1;
                Some(Singleton { permit: *num_tickets })
            } else {
                None
            }
        }
    }

    impl Drop for Singleton {
        fn drop(&mut self) {
            let mut num_tickets = TICKETS.lock().unwrap();
            *num_tickets += 1;
        }
    }

    static TICKETS: Mutex<u32> = Mutex::new(3);
}

fn main() {
    use crate::singleton::Singleton;

    let instance = Singleton::new();
    println!("count: {:?}", instance);
    let instance1 = Singleton::new();
    println!("count: {:?}", instance1);
    {
        let instance2 = Singleton::new();
        println!("count: {:?}", instance2);
    }
    let instance3 = Singleton::new();
    println!("count: {:?}", instance3);
    let instance4 = Singleton::new();
    println!("count: {:?}", instance4);
}
