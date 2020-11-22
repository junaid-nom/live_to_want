// use crate::utils::Vector2;

// use super::{MapRegion, Location};



// #[derive(Debug)]
// #[derive(Default)]
// pub struct NavRegion {
    
// }

// #[derive(Debug)]
// #[derive(Default)]
// pub struct NavPoint {
//     blocked: bool,
//     point_distances: Vec<Vec<u32>>,
//     is_exit: ExitPoint
// }


// #[derive(Debug)]
// #[derive(Default)]
// pub struct NavigationMap {
//     map: Vec<Vec<NavRegion>>,
// }
// impl NavigationMap {
//     fn update(&mut self, region: Vector2, map_region: &MapRegion) {
//         // update the navRegion
        
//         // if the left/right/up/down access changes then update all the region_distances

//         // PANIC if exit nodes are blocked by a creature. also if exit nodes arent together, like there shouldnt be a permablocked location inbetween 2 exit nodes. like for top if it was OOOXOO thats bad because it can cause strange splits where one region is accessible from another but only from a particular entrance. wish I had a better way to make sure u cant do this
//     }

    
// }
