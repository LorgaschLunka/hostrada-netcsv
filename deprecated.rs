// fn plot_data(data: Vec<(DateTime<Utc>, f64)>) {

//     let root = BitMapBackend::new("chart.png", (1280, 720)).into_drawing_area();
//     root.fill(&WHITE).unwrap();

//     let mut chart = ChartBuilder::on(&root)
//         .caption("Chart", ("sans-serif", 25))
//         .margin(20)
//         .x_label_area_size(40)
//         .y_label_area_size(30)
//         .build_cartesian_2d(
//             data.first().unwrap().0..data.last().unwrap().0,
//             0.0..30.0,
//         ).unwrap();

//     chart.configure_mesh().draw().unwrap();

//     chart.draw_series(LineSeries::new(
//         data.iter().map(|(t, y)| (*t, *y)),
//         &BLUE,
//     )).unwrap();

//     root.present().unwrap();
// }


// #[derive(Debug, thiserror::Error)]
// pub enum DownloadError {
//     #[error(transparent)]
//     RequestErr(#[from] reqwest::Error),
    
//     #[error(transparent)]
//     MetadataToStrErr(#[from] reqwest::header::ToStrError),

//     #[error(transparent)]
//     ParseFloatErr(#[from] ParseFloatError),

//     #[error("I/O error while accessing {path}: {source}")]
//     IOErr {
//         source: std::io::Error,
//         path: std::path::PathBuf,
//     },

//     #[error(transparent)]
//     ReaderWriterErr(#[from] std::io::Error),

//     #[error(transparent)]
//     ConfigError(#[from] ConfigError),
// }




// #[error("Could not find key {key} in file metadata")]
    // MetadataKeyErr {
    //     key: String,
    // },
    
    
    // Approximates the nearest pixel via config jumps -> this does not work
    // New Approach: Try to jump pixels until matching lat, than until matching lon; start in the middle
    // -> this could work, if i alternate between x and y step. If not, it does not work because of the curvature of lat/lon axis
    
    pub fn nearest_pixel_at_coord_approx(&self, lat: f64, lon: f64) {
        // get middle pixel 
        let start = self.pixel_at(720/2, 938/2).unwrap();
        
        // move negatively (=left) when center grid point latitude is larger than searched latitude

        let lat_sign = if start.lat > lat { -1 } else { 1 };

        let lon_sign = if start.lon > lon { -1 } else { 1 };

        // TODO handle first step later; after loop is functional
        let mut new_x= start.x as i32;
        let mut new_y= start.y as i32;
        loop {
            // take a step
            new_y += 1*lat_sign;


            let current_pixel = self.pixel_at(start.x, new_y as u16).unwrap();

            let current_lat_sign = if current_pixel.lat > lat { -1 } else { 1 };
            if current_lat_sign != lat_sign {
                // handle switch; could be a break, i dont know yet
                // find out, if the last or second to last pixel is the correct one
                let last_pixel_dist = self.haversine((current_pixel.lat, current_pixel.lon), (lat, lon), 6371000.0);
                
                let second_to_last_pixel = self.pixel_at(start.x, (new_x-1*lat_sign) as u16).unwrap();

                let second_to_last_pixel_dist = self.haversine((second_to_last_pixel.lat, second_to_last_pixel.lon), (lat, lon), 6371000.0);

                let final_y = if last_pixel_dist < second_to_last_pixel_dist {new_y} else {new_y-1*lat_sign};

                dbg!(final_y);
                break
            }
        }


    }

// let bytes_result = fs::read("./cache/grid.bin");
// if bytes_result.is_ok() {
//     println!("Found /cache/grid.bin; using.");
//     let grid: Vec<HostradaGridPixel> = postcard::from_bytes(&bytes_result.unwrap()).expect("Failed to deserialize grid.bin");

//     return grid;
// }
// GEFÄLLT MIR BESSER:
// if let Ok(bytes) = fs::read("./cache/grid.bin") {
//     info!("Found ./cache/grid.bin; using.");
//     if let Ok(grid) = postcard::from_bytes(&bytes) {
//         return grid;
//     } else {
//         warn!("Failed to deserialize grid.bin");
//     }
// }


// info!("Serializing grid into binary file...");
// let bytes = postcard::to_stdvec(&grid).expect("Couldn't convert HasMap to bytes.");

// match fs::create_dir_all("./cache") {
//     Ok(_) => (),
//     Err(e) => warn!("Failed to create directory ./cache! (Error: {e})"),
// }
// let bin_file = match fs::File::create("./cache/grid.bin") {
//     Ok(bin_file) => Some(bin_file),
//     Err(_) => {
//         warn!("Couldn't create file /cache/grid.bin.");
//         None
//     },
// };

