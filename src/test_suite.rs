use std::cmp;
use std::collections::HashMap;
use std::time::SystemTime;
use datetime::LocalDateTime;
use rand::{seq::SliceRandom, thread_rng};
use colored::Colorize;

use crate::identity::Identity;
use crate::location::{DistanceGraph, MomentEdge};
use crate::polar::PolarCoordinates;
use crate::filter::beam_deviation_filter;

use rand::prelude::*;

pub fn page_break() {
    println!("{}", format!("\n{:#<80}\n", ""));
}

pub fn line_break() {
    println!("{}", format!("\n{:-<80}\n", ""));
}

fn get_node_alpha_table() -> Vec<Identity> {
    vec![
        "A".into(),
        "B".into(),
        "C".into(),
        "D".into(),
        "E".into(),
        "F".into(),
        "G".into(),
        "H".into(),
        "I".into(),
        "J".into(),
        "K".into(),
        "L".into(),
        "M".into(),
        "N".into(),
        "O".into(),
        "P".into(),
        "Q".into(),
        "R".into(),
        "S".into(),
        "T".into(),
        "U".into(),
        "V".into(),
        "W".into(),
        "X".into(),
        "Y".into(),
        "Z".into(),
    ]
}

fn get_node_names(nodes: usize) -> Vec<Identity> {
    let mut output: Vec<Identity> = vec![];
    let mut name_map: HashMap<&Identity, usize> = HashMap::new();
    let alpha_list = get_node_alpha_table();

    for name in &alpha_list {
        name_map.insert(name, 0);
    }

    let mut i = 0;

    while i < nodes {
        for alpha in &alpha_list {
            if i == nodes {
                break;
            }

            if let Some(a) = name_map.get_mut(alpha) {
                output.push(format!("{alpha}{a}").into());
                *a += 1;
            }

            i += 1;
        }
    }

    return output;
}

pub fn get_distance_graph(nodes: Vec<(Identity, usize, usize)>) -> DistanceGraph {
    let mut edges: Vec<MomentEdge> = vec![];

    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let x = (nodes[i].1 as f64 - nodes[j].1 as f64) as f64;
            let y = (nodes[i].2 as f64 - nodes[j].2 as f64) as f64;
            let dist = ((x * x + y * y) as f64).sqrt();
            edges.push(MomentEdge { left: nodes[i].0.clone(), right: nodes[j].0.clone(), distance: dist, timestamp: SystemTime::now() });
        }
    }

    return DistanceGraph::from_edges(edges);
}

pub fn create_nodes_with_positions(nodes: usize, grid: (usize, usize)) -> Vec<(Identity, usize, usize)> {
    let mut rng = thread_rng();
    let node_names = get_node_names(nodes);
    let mut nodes: Vec<(Identity, usize, usize)> = vec![];
    let x_index: Vec<usize> = (0..grid.0).collect();
    let y_index: Vec<usize>  = (0..grid.1).collect();
    
    for node in node_names {
        let rand_x = x_index.choose(&mut rng).unwrap();
        let rand_y = y_index.choose(&mut rng).unwrap();
        nodes.push((node.clone(), *rand_x, *rand_y));
    }

    return nodes;
}

pub fn create_grid(nodes: &Vec<(Identity, usize, usize)>) -> Vec<Vec<Identity>> {
    // [TODO] Currently allows for collision within the grid, need to ensure each (x, y) coord is unique.
    let max_x = nodes.iter().cloned().max_by(|i, j| {i.1.cmp(&j.1)}).unwrap().1 + 1;
    let max_y = nodes.iter().cloned().max_by(|i, j| {i.2.cmp(&j.2)}).unwrap().2 + 1;
    let max_name_length = nodes.iter().cloned().max_by(|i, j| {i.0.len().cmp(&j.0.len())}).unwrap().0.len();
    let mut grid: Vec<Vec<Identity>> = vec![vec![format!("{:_<width$}", "", width = max_name_length).into(); max_x]; max_y];

    for node in nodes {
        grid[node.2][node.1] = node.0.clone();
    }

    return grid;
}

pub fn display_nodes(nodes: Vec<(Identity, usize, usize)>) {
    let grid = create_grid(&nodes);
    display_grid(grid);
}

pub fn display_grid(grid: Vec<Vec<Identity>>) {
    for i in (0..grid.len()).rev() {
        println!("{:?}", grid[i]);
    }
}

pub fn create_beacon_grid(nodes: usize, grid: (usize, usize)) -> PolarCoordinates {
    let beacon_nodes = create_nodes_with_positions(nodes, grid);
    let beacons: Vec<Identity> = beacon_nodes.iter().cloned().map(|x| {x.0.clone()}).collect();
    let beacon_graph = get_distance_graph(beacon_nodes);

    return beacon_graph.get_position_graph(&beacons, &beam_deviation_filter);
}

pub fn create_beacon_polar(nodes: usize, max_distance: f64) -> PolarCoordinates {
    let mut rng = thread_rng();
    let mut beacons: Vec<Identity> = vec![];
    let mut beacon_graph = DistanceGraph::new();

    return beacon_graph.get_position_graph(&beacons, &beam_deviation_filter);
}