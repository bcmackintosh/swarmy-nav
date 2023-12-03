use std::cmp::{Ordering};

pub fn f64_ordering(x: &f64, y: &f64) -> Ordering {
    if x < y { 
        return Ordering::Less 
    } 
    
    if y < x { 
        return Ordering::Greater 
    } 
    
    return Ordering::Equal;
}

fn get_spread(v: &Vec<f64>) -> f64 {
    // let v_iter = v.into_iter();
    let min_val: f64 = {v.iter().cloned().min_by(f64_ordering).unwrap()};
    let max_val: f64 = {v.iter().cloned().max_by(f64_ordering).unwrap()};
    max_val - min_val
}

pub fn beam_filter(v: &Vec<f64>) -> f64 {
    let spread = get_spread(v);
    // [TODO] Create dynamic width based on variance within the given values.  This would basically be standard deviation.
    let beam = spread * 0.2;
    let mut best_fit: Vec<f64> = vec![];

    for i in 0..v.len() {
        let mut current_fit: Vec<f64> = Vec::<f64>::new();

        for j in 0..v.len() {
            if i == j {
                continue;
            }

            let dist = v[j] - v[i];

            if dist * dist <= beam {
                current_fit.push(v[j]);
            }
        }

        if current_fit.len() > best_fit.len() {
            best_fit = current_fit;
        }
    }
    
    let output = best_fit.iter().sum::<f64>() / best_fit.len() as f64;
    let certainty = best_fit.len() as f64 / v.len() as f64;

    // println!("input: {:?}, output: {:?}, len diff: {:?}, avg: {:?}", v, best_fit, v.len() - best_fit.len(), output);
    // println!("len diff: {:?}, avg: {:?}, certainty: {:?}", v.len() - best_fit.len(), output, certainty);
    
    output
}

pub fn beam_deviation_filter(v: &Vec<f64>) -> f64 {
    if v.len() == 0 {
        return 0.0;
    }

    if v.len() == 1 {
        return v[0];
    }

    let mut working_v = v.clone();
    working_v.sort_by(f64_ordering );

    let v_dev = (working_v.len() as f64 * 0.1) as usize;
    let mid = working_v.len() / 2 as usize;
    let start = mid - v_dev;
    let end = mid + v_dev;

    // [TODO] Create dynamic width based on variance within the given values.  This would basically be standard deviation.
    let mut best_fit: Vec<f64> = vec![];

    for i in start..(end + 1) {
        for j in start..(end + 1) {
            if i == j {
                continue;
            }
            best_fit.push(working_v[j]);
        }
    }
    
    let output = best_fit.iter().sum::<f64>() / best_fit.len() as f64;
    
    output
}