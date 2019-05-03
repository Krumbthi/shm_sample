use std::env;
use std::fs;
use std::path::Path;

use std::thread;
use std::time::Duration;

use std::str::from_utf8_unchecked;
use ipc::Semaphore;

const LOOP_COUNTER: i32 = 30;


struct SomeState {
    num_listenners: u32,
    message: [u8; 256],
}


fn from_ut8f_to_null(bytes: &[u8], max_len: usize) -> &str {
    for i in 0..max_len {
        if bytes[i] == 0 {
            return unsafe {from_utf8_unchecked(&bytes[0..i])};
        }
    }
    panic!("Couldnt find null terminator.");
}


fn main() {
    println!("SHM client");
    
    let server_res = Path::new("/dev/shm/server.res");
    let display_res = server_res.display();

	let server_cmd = Path::new("/dev/shm/server.cmd");
    let display_cmd = server_cmd.display();    

    let sem = match Semaphore::new("43", 1) {
    	Ok(sem) => sem,
		Err(s) => panic!("failed to create a semaphore: {}", s)
	};

	// Write to cmd file
	println!("Sent CMD: {:?}", "INCR");
	fs::write(server_cmd, "State: INCR");

	// Read the contents of the buffer again
	let handle = thread::spawn(move || loop {
	   	// manage the semaphore count manually
	    sem.acquire();

	    let res = fs::read_to_string(server_res).expect("Something went wrong reading the file");
	    println!("RES: {}", res);

		sem.release();

		thread::sleep(Duration::from_millis(900));
	});

	for i in 1..LOOP_COUNTER {
		let mut cmd = String::new();
		let m1 = i % 7;
		let m2 = i % 3;
		
		if m1 == 0 {
			cmd = "State: INCR".to_string();
		}
		else if m2 == 0 {
			cmd = "State: DECR".to_string();
		}
		
		fs::write(server_cmd, cmd);

        thread::sleep(Duration::from_secs(1));
    }

	// Write to cmd file
	fs::write(server_cmd, "State: QUIT");

	handle.join().unwrap();

	// Write to cmd file
	fs::write(server_cmd, "State: QUIT");
	// unlock the semaphore
	//drop(sem);
	
}

