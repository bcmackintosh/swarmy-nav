use std::{sync::Arc, thread::sleep, time::Duration};

mod agent;
mod agent_manager;
mod beacon;
mod filter;
mod identity;
mod location;
mod polar;
mod signal;
mod test_suite;

use filter::beam_deviation_filter;
use test_suite::*;

use crate::{location::{MomentEdge, DistanceGraph}, identity::Identity, polar::{PolarCoordinates, Radial, add_radials}, agent::Agent, beacon::beacon::Beacon};
use crate::polar::get_unknown_triangle_angle;

use std::time::SystemTime;

use datetime::LocalDateTime;
use rand::prelude::*;
use rand::thread_rng;

// [TODO] Create testing to prove grid created was resolved correctly.

fn main() {
    page_break();
    test_polar_coordinate_beacons();
    page_break();
}

fn test_movement_many() {
    println!("Moving Many");
    // [TODO] Create many agents.
    // [TODO] Make agents move simultaneously in random directions.
    // [TODO] Make agents move at different time intervals with different alignments to update frequencies.  Need to be able to adjust beacon update times to agent update times.
}

fn test_polar_coordinate_beacons() {
    println!("Initializing Coordinates");
    println!("Setting Configs");
    let max_distance = 100.0;
    println!("Creating Beacons");

    // [TODO] Create beacons with polar coordinates.
    // let beacon_positions = create_beacon_polar(10, max_distance);

    // println!("beacon_positions: {:?}", beacon_positions);

    println!("testing beacon thread");
    
    {
        let mut beacon = Beacon::new("0".into(), Radial {id: "0".into(), radius: 0.0, angle: 0.0});
        beacon.listen();
        let mut beacon_2 = Beacon::new("1".into(), Radial {id: "1".into(), radius: 0.0, angle: 0.0});
        beacon_2.listen();
        sleep(Duration::from_millis(1000));
        beacon.stop();
        beacon_2.stop();
    }

    println!("stop called");
    sleep(Duration::from_millis(50));

}

fn test_multi_beacon_tracking() {
    println!("Initializing Coordinates");
    println!("Setting Configs");
    let grid = (10, 10);
    println!("Creating Beacons");
    let beacon_positions = create_beacon_grid(10, grid);
    // [TODO] Properly manage multiple beacons, with them tracking movement of each agent separately.
    // [TODO] Have beacons actually construct agent position based on distance from agent.  Origin becomes much messier without the universal reference frame.
    // [TODO] Have differing updates for beacons, where the agent moves in real time, and the beacons have to piece together where the agent is.
    // [TODO] Confirm movement of agents are internally tracked within the agents, so that requesting the position of the agent is done from each beacon.


}

fn test_movement_one() {
    println!("Initializing Coordinates");
    println!("Setting Configs");
    let grid = (10, 10);
    println!("Creating Beacons");
    let beacon_nodes = create_nodes_with_positions(10, grid);
    let beacons: Vec<Identity> = beacon_nodes.iter().cloned().map(|x| {x.0.clone()}).collect();
    let beacon_graph = get_distance_graph(beacon_nodes);
    let beacon_coords = beacon_graph.get_position_graph(&beacons, &beam_deviation_filter);
    println!("Beacons Created");

    println!("Creating Agents");
    let agent_nodes = create_nodes_with_positions(10, grid);
    let agents: Vec<Identity> = agent_nodes.iter().cloned().map(|x| {x.0.clone()}).collect();
    let mut agent_graph = get_distance_graph(agent_nodes.clone());
    let mut agent_coords = agent_graph.get_position_graph(&agents, &beam_deviation_filter);
    println!("Agents Created");

    line_break();
    let mut rng = thread_rng();
    
    let mut agent = Agent::new("0".into(), 0, agent_coords.radials.get(&agent_nodes[1].0).unwrap().clone(), 100.0);
    
    println!("Agent: {:?}", agent);
    let mut radials: Vec<Radial> = vec![];
    
    for _ in 0..10 {
        let new_angle = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let new_radius = rng.gen::<f64>() * 100.0;
        let radial = Radial {id: agent.position.id.clone(), radius: new_radius, angle: new_angle};

        radials.push(radial.clone());
    
        println!("Radial added: {:?}", radial.to_degrees());
        agent.send_position(&radial);
    }

    line_break();
    
    println!("Moving Agent {} nodes", radials.len());
    while agent.current_coord.is_some() {
        agent.get_position();
    }
    println!("Agent Moved");
    
    line_break();
    
    let target_position = radials[radials.len() - 1].clone();
    
    println!("Target Position: {:?}, Agent Position: {:?}, Difference: {:?}", &target_position.to_degrees(), agent.position.to_degrees(), agent.position.subtract(&target_position).to_degrees());

}

fn test_coordinates() {
    let nodes = create_nodes_with_positions(10, (10, 10));
    let beacons: Vec<Identity> = nodes.iter().cloned().map(|x| {x.0.clone()}).collect();
    println!("beacons: {:?}", beacons);
    line_break();
    let grid = create_grid(&nodes);
    display_grid(grid);
    line_break();
    let dg = get_distance_graph(nodes);
    println!("distance graph created.");
    println!("coords: {:?}", dg.get_position_graph(&beacons, &beam_deviation_filter));
}

fn test_position_graph() {
    let beacons: Vec<Arc<str>> = vec!["a".into(),
    "b".into(),
    "c".into(),
    "d".into(),
    "e".into(),
    "f".into(),
    "g".into(),
    "h".into(),
    ];

    let mut edges: Vec<MomentEdge> = vec![];
    let mut rng = thread_rng();

    for i in 0..beacons.len() {
        for j in (i + 1)..beacons.len() {
            if i == j {
                continue;
            }

            for _ in 0..1_000 {
                let mult: f64 = rng.gen();
                let distance = ((j - i) as f64) * (mult + 0.5);
                // println!("rand: {:?}, dist: {:?}", mult, distance);
                edges.push(MomentEdge::new(beacons[i].clone(), beacons[j].clone(), distance, LocalDateTime::now()));
            }
        }
    }

    let mut dg = DistanceGraph::from_edges(edges);

    let start = SystemTime::now();
    dg.get_position_graph(&beacons, &beam_deviation_filter);
    let end = SystemTime::now();
    let duration = end.duration_since(start);
    println!("in {:?} milliseconds", duration);
}