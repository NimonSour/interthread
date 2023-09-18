

#[test]

fn debut_std_test() {

    static N: u16 = 1000;
    
    pub fn debut ()-> std::sync::Arc<std::time::SystemTime> {
        static LAST: std::sync::Mutex<std::time::SystemTime> = std::sync::Mutex::new(std::time::SystemTime::UNIX_EPOCH);
    
        let mut last_time = LAST.lock().unwrap();
        let mut next_time = std::time::SystemTime::now();
    
        // we check for 'drift'
        // as described in docs 
        while !(*last_time < next_time)  {
            // in case if they are just equal
            // add a nano but don't break the loop yet
            if *last_time == next_time {
                next_time += std::time::Duration::new(0, 1);
            } else {
                next_time = std::time::SystemTime::now();
            }
        }
        // update LAST 
        *last_time = next_time.clone();
        std::sync::Arc::new(next_time)
    }
    
    
    
    let hn = vec![
    
        std::thread::spawn(
            move || { (0..N).into_iter().map(|x|(1,x,debut()) ).collect::<Vec<_>>() }
        ),
        std::thread::spawn(
            move || { (0..N).into_iter().map(|x|(2,x,debut()) ).collect::<Vec<_>>() }
        ),
        std::thread::spawn(
            move || { (0..N).into_iter().map(|x|(3,x,debut()) ).collect::<Vec<_>>() }
        ),
        std::thread::spawn(
            move || { (0..N).into_iter().map(|x|(4,x,debut()) ).collect::<Vec<_>>() }
        ),
        std::thread::spawn(
            move || { (0..N).into_iter().map(|x|(5,x,debut()) ).collect::<Vec<_>>() }
        ),
    ];
    
    let result = hn.into_iter().map(|x| x.join()).collect::<Vec<_>>();
    let mut res  = result.into_iter().map(|x| x.unwrap()).flat_map(|x|x).collect::<Vec<_>>(); 
    
    res.sort_by(|a,b| a.2.partial_cmp(&b.2).unwrap()  );
    
    
    // let trbl = res.get(7).unwrap().clone();
    // res.insert(20,trbl);
    
    
    let mut faults = 0;
    while let Some(item) = res.pop(){ 
        if  res.iter().any(|x| x.2 == item.2 ) { 
            faults += 1;
        } 
    }
    // println!("Faults - {}",faults);
    assert_eq!(faults,0);

} 


#[test]

fn debut_async_tokio_test() {
    // 10 000   - time: 1.11s -  (9009 id/sec)
    // 80 000   - time: 68 s  -  (1176 id/sec)
    static N: u16 = 100;
    
    pub async fn debut ()-> std::sync::Arc<std::time::SystemTime> {
        static LAST: std::sync::Mutex<std::time::SystemTime> = std::sync::Mutex::new(std::time::SystemTime::UNIX_EPOCH);
    
        let mut last_time = LAST.lock().unwrap();
        let mut next_time = std::time::SystemTime::now();
    
        // we check for 'drift'
        // as described in docs 
        while !(*last_time < next_time)  {
            // in case if they are just equal
            // add a nano but don't break the loop yet
            if *last_time == next_time {
                next_time += std::time::Duration::new(0, 1);
            } else {
                next_time = std::time::SystemTime::now();
            }
        }
        // update LAST 
        *last_time = next_time.clone();
        std::sync::Arc::new(next_time)
    }

    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(
        async {

    let mut handles = Vec::new();

    for i in 0..100 {
        let handle = tokio::spawn(async move {
            let mut loc = Vec::new();
            for x in 0..N{
                loc.push( (i,x,debut().await) )
            }
            loc
        });
        handles.push(handle);
    }

    let mut hns  = Vec::new();
    for hn in handles {
        hns.push( hn.await.unwrap());
    }
    let mut res  = hns.into_iter().flat_map(|x|x).collect::<Vec<_>>(); 

    res.sort_by(|a,b| a.2.partial_cmp(&b.2).unwrap() );

    // let trbl = res.get(7).unwrap().clone();
    // res.insert(20,trbl);
    
    
    let mut faults = 0;
    while let Some(item) = res.pop(){ 
        if  res.iter().any(|x| x.2 == item.2 ) { 
            faults += 1;
        } 
    }
    // println!("Faults - {}",faults);
    assert_eq!(faults,0);

    });
}



#[test]

fn debut_std_tokio_test() {
    // 10 000   - time: 1.07s ( 9345 id/sec )
    // 80 000   - time: 67 s  ( 1190 id/sec )
    static N: u16 = 100;
    
    pub fn debut ()-> std::sync::Arc<std::time::SystemTime> {
        static LAST: std::sync::Mutex<std::time::SystemTime> = std::sync::Mutex::new(std::time::SystemTime::UNIX_EPOCH);
    
        let mut last_time = LAST.lock().unwrap();
        let mut next_time = std::time::SystemTime::now();
    
        // we check for 'drift'
        // as described in docs 
        while !(*last_time < next_time)  {
            // in case if they are just equal
            // add a nano but don't break the loop yet
            if *last_time == next_time {
                next_time += std::time::Duration::new(0, 1);
            } else {
                next_time = std::time::SystemTime::now();
            }
        }
        // update LAST 
        *last_time = next_time.clone();
        std::sync::Arc::new(next_time)
    }

    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(
        async {

    let mut handles = Vec::new();

    for i in 0..100 {
        let handle = tokio::spawn(async move {
            let mut loc = Vec::new();
            for x in 0..N{
                loc.push( (i,x,debut()) )
            }
            loc
        });
        handles.push(handle);
    }

    let mut hns  = Vec::new();
    for hn in handles {
        hns.push( hn.await.unwrap());
    }
    let mut res  = hns.into_iter().flat_map(|x|x).collect::<Vec<_>>(); 

    res.sort_by(|a,b| a.2.partial_cmp(&b.2).unwrap() );

    // let trbl = res.get(7).unwrap().clone();
    // res.insert(20,trbl);
    
    
    let mut faults = 0;
    while let Some(item) = res.pop(){ 
        if  res.iter().any(|x| x.2 == item.2 ) { 
            faults += 1;
        } 
    }
    // println!("Faults - {}",faults);
    assert_eq!(faults,0);

    });
}

