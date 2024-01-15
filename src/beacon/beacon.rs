use crate::identity::Identity;
use crate::polar::Radial;
use crate::signal::Signal;

use std::collections::{VecDeque, HashMap};
use std::sync::Arc;
use std::thread::{self, JoinHandle, sleep, Thread};
use std::time::{Duration, SystemTime};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::fmt;

const LOOP_TIME: Duration = Duration::from_millis(500);
const POLL_TIME: Duration = Duration::from_millis(50);

thread_local! {
    static BEACON_CANCEL: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

#[derive(Debug, Clone)]
pub struct BeaconSignal {
    pub id: Identity,
    pub distance: f64,
    pub timestamp: SystemTime,
}

impl Signal for BeaconSignal {
    fn serialize(&self) -> String {
        format!("{} {} {}", self.id, self.distance, self.timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis())
    }

    fn deserialize(data: String) -> BeaconSignal {
        let mut split = data.split(" ");
        let id = split.next().unwrap().into();
        let distance = split.next().unwrap().parse::<f64>().unwrap();
        let timestamp = SystemTime::UNIX_EPOCH + Duration::from_millis(split.next().unwrap().parse::<u64>().unwrap());

        BeaconSignal {
            id: id,
            distance: distance,
            timestamp: timestamp,
        }
    }
}

pub struct Beacon {
    pub id: Identity,
    pub position: Radial,
    cancel: Arc<AtomicBool>,
    queue: Option<Sender<BeaconSignal>>,
    last_update: SystemTime,
    distance_cache: HashMap<Identity, (f64, SystemTime)>,
    process_receiver: Receiver<BeaconSignal>,
    process_sender: Sender<BeaconSignal>,
}

fn wait_time() {
    sleep(Duration::from_millis(1));
}

impl Beacon {
    pub fn new(id: Identity, position: Radial) -> Beacon {
        let (tx, rx) = channel::<BeaconSignal>();

        Beacon {
            id: id,
            position: position,
            cancel: Arc::new(AtomicBool::new(false)),
            queue: None,
            last_update: SystemTime::now(),
            distance_cache: HashMap::new(),
            process_receiver: rx,
            process_sender: tx,
        }
    }

    pub fn receive(&self, data: String) {
        self.queue.as_ref().unwrap().send(BeaconSignal::deserialize(data)).unwrap();
    }

    fn start(&self, rx: Receiver<BeaconSignal>) {
        let id = self.id.clone();
        let tx = self.process_sender.clone();
        let cancel = self.cancel.clone();

        thread::spawn(move || {
            loop {
                if cancel.load(Ordering::Acquire) {
                    println!("received stop {}", id);
                    break;
                }
                
                println!("doing thing");
                let init_time = SystemTime::now();
                sleep(Duration::from_millis(20));
                let finish_time = SystemTime::now();
                println!("beacon: {} finished in {:?}", id, finish_time);
                tx.send(BeaconSignal {id: "A0".into(), distance: 100.0, timestamp: SystemTime::now()}).unwrap();
                sleep(LOOP_TIME - finish_time.duration_since(init_time).unwrap_or(Duration::from_millis(0)));
            }
        
            println!("stopping beacon");
        });
    }

    // pub fn listen(&self, conn: VecDeque<Radial>) {
    pub fn listen(&mut self) {
        let (tx, rx) = channel::<BeaconSignal>();

        // [TODO] Mock up a listener that creates a thread to wait for a piece of data and processes every 50ms.
        self.start(rx);

        self.queue = Some(tx);
    }

    pub fn get_distances(&mut self) -> Vec<(Identity, f64)> {
        let current_time = SystemTime::now();
        if current_time.duration_since(self.last_update).unwrap().as_millis() < POLL_TIME.as_millis() {
            return self.distance_vec();
        }

        let mut next_item = self.process_receiver.try_recv();

        while next_item.is_ok() {
            let signal = next_item.as_ref().unwrap();
            self.distance_cache.entry(signal.id.clone()).or_insert((signal.distance, signal.timestamp));
        }

        return self.distance_vec();
    }

    fn distance_vec(&self) -> Vec<(Identity, f64)> {
        return self.distance_cache.iter().map(|x| (x.0.clone(), x.1.clone().0)).collect();
    }

    pub fn stop(&mut self) {
        // [TODO] This doesn't work at all, wut?
        self.cancel.store(true, Ordering::Release);
        // drop(self.sender.take());
    }
}