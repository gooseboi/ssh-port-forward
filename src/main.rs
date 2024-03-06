use std::{env, fs, process::Command, thread};

fn main() {
    let ports: Vec<_> = fs::read_to_string("ports")
        .expect("Failed reading from ports file")
        .split(',')
        .map(|p| {
            let mut iter = p.split(':').map(|s| {
                s.trim()
                    .parse::<u16>()
                    .expect("Failed to parse port as int")
            });
            let p1 = iter.next().expect("No port was provided");
            let p2 = iter.next().unwrap_or(p1);
            if iter.next().is_some() {
                panic!("Too many ports were provided");
            }
            (p1, p2)
        })
        .collect();

    let remote_host = env::args()
        .nth(1)
        .expect("Remote server was not provided as first argument");

    let port_maps: Vec<_> = ports
        .into_iter()
        .map(|(local_port, remote_port)| {
            (
                local_port,
                remote_port,
                format!("{remote_port}:127.0.0.1:{local_port}"),
            )
        })
        .collect();

    let children: Vec<_> = port_maps
        .iter()
        .map(|p| {
            let mut cmd = Command::new("ssh");
            cmd.arg("-N").args(["-R", &p.2]).arg(&remote_host);
            println!("Running `ssh -N -R {} {}`", p.2, remote_host);
            let child = cmd
                .spawn()
                .unwrap_or_else(|_| panic!("Failed spawning ssh mapping local {} to remote {}", p.0, p.1));
            println!("Spawned ssh port forward mapping local {} to remote {}", p.0, p.1);
            (p.0, p.1, cmd, child)
        })
        .collect();

    let handles: Vec<_> = children
        .into_iter()
        .map(|(local_port, remote_port, mut cmd, mut child)| {
            thread::spawn(move || loop {
                child.wait().expect("Failed waiting for child");
                println!(
                    "ssh process mapping local {local_port} to remote {remote_port} stopped, restarting..."
                );
                child = cmd.spawn().unwrap_or_else(|_| {
                    panic!("Failed spawning new ssh process mapping local {local_port} to remote {remote_port}")
                });
                println!("Restarted ssh port forward mapping local {local_port} to remote {remote_port}");
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("Failed to join thread");
    }
}
