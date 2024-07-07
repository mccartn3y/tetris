use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main(){
    // This code will create a UI for the tetris game. 
    // The complexity of a tetis UI comes from the
    // fact that you have a limited amount of time to
    // make changes to the state of the piece before
    // it moves. This means that we need to have some way
    // of collecting and processing moves for a set time
    // period, after which we shall cease to collect inputs
    // and make some sort of change.
    //
    // At the moment it looks like the best way to do this is
    // leveraging channels. (This will be my first bit of
    // concurrent programming, YaY!). We shall set up two channels,
    // one that collects inputs from the cli and the other which
    // runs a timer for the given period of time. When there is a
    // message from the first channel (i.e. a movement), we shall
    // process it; when there is a message fromt the second channel
    // we shall move on.
    
    let (sender, receiver) = mpsc::channel();
    
    let timeout_duration = 10;
    // set up timer to accept input for
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(timeout_duration));
        sender.send(true).unwrap();
    });
    for recieved in receiver{
        println!("Timer timed out!");
    }
}

