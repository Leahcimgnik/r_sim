/*
Agent trajectory graph:

-> spawn(time)
-> Queue(name1)
-> Server(name1)
-> Queue(name2)
-> Server(name1)
-> Finish

P(spawn_time,process_time,trajectory_name)

trajectory = struct{name:"t1", path:["q1","s1","q2","s1"]}
P1(1,2,"t1")
P2(2,2,"t1")
P3(3,2,"t1")

t0---
t = min(SpawnQ_min,S1_min)



*/


use std::collections::VecDeque;

fn main() {

    let people:Vec<Person> = vec![
        Person{id:1,spawn_time:1,process_time:5},
        Person{id:2,spawn_time:2,process_time:5},
        Person{id:3,spawn_time:3,process_time:5},
        Person{id:4,spawn_time:4,process_time:5},
        Person{id:5,spawn_time:5,process_time:5},
        Person{id:6,spawn_time:6,process_time:5},
    ];

    let mut servers:Vec<Server> = Vec::new();
    let num_servers:u8 = 2;

    for id in 0..num_servers {
        servers.push(Server::new(id+1));
    }

    sim(people,servers);
    // random_sampling_data();

}

struct Person {
    id:u8,
    spawn_time:u8,
    process_time:u8
}

//TODO: Attach a queue for the server, re-write code below to accommodate.
struct Server {
    id:u8,
    available_time:u8,
    person:Person,
}

impl Server {

    fn new(id:u8) -> Server {
        Server {
            id:id,
            available_time:0,
            person:Person{id:0,spawn_time:0,process_time:0},
        }
    }

    fn idle_server(&mut self) {
        self.available_time = u8::MAX;
        self.person = Person{id:0,spawn_time:0,process_time:0};
    }

    fn release_agent(&mut self, sim_time:&u8) {
        if sim_time == &self.available_time {
            println!("At time {} server_{} finished serving {}.", sim_time, self.id, self.person.id);
            self.idle_server(); 
        }
    }

}

fn spawning(spawn_queue:&mut VecDeque<Person>, sim_time:&u8, process_queue:&mut VecDeque<Person>) {
    if spawn_queue.len() > 0 {
        if sim_time == &spawn_queue[0].spawn_time {
            let person:Person = spawn_queue.pop_front().unwrap();
            println!("At time {} person id {} is spawned and put in queue.",sim_time,person.id);
            process_queue.push_back(person);
        }
    }
}

fn move_agents_from_queue_to_server(process_queue:&mut VecDeque<Person>, servers:&mut Vec<Server>, sim_time:&u8) {
    if process_queue.len() > 0 {
        for server in servers.iter_mut() {
            if server.available_time == u8::MAX {
                let person:Person = process_queue.pop_front().unwrap();
                println!("At time {} person id {} is placed in server_{}.",sim_time,person.id,server.id);
                server.person = person;
                server.available_time = sim_time+server.person.process_time;
                break;
            }
        }
    }
}

fn update_sim_time(servers:&Vec<Server>, spawn_queue:&mut VecDeque<Person>, sim_time:&mut u8) {
    let min_server_available_time:u8 = servers.iter().map(|x| x.available_time).min().expect(
        "When calculating the minimum value between the server available times, the vector of servers
        should not be empty, and the available times should be u8::MAX, or the time they finish serving their
        current person."
    );

    if spawn_queue.len() > 0 {
        *sim_time = spawn_queue[0].spawn_time.min(min_server_available_time);
    } else {
        *sim_time = min_server_available_time;
    }
}

fn sim(mut people:Vec<Person>, mut servers:Vec<Server>) {

    let mut process_queue:VecDeque<Person> = VecDeque::new();
    let mut loop_num:u8 = u8::MAX;

    people.sort_by(|a,b| a.spawn_time.cmp(&b.spawn_time));
    let mut spawn_queue:VecDeque<Person> = people.into();

    let mut sim_time:u8 = spawn_queue[0].spawn_time;

    while loop_num > 0 {

        spawning(&mut spawn_queue, &sim_time, &mut process_queue);

        for server in servers.iter_mut() {
            server.release_agent(&sim_time);
        }

        move_agents_from_queue_to_server(&mut process_queue, &mut servers, &sim_time);
        update_sim_time(&servers, &mut spawn_queue, &mut sim_time);

        if sim_time == u8::MAX {
            println!("Simulation is finised. Loops: {}",u8::MAX-loop_num);
            break
        }

        loop_num -= 1;
    }


}