// match bin_file {
//     Some(mut file) => match file.write_all(&bytes) {
//         Ok(_) => (),
//         Err(e) => warn!("Failed to write bytes to file! (Error: {e})"),
//     },
//     None => (),
// }


// THREADPOOL MULTIPROCESSING
// IDEA: MAKE THREADPOOL, LET EACH HANDLE ONE DATASET (=1 FILE) 
// Arc+Mutex for all threads to edit the rows to write 

// let n_workers = 6;
// let pool = ThreadPool::new(n_workers);

// let (tx, rx) = mpsc::channel();

// for dataset in datasets {
//     let tx = tx.clone();
//     pool.execute(move || {
//         let start_date = dataset
//             .time_map
//             .iter()
//             .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
//             .unwrap()
//             .0;
//         let end_date = dataset
//             .time_map
//             .iter()
//             .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
//             .unwrap()
//             .0
//             .clone();
        
//         let mut current = *start_date;

//         let mut row: Vec<(DateTime<Utc>, f64)> = Vec::new();
//         while current <= end_date{

//             let val = dataset.value_at("tas", &current, 461, 333).unwrap();

//             row.push((current, val));
//             current += Duration::hours(1);
//         }
//         tx.send(row).expect("Channel waiting for the pool?");
//         println!("A thread finished");
//     });

// }
// drop(tx);
// println!("Waiting for threads to complete");
// let rows: Vec<Vec<(DateTime<Utc>, f64)>> = rx
//     .into_iter()
//     .collect();

// for row in rows {
//     for tup in row {
//         wtr.serialize(tup).unwrap();
//     }
// }
// println!("All values with timestamp for a pixel with threadool took {:?}", start.elapsed());


// let vars = dataset.file().variables();

// for var in vars {
//     if var.name() == "tas" {
//         for attr in var.attributes() {
//             println!("{:?}: {:?}", attr.name(), attr.value().unwrap());
//             let test = attr.value().unwrap();
//             match test {

//                 AttributeValue::Uchar(val) => println!("{:?}", val),
//                 AttributeValue::Uchars(val) => println!("{:?}", val),
//                 AttributeValue::Schar(val) => println!("{:?}", val),
//                 AttributeValue::Schars(val) => println!("{:?}", val),
//                 AttributeValue::Ushort(val) => println!("{:?}", val),
//                 AttributeValue::Ushorts(val) => println!("{:?}", val),
//                 AttributeValue::Short(val) => println!("{:?}", val),
//                 AttributeValue::Shorts(val) => println!("{:?}", val),
//                 AttributeValue::Uint(val) => println!("{:?}", val),
//                 AttributeValue::Uints(val) => println!("{:?}", val),
//                 AttributeValue::Int(val) => println!("{:?}", val),
//                 AttributeValue::Ints(val) => println!("{:?}", val),
//                 AttributeValue::Ulonglong(val) => println!("{:?}", val),
//                 AttributeValue::Ulonglongs(val) => println!("{:?}", val),
//                 AttributeValue::Longlong(val) => println!("{:?}", val),
//                 AttributeValue::Longlongs(val) => println!("{:?}", val),
//                 AttributeValue::Float(val) => println!("{:?}", val),
//                 AttributeValue::Floats(val) => println!("{:?}", val),
//                 AttributeValue::Double(val) => println!("{:?}", val),
//                 AttributeValue::Doubles(val) => println!("{:?}", val),
//                 AttributeValue::Str(val) => println!("{:?}", val),
//                 AttributeValue::Strs(val) => println!("{:?}", val),


//         }
//     }
    
//     }
// }


// fn unwrap_attribute_value(attr: AttributeValue) {
//     match attr {
//         AttributeValue::Uchar(val) => val,
//         AttributeValue::Uchars(val) => val,
//         AttributeValue::Schar(val) => val,
//         AttributeValue::Schars(val) => val,
//         AttributeValue::Ushort(val) => val,
//         AttributeValue::Ushorts(val) => val,
//         AttributeValue::Short(val) => val,
//         AttributeValue::Shorts(val) => val,
//         AttributeValue::Uint(val) => val,
//         AttributeValue::Uints(val) => val,
//         AttributeValue::Int(val) => val,
//         AttributeValue::Ints(val) => val,
//         AttributeValue::Ulonglong(val) => val,
//         AttributeValue::Ulonglongs(val) => val,
//         AttributeValue::Longlong(val) => val,
//         AttributeValue::Longlongs(val) => val,
//         AttributeValue::Float(val) => val,
//         AttributeValue::Floats(val) => val,
//         AttributeValue::Double(val) => val,
//         AttributeValue::Doubles(val) => val,
//         AttributeValue::Str(val) => val,
//         AttributeValue::Strs(val) => val,
//     }
// }


